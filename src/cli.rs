use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::bluetooth::device::{DeviceKind, FoundDevice};

pub fn print_devices(found: &[FoundDevice]) {
    println!("\nFound devices:");
    for (i, d) in found.iter().enumerate() {
        println!("  [{}] {:?}: {}", i, d.kind, d.name);
    }
}

/// Ask the user to pick one trainer and/or one HR monitor.
/// Only asks about kinds that were actually found.
pub fn choose_devices(found: Vec<FoundDevice>) -> Result<Vec<FoundDevice>> {
    let mut picks = Vec::new();
    for (kind, label) in [(DeviceKind::Trainer, "trainer"), (DeviceKind::HeartRate, "HR monitor")] {
        if found.iter().any(|d| d.kind == kind) {
            if let Some(i) = prompt_index(&found, kind, label)? {
                picks.push(i);
            }
        }
    }
    Ok(found
        .into_iter()
        .enumerate()
        .filter(|(i, _)| picks.contains(i))
        .map(|(_, d)| d)
        .collect())
}

fn prompt_index(found: &[FoundDevice], kind: DeviceKind, label: &str) -> Result<Option<usize>> {
    print!("Index of {label} (enter to skip): ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let i: usize = line.parse().context("not a number")?;
    anyhow::ensure!(found.get(i).map(|d| d.kind) == Some(kind), "that index is not a {label}");
    Ok(Some(i))
}