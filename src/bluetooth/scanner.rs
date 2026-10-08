use std::time::Duration;

use anyhow::{Context, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use tokio::time::sleep;

use super::device::{DeviceKind, FoundDevice};
use crate::config::uuids::{FTMS_SERVICE, HR_SERVICE};

pub async fn default_adapter() -> Result<Adapter> {
    let manager = Manager::new().await?;
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .context("No Bluetooth adapter found")
}

/// Scan for `duration` and return every trainer / HR monitor seen.
pub async fn scan(adapter: &Adapter, duration: Duration) -> Result<Vec<FoundDevice>> {
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(duration).await;
    adapter.stop_scan().await?;

    let mut found = Vec::new();
    for p in adapter.peripherals().await? {
        let Some(props) = p.properties().await? else { continue };
        let kind = if props.services.contains(&FTMS_SERVICE) {
            DeviceKind::Trainer
        } else if props.services.contains(&HR_SERVICE) {
            DeviceKind::HeartRate
        } else {
            continue;
        };
        let name = props.local_name.unwrap_or_else(|| "(unnamed)".into());
        found.push(FoundDevice { peripheral: p, name, kind });
    }
    Ok(found)
}