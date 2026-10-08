use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

pub const WORKOUT_DIR: &str = "workouts";

#[derive(Debug, Deserialize)]
pub struct Workout {
    pub name: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Steady { duration_s: u32, watts: i16 },
    Ramp { duration_s: u32, from_watts: i16, to_watts: i16 },
    Repeat { times: u32, blocks: Vec<Block> },
}
#[derive(Debug, Clone)]
pub struct Step {
    pub watts: i16,
    pub segment: usize,
    pub segment_total: usize,
    pub segment_remaining_s: u32,
    pub label: String,
}

impl Workout {
    /// One step per second of the workout.
    pub fn plan(&self) -> Vec<Step> {
        let mut out = Vec::new();
        let mut seg = 0;
        expand(&self.blocks, "", &mut seg, &mut out);
        for s in &mut out {
            s.segment_total = seg;
        }
        out
    }
}

fn expand(blocks: &[Block], prefix: &str, seg: &mut usize, out: &mut Vec<Step>) {
    for b in blocks {
        match b {
            Block::Steady { duration_s, watts } => {
                *seg += 1;
                push_segment(out, *seg, format!("{prefix}Steady {watts} W"), *duration_s, |_| *watts);
            }
            Block::Ramp { duration_s, from_watts, to_watts } => {
                *seg += 1;
                let label = format!("{prefix}Ramp {from_watts} -> {to_watts} W");
                push_segment(out, *seg, label, *duration_s, |s| {
                    let t = s as f32 / *duration_s as f32;
                    (*from_watts as f32 + (*to_watts - *from_watts) as f32 * t).round() as i16
                });
            }
            Block::Repeat { times, blocks } => {
                for n in 1..=*times {
                    let p = format!("{prefix}Rep {n}/{times}: ");
                    expand(blocks, &p, seg, out);
                }
            }
        }
    }
}

fn push_segment(out: &mut Vec<Step>, seg: usize, label: String, duration_s: u32, watts_at: impl Fn(u32) -> i16) {
    for s in 0..duration_s {
        out.push(Step {
            watts: watts_at(s),
            segment: seg,
            segment_total: 0, // filled in by plan()
            segment_remaining_s: duration_s - s,
            label: label.clone(),
        });
    }
}

/// Load every *.json in the workouts folder, sorted by file name.
pub fn load_all() -> Result<Vec<Workout>> {
    let mut paths: Vec<_> = fs::read_dir(WORKOUT_DIR)
        .with_context(|| format!("no '{WORKOUT_DIR}' folder next to where you run xwift"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths.iter().map(|p| load(p)).collect()
}

fn load(path: &Path) -> Result<Workout> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}