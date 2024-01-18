use chrono::Utc;
mod db;
mod db_executor;
mod domain;
mod lib;
mod notification_sender;

use anyhow::{Ok, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let j = tokio::spawn(async {
        lib::run_notification_service().await;
    });
    let _ = j.await;
    Ok(())
}
