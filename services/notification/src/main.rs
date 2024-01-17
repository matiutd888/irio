use chrono::Utc;
use scylla::Session;
mod lib;
mod utils;
mod domain;
mod db;
mod db_executor;

pub use crate::domain::*;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let x = Utc::now();
    println!("Hello scylla! {}", x.timestamp_millis());
    Ok(())
}