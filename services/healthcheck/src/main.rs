use log::{debug, error, info, LevelFilter};
use reqwest;
use sqlx::{query, Pool, Postgres};
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;
use std::{env, sync::Arc};
use tokio::{self, sync::Mutex};
use uuid::Uuid;

#[derive(Debug)]
struct Endpoint {
    url: String,
    frequency: Duration,
    is_down: bool,
    is_up_counter: i32,
    task: tokio::task::JoinHandle<()>,
}

impl Endpoint {
    fn new(url: String, frequency: Duration, task: tokio::task::JoinHandle<()>) -> Endpoint {
        Endpoint {
            url,
            task,
            frequency,
            is_down: false,
            is_up_counter: 0,
        }
    }
}

//define endpoint_data type
type EndpointData = Arc<Mutex<HashMap<String, Endpoint>>>;

async fn update_endpoint(
    pool: &Pool<Postgres>,
    endpoint: &Endpoint,
    outage_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    if outage_id.is_none() {
        query!(
            "UPDATE endpoint_data SET is_down = $1, last_ping_time = NOW() WHERE http_address = $2",
            endpoint.is_down,
            endpoint.url
        )
        .execute(pool)
        .await?;
        return Ok(());
    } else {
        query(
            "UPDATE endpoint_data SET is_down = $1, last_ping_time = NOW(), outage_id = $2, 
            ntf_is_being_handled = False,
            ntf_is_first_notification_sent = False,
            ntf_is_second_notification_sent = False,
            ntf_first_responded = False
         WHERE http_address = $3",
        )
        .bind(endpoint.is_down)
        .bind(outage_id)
        .bind(endpoint.url.clone())
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn health_check(
    address: String,
    endpoint_data: EndpointData,
    pool: &Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    loop {
        // let endpoint = endpoint.get_mut();
        info!("Checking {}", address);
        let response = reqwest::get(&address).await;
        let mut endpoint_data = endpoint_data.lock().await;
        let endpoint = endpoint_data.get_mut(&address).unwrap();
        let mut outage_id: Option<Uuid> = None;
        match response {
            Ok(resp) if resp.status().is_success() => {
                if endpoint.is_down {
                    if endpoint.is_up_counter > 2 {
                        endpoint.is_down = false;
                        info!("{} is back up", endpoint.url);
                    } else {
                        info!("{} is up for now", endpoint.url);
                        endpoint.is_up_counter += 1;
                    }
                } else {
                    debug!("{} is still up", endpoint.url);
                }
            }
            _ => {
                if !endpoint.is_down {
                    info!("{} is down", endpoint.url);
                    endpoint.is_down = true;
                    outage_id = Some(uuid::Uuid::new_v4());
                } else {
                    debug!("{} is still down", endpoint.url);
                }
            }
        }
        update_endpoint(pool, endpoint, outage_id).await?;
        tokio::time::sleep(endpoint.frequency).await;
    }
}

async fn poll_for_new_endpoint_data(
    pool: Pool<Postgres>,
    endpoint_data_mut: EndpointData,
    freq: Duration,
) -> Result<(), sqlx::Error> {
    let max_endpoint_data: i64 = env::var("MAX_endpoint_data")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("MAX_endpoint_data must be a valid integer");
    loop {
        let mut endpoint_data = endpoint_data_mut.lock().await;
        if endpoint_data.len() > 0 {
            let recs = sqlx::query!(
                "SELECT http_address, is_removed, frequency FROM endpoint_data WHERE http_address = ANY($1)",
                &endpoint_data.values().map(|e| e.url.clone()).collect::<Vec<String>>()
            ).fetch_all(&pool).await?;
            for rec in recs {
                let address = rec.http_address.clone();
                let is_removed = rec.is_removed;
                let endpoint = endpoint_data.get_mut(&address).unwrap();
                if is_removed {
                    endpoint.task.abort();
                    info!("Aborted task for {}", address);
                    endpoint_data.remove(&address);
                } else {
                    let frequency = Duration::from_micros(rec.frequency.microseconds as u64);
                    if endpoint.frequency != frequency {
                        info!(
                            "Changing frequency for {} from {:?} to {:?}",
                            address, endpoint.frequency, frequency
                        );
                        endpoint.frequency = frequency;
                    }
                }
            }
        }
        let endpoint_data_len = endpoint_data.len();
        info!("Currently {} endpoint_data", endpoint_data_len);
        drop(endpoint_data);

        let endpoint_data_fetch_number = max_endpoint_data - endpoint_data_len as i64;
        info!("Fetching {} endpoint_data", endpoint_data_fetch_number);
        let mut transaction = pool.begin().await?;
        let recs = sqlx::query!(
            "SELECT http_address, frequency FROM endpoint_data WHERE (last_ping_time IS NULL OR last_ping_time + 3 * frequency < NOW()) AND not is_removed LIMIT $1 FOR UPDATE SKIP LOCKED", 
            endpoint_data_fetch_number)
            .fetch_all(&mut*transaction)
            .await?;
        sqlx::query!(
            "UPDATE endpoint_data SET last_ping_time = NOW() WHERE http_address = ANY($1)",
            &recs
                .iter()
                .map(|e| e.http_address.clone())
                .collect::<Vec<String>>()
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        info!("Found {} endpoint_data", recs.len());
        if recs.len() == 0 {
            tokio::time::sleep(freq).await;
            continue;
        }

        let mut tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
        for rec in &recs {
            let address = rec.http_address.clone();
            let endpoint_data_mut_clone = Arc::clone(&endpoint_data_mut);
            let pool_copy = pool.clone();
            let task = tokio::spawn(async move {
                if let Err(e) = health_check(address, endpoint_data_mut_clone, &pool_copy).await {
                    error!("Health check failed: {:?}", e);
                }
            });
            tasks.push(task);
        }
        let mut endpoint_data = endpoint_data_mut.lock().await;
        for (task, rec) in tasks.into_iter().zip(recs.into_iter()) {
            let address = rec.http_address.clone();
            let freq = rec.frequency.clone();
            let endpoint = Endpoint::new(
                address,
                Duration::from_micros(freq.microseconds as u64),
                task,
            );
            endpoint_data.insert(endpoint.url.clone(), endpoint);
        }

        tokio::time::sleep(freq).await;
    }
}

fn init_logger() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                r#"{{"timestamp":"{}","level":"{}","message":"{}","module":"{}","line":{}}}"#,
                chrono::Utc::now().to_rfc3339(),
                record.level(),
                record.args(),
                record.module_path().unwrap_or_default(),
                record.line().unwrap_or(0),
            )
        })
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    init_logger();

    let database_url = env::var("DATABASE_URL")?;
    let pool = Pool::<Postgres>::connect(&database_url).await?;
    let endpoint_data: Arc<Mutex<HashMap<String, Endpoint>>> = Arc::new(Mutex::new(HashMap::new()));

    let freq: u64 = env::var("DB_POLL_FREQUENCY")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("DB_POLL_FREQUENCY must be a valid integer");

    tokio::spawn(async move {
        if let Err(e) =
            poll_for_new_endpoint_data(pool, endpoint_data, Duration::from_secs(freq)).await
        {
            error!("Error polling for new endpoint_data: {:?}", e);
        }
    });
    info!(
        "Started polling for new endpoint_data every {} seconds",
        freq
    );

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
