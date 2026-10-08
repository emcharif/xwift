use anyhow::Result;
use tokio::sync::{mpsc, watch};

use super::pretty_print;
use crate::bluetooth::control::TrainerCommand;
use crate::bluetooth::device::{DeviceKind, FoundDevice};
use crate::bluetooth::stream::stream_device;
use crate::models::snapshot::Snapshot;
use crate::models::Reading;
use crate::workout::model::Workout;
use crate::workout::runner::{self, WorkoutStatus};
use std::time::Duration;
use chrono::Utc;
use super::export::{self, Sample};

/// Start one streaming task per device, fold their readings into a Snapshot,
/// and redraw after every update. Returns when all devices disconnect.
pub async fn run(devices: Vec<FoundDevice>, mut workout: Option<Workout>) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(64);
    let mut pending: Option<(Workout, mpsc::Sender<TrainerCommand>)> = None;
    let (status_tx, status_rx) = watch::channel::<Option<WorkoutStatus>>(None);
    let ride_name = workout.as_ref().map(|w| w.name.clone()).unwrap_or_else(|| "Free ride".into());

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
    let mut samples: Vec<Sample> = Vec::new();
    let start = Utc::now();
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            r = rx.recv() => {
                let Some(reading) = r else { break };
                if matches!(reading, Reading::Bike(_)) {
                    if let Some((w, cmd_tx)) = pending.take() {
                        let status_tx = status_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = runner::run(w, cmd_tx, status_tx).await {
                                eprintln!("\nworkout error: {e:#}");
                            }
                        });
                    }
                }
                snapshot.apply(reading);
                pretty_print::render(&snapshot, status_rx.borrow().as_ref());
            }
            _ = ticker.tick() => {
                let b = snapshot.bike.as_ref();
                samples.push(Sample {
                    power_w: b.and_then(|b| b.power_w).map(|v| v as i32),
                    cadence_rpm: b.and_then(|b| b.cadence_rpm).map(|v| (v as f64).round() as u32),
                    hr_bpm: snapshot.heart_rate.or(b.and_then(|b| b.heart_rate_bpm.map(u16::from))),
                    speed_kmh: b.and_then(|b| b.speed_kmh).map(|v| v as f64),
                });
                if status_rx.borrow().as_ref().is_some_and(|s| s.finished) {
                    break; // workout done, even if the HR strap is still connected
                }
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    pretty_print::render(&snapshot, status_rx.borrow().as_ref());
    if samples.len() > 10 {
        let path = export::write_tcx(&ride_name, start, &samples)?;
        println!("\nSaved {path}");
    }
    println!("All devices disconnected.");
    Ok(())
}