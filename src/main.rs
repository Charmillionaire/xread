mod api;
mod app;
mod crawler;
mod error;
mod model;
mod parser;
mod service;
mod storage;
mod util;

use app::bootstrap::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Server starting...");
    run().await
}
