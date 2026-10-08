use std::io::{self, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use futures::StreamExt;
use tokio::time::sleep;
use uuid::Uuid;

const FTMS_SERVICE: Uuid = Uuid::from_u128(0x00001826_0000_1000_8000_00805F9B34FB);
const HR_SERVICE: Uuid = Uuid::from_u128(0x0000180D_0000_1000_8000_00805F9B34FB);
const BIKE_DATA: Uuid = Uuid::from_u128(0x00002AD2_0000_1000_8000_00805F9B34FB);
const HR_MEASUREMENT: Uuid = Uuid::from_u128(0x00002A37_0000_1000_8000_00805F9B34FB);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Trainer,
    HeartRate,
}

struct Found {
    peripheral: Peripheral,
    name: String,
    kind: Kind,
}

#[tokio::main]
async fn main() -> Result<()> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .context("No Bluetooth adapter found")?;

    println!("Scanning for 6 seconds...");
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(6)).await;
    adapter.stop_scan().await?;

    let mut found: Vec<Found> = Vec::new();
    for p in adapter.peripherals().await? {
        let Some(props) = p.properties().await? else { continue };
        let kind = if props.services.contains(&FTMS_SERVICE) {
            Kind::Trainer
        } else if props.services.contains(&HR_SERVICE) {
            Kind::HeartRate
        } else {
            continue;
        };
        let name = props.local_name.unwrap_or_else(|| "(unnamed)".into());
        found.push(Found { peripheral: p, name, kind });
    }

    if found.is_empty() {
        println!("No trainers or HR monitors found. Is the trainer awake (pedal a bit)?");
        return Ok(());
    }

    println!("\nFound devices:");
    for (i, f) in found.iter().enumerate() {
        println!("  [{}] {:?}: {}", i, f.kind, f.name);
    }

    let trainer = pick(&found, Kind::Trainer, "trainer")?;
    let hr = pick(&found, Kind::HeartRate, "HR monitor")?;

    let mut tasks = Vec::new();
    if let Some(i) = trainer {
        tasks.push(tokio::spawn(stream(found[i].peripheral.clone(), BIKE_DATA, Kind::Trainer)));
    }
    if let Some(i) = hr {
        tasks.push(tokio::spawn(stream(found[i].peripheral.clone(), HR_MEASUREMENT, Kind::HeartRate)));
    }
    for t in tasks {
        t.await??;
    }
    Ok(())
}

fn pick(found: &[Found], kind: Kind, label: &str) -> Result<Option<usize>> {
    print!("Index of {label} (enter to skip): ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let i: usize = line.parse().context("not a number")?;
    anyhow::ensure!(found.get(i).map(|f| f.kind) == Some(kind), "that index is not a {label}");
    Ok(Some(i))
}

async fn stream(p: Peripheral, char_uuid: Uuid, kind: Kind) -> Result<()> {
    p.connect().await?;
    p.discover_services().await?;

    let ch = p
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == char_uuid && c.properties.contains(CharPropFlags::NOTIFY))
        .context("characteristic not found")?;
    p.subscribe(&ch).await?;
    println!("Connected, streaming {:?}...", kind);

    let mut notifications = p.notifications().await?;
    while let Some(n) = notifications.next().await {
        if n.uuid != char_uuid {
            continue;
        }
        match kind {
            Kind::HeartRate => println!("HR raw: {:02X?}", n.value),
            Kind::Trainer => println!("Bike raw: {:02X?}", n.value),
        }
    }
    Ok(())
}