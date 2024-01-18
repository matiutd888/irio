use chrono::Utc;
mod lib;
mod utils;
mod domain;
mod db;
mod db_executor;
mod notification_sender;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let x = Utc::now();
    println!("Hello postgres! {}", x.timestamp_millis());
    Ok(())
}