mod bluetooth;
mod cli;
mod config;
mod models;
mod services;
mod workout;

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

    let workout = if chosen.iter().any(|d| d.kind == bluetooth::device::DeviceKind::Trainer) {
    let workouts = workout::model::load_all()?;
    cli::choose_workout(&workouts)?.and_then(|i| workouts.into_iter().nth(i))
    } else {
        None // no trainer selected, nothing to control
    };

    services::data_collector::run(chosen, workout).await
}