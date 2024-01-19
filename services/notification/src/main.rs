mod db;
mod db_executor;
mod domain;
mod notification_sender;
mod notification_service;
use std::io::Write;

use log::LevelFilter;

use anyhow::{Ok, Result};

#[tokio::main]
async fn main() -> Result<()> {
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

    let j = tokio::spawn(notification_service::run_notification_service());
    let _ = j.await;
    Ok(())
}
