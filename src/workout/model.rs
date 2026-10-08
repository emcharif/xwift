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

impl Workout {
    /// One target wattage per second of the workout.
    pub fn plan(&self) -> Vec<i16> {
        let mut out = Vec::new();
        expand(&self.blocks, &mut out);
        out
    }
}

fn expand(blocks: &[Block], out: &mut Vec<i16>) {
    for b in blocks {
        match b {
            Block::Steady { duration_s, watts } => {
                out.extend(std::iter::repeat(*watts).take(*duration_s as usize));
            }
            Block::Ramp { duration_s, from_watts, to_watts } => {
                for s in 0..*duration_s {
                    let t = s as f32 / *duration_s as f32;
                    let w = *from_watts as f32 + (*to_watts - *from_watts) as f32 * t;
                    out.push(w.round() as i16);
                }
            }
            Block::Repeat { times, blocks } => {
                for _ in 0..*times {
                    expand(blocks, out);
                }
            }
        }
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