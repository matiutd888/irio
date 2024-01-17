use scylla::Session;
mod lib;
mod utils;
mod domain;
mod db;

pub use crate::domain::*;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let x: domain::AdminId = 10;
    println!("Hello scylla!");
    Ok(())
}