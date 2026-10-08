// pretty_print.rs
use std::io::{self, Write};

use crate::models::snapshot::Snapshot;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::workout::runner::WorkoutStatus;

static DRAWN_LINES: AtomicUsize = AtomicUsize::new(0);

const POWER_BAR_WIDTH: usize = 20;
const POWER_BAR_MAX_W: f32 = 400.0;
/// Redraw the status block in place (bike line + optional workout lines).
pub fn render(s: &Snapshot, workout: Option<&WorkoutStatus>) {
    let bike = s.bike.as_ref();

    let speed = fmt(bike.and_then(|b| b.speed_kmh), |v| format!("{v:5.1}"));
    let cadence = fmt(bike.and_then(|b| b.cadence_rpm), |v| format!("{v:3.0}"));
    let power_w = bike.and_then(|b| b.power_w);
    let power = fmt(power_w, |v| format!("{v:4}"));
    let bar = power_w.map(power_bar).unwrap_or_default();
    let avg = fmt(s.power_avg_w(), |v| format!("{v:4}"));

    // Prefer the dedicated HR strap, fall back to HR reported by the trainer.
    let hr = s.heart_rate.or(bike.and_then(|b| b.heart_rate_bpm.map(u16::from)));
    let hr = fmt(hr, |v| format!("{v:3}"));

    let mut line = format!("🚴 {speed} km/h │ {cadence} rpm │ {power} W ({avg} avg) {bar} │ ❤  {hr} bpm");

    if let Some(d) = bike.and_then(|b| b.distance_m) {
        line.push_str(&format!(" │ {:.2} km", d as f32 / 1000.0));
    }
    if let Some(t) = bike.and_then(|b| b.elapsed_s) {
        line.push_str(&format!(" │ {:02}:{:02}", t / 60, t % 60));
    }

    let mut lines = vec![line];
    if let Some(w) = workout {
        lines.extend(w.summary().lines().map(String::from));
    }

    // Go back to the top of the previous block, clear it, and draw again.
    let prev = DRAWN_LINES.swap(lines.len(), Ordering::Relaxed);
    let mut out = String::from("\r");
    if prev > 1 {
        out.push_str(&format!("\x1b[{}A", prev - 1));
    }
    out.push_str("\x1b[J"); // clear from cursor to end of screen
    out.push_str(&lines.join("\n"));

    print!("{out}");
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