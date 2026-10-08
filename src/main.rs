mod bluetooth;
mod cli;
mod config;
mod models;
mod services;

use std::time::Duration;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let adapter = bluetooth::scanner::default_adapter().await?;

    println!("Scanning for {} seconds...", config::SCAN_SECONDS);
    let found = bluetooth::scanner::scan(&adapter, Duration::from_secs(config::SCAN_SECONDS)).await?;

    if found.is_empty() {
        println!("No trainers or HR monitors found. Is the trainer awake (pedal a bit)?");
        return Ok(());
    }

    cli::print_devices(&found);
    let chosen = cli::choose_devices(found)?;
    if chosen.is_empty() {
        println!("Nothing selected.");
        return Ok(());
    }

    services::data_collector::run(chosen).await
}