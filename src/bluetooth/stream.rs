// stream.rs
use anyhow::{Context, Result};
use btleplug::api::{CharPropFlags, Peripheral as _};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use super::device::{DeviceKind, FoundDevice};
use crate::config::uuids::{HR_MEASUREMENT, INDOOR_BIKE_DATA};
use crate::models::{bike_data::BikeData, heart_rate, Reading};

use tokio::sync::mpsc::Receiver;
use super::control::{self, TrainerCommand};
use crate::config::uuids::FITNESS_MACHINE_CONTROL_POINT;

/// Connect to a device, subscribe to its data characteristic, and push
/// parsed readings into `tx` until the connection drops.
pub async fn stream_device(
    device: FoundDevice,
    tx: Sender<Reading>,
    mut cmd_rx: Option<Receiver<TrainerCommand>>,
) -> Result<()> {
    let FoundDevice { peripheral: p, name, kind } = device;
    let char_uuid = match kind {
        DeviceKind::Trainer => INDOOR_BIKE_DATA,
        DeviceKind::HeartRate => HR_MEASUREMENT,
    };

    p.connect().await.with_context(|| format!("connecting to {name}"))?;
    p.discover_services().await?;

    let ch = p
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == char_uuid && c.properties.contains(CharPropFlags::NOTIFY))
        .with_context(|| format!("{name}: data characteristic not found"))?;
    p.subscribe(&ch).await?;

    // Trainer only: grab the control point and take control.
    let control_ch = if kind == DeviceKind::Trainer && cmd_rx.is_some() {
        let cp = p
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == FITNESS_MACHINE_CONTROL_POINT)
            .with_context(|| format!("{name}: control point not found"))?;
        p.subscribe(&cp).await?; // responses arrive as indications
        control::take_control(&p, &cp).await?;
        Some(cp)
    } else {
        None
    };

    let mut notifications = p.notifications().await?;
    loop {
        tokio::select! {
            n = notifications.next() => {
                let Some(n) = n else { break };
                if n.uuid == FITNESS_MACHINE_CONTROL_POINT {
                    // [0x80, request opcode, result]; result 0x01 = success
                    if n.value.len() >= 3 && n.value[2] != 0x01 {
                        eprintln!("trainer rejected opcode {:#04X}: result {:#04X}", n.value[1], n.value[2]);
                    }
                    continue;
                }
                if n.uuid != char_uuid {
                    continue;
                }
                let reading = match kind {
                    DeviceKind::Trainer => BikeData::parse(&n.value).map(Reading::Bike),
                    DeviceKind::HeartRate => heart_rate::parse(&n.value).map(Reading::HeartRate),
                };
                if let Some(r) = reading {
                    if tx.send(r).await.is_err() {
                        break;
                    }
                }
            }
            cmd = next_cmd(&mut cmd_rx) => {
                let (Some(cmd), Some(cp)) = (cmd, control_ch.as_ref()) else { break };
                match cmd {
                    TrainerCommand::SetTargetPower(w) => control::set_target_power(&p, cp, w).await?,
                    TrainerCommand::Stop => control::stop(&p, cp).await?,
                }
            }
        }
    }
    Ok(())
}

/// Waits for a command, or forever if there is no command channel (HR monitor).
async fn next_cmd(rx: &mut Option<Receiver<TrainerCommand>>) -> Option<TrainerCommand> {
    match rx {
        Some(r) => r.recv().await,
        None => std::future::pending().await,
    }
}