// data_collector.rs
use anyhow::Result;
use tokio::sync::mpsc;

use super::pretty_print;
use crate::bluetooth::{device::FoundDevice, stream::stream_device};
use crate::models::snapshot::Snapshot;
use crate::bluetooth::control::TrainerCommand;
use crate::bluetooth::device::DeviceKind;
use crate::models::Reading;
use crate::workout::{model::Workout, runner};

pub async fn run(devices: Vec<FoundDevice>, mut workout: Option<Workout>) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(64);
    let mut pending: Option<(Workout, mpsc::Sender<TrainerCommand>)> = None;

    for device in devices {
        let tx = tx.clone();

        // Only the trainer gets a command channel; the workout waits in `pending`.
        let cmd_rx = match (device.kind, workout.take()) {
            (DeviceKind::Trainer, Some(w)) => {
                let (cmd_tx, cmd_rx) = mpsc::channel(8);
                pending = Some((w, cmd_tx));
                Some(cmd_rx)
            }
            (_, w) => {
                workout = w;
                None
            }
        };

        tokio::spawn(async move {
            if let Err(e) = stream_device(device, tx, cmd_rx).await {
                eprintln!("\ndevice error: {e:#}");
            }
        });
    }
    drop(tx); // so rx closes once every device task ends

    let mut snapshot = Snapshot::default();
    while let Some(reading) = rx.recv().await {
        // First bike data means the trainer is connected and under our control: start the clock.
        if matches!(reading, Reading::Bike(_)) {
            if let Some((w, cmd_tx)) = pending.take() {
                tokio::spawn(async move {
                    if let Err(e) = runner::run(w, cmd_tx).await {
                        eprintln!("\nworkout error: {e:#}");
                    }
                });
            }
        }
        snapshot.apply(reading);
        pretty_print::render(&snapshot);
    }
    println!("\nAll devices disconnected.");
    Ok(())
}