mod db;
mod db_executor;
mod domain;
mod notification_sender;
mod notification_service;
use clap::{arg, command, Parser};
use log::LevelFilter;
use std::io::Write;

use anyhow::{Ok, Result};

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

// / Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Tcp server address to send notifications to
    #[arg(long)]
    notify_tcp: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let args = Args::parse();
    log::info!("Args = {:?}", args);
    let j = tokio::spawn(notification_service::run_notification_service(
        args.notify_tcp,
    ));
    let _ = j.await;
    Ok(())
}
