mod db;
mod db_executor;
mod domain;
mod notification_sender;
mod notification_service;

use anyhow::{Ok, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let j = tokio::spawn(notification_service::run_notification_service());
    let _ = j.await;
    Ok(())
}
