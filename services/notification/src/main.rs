use scylla::Session;
mod lib;
mod utils;
mod domain;
mod db;

#[tokio::main]
async fn main() {
    println!("Hello scylla!");
}