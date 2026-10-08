use anyhow::{Context, Result};
use btleplug::api::{CharPropFlags, Peripheral as _};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use super::device::{DeviceKind, FoundDevice};
use crate::config::uuids::{HR_MEASUREMENT, INDOOR_BIKE_DATA};
use crate::models::{bike_data::BikeData, heart_rate, Reading};

/// Connect to a device, subscribe to its data characteristic, and push
/// parsed readings into `tx` until the connection drops.
pub async fn stream_device(device: FoundDevice, tx: Sender<Reading>) -> Result<()> {
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

    let mut notifications = p.notifications().await?;
    while let Some(n) = notifications.next().await {
        if n.uuid != char_uuid {
            continue;
        }
        let reading = match kind {
            DeviceKind::Trainer => BikeData::parse(&n.value).map(Reading::Bike),
            DeviceKind::HeartRate => heart_rate::parse(&n.value).map(Reading::HeartRate),
        };
        if let Some(r) = reading {
            if tx.send(r).await.is_err() {
                break; // receiver gone, we're shutting down
            }
        }
    }
    Ok(())
}