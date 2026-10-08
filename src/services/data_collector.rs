use anyhow::Result;
use tokio::sync::mpsc;

use super::pretty_print;
use crate::bluetooth::{device::FoundDevice, stream::stream_device};
use crate::models::snapshot::Snapshot;

/// Start one streaming task per device, fold their readings into a Snapshot,
/// and redraw after every update. Returns when all devices disconnect.
pub async fn run(devices: Vec<FoundDevice>) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(64);

    for device in devices {
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = stream_device(device, tx).await {
                eprintln!("\ndevice error: {e:#}");
            }
        });
    }
    drop(tx); // so rx closes once every device task ends

    let mut snapshot = Snapshot::default();
    while let Some(reading) = rx.recv().await {
        snapshot.apply(reading);
        pretty_print::render(&snapshot);
    }
    println!("\nAll devices disconnected.");
    Ok(())
}