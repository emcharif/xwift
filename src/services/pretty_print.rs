use std::io::{self, Write};

use crate::models::snapshot::Snapshot;

const POWER_BAR_WIDTH: usize = 20;
const POWER_BAR_MAX_W: f32 = 400.0;

/// Redraw a single status line in place.
pub fn render(s: &Snapshot) {
    let bike = s.bike.as_ref();

    let speed = fmt(bike.and_then(|b| b.speed_kmh), |v| format!("{v:5.1}"));
    let cadence = fmt(bike.and_then(|b| b.cadence_rpm), |v| format!("{v:3.0}"));
    let power_w = bike.and_then(|b| b.power_w);
    let power = fmt(power_w, |v| format!("{v:4}"));
    let bar = power_w.map(power_bar).unwrap_or_default();

    // Prefer the dedicated HR strap, fall back to HR reported by the trainer.
    let hr = s.heart_rate.or(bike.and_then(|b| b.heart_rate_bpm.map(u16::from)));
    let hr = fmt(hr, |v| format!("{v:3}"));

    let mut line = format!("🚴 {speed} km/h │ {cadence} rpm │ {power} W {bar} │ ❤  {hr} bpm");

    if let Some(d) = bike.and_then(|b| b.distance_m) {
        line.push_str(&format!(" │ {:.2} km", d as f32 / 1000.0));
    }
    if let Some(t) = bike.and_then(|b| b.elapsed_s) {
        line.push_str(&format!(" │ {:02}:{:02}", t / 60, t % 60));
    }

    // \r = back to line start, \x1b[2K = clear the line
    print!("\r\x1b[2K{line}");
    let _ = io::stdout().flush();
}

fn fmt<T>(v: Option<T>, f: impl Fn(T) -> String) -> String {
    v.map(f).unwrap_or_else(|| " --".into())
}

fn power_bar(watts: i16) -> String {
    let frac = (watts.max(0) as f32 / POWER_BAR_MAX_W).min(1.0);
    let filled = (frac * POWER_BAR_WIDTH as f32).round() as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(POWER_BAR_WIDTH - filled))
}