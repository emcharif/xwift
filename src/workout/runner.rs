use std::time::Duration;

use anyhow::Result;
use tokio::sync::{mpsc::Sender, watch};

use super::model::Workout;
use crate::bluetooth::control::TrainerCommand;

#[derive(Debug, Clone, Default)]
pub struct WorkoutStatus {
    pub elapsed_s: u32,
    pub total_s: u32,
    pub target_watts: i16,
    pub segment: usize,
    pub segment_total: usize,
    pub segment_remaining_s: u32,
    pub label: String,
    pub finished: bool,
}

fn mmss(s: u32) -> String {
    format!("{:02}:{:02}", s / 60, s % 60)
}

impl WorkoutStatus {
    pub fn summary(&self) -> String {
        if self.finished {
            return format!("Workout finished ({})", mmss(self.total_s));
        }
        format!(
            "Workout  {} / {}  (left {})\nTarget   {} W\nBlock    {}/{}: {} ({} s left)",
            mmss(self.elapsed_s),
            mmss(self.total_s),
            mmss(self.total_s - self.elapsed_s),
            self.target_watts,
            self.segment,
            self.segment_total,
            self.label,
            self.segment_remaining_s,
        )
    }
}

pub async fn run(
    workout: Workout,
    tx: Sender<TrainerCommand>,
    status_tx: watch::Sender<Option<WorkoutStatus>>,
) -> Result<()> {
    let plan = workout.plan();
    let total_s = plan.len() as u32;
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    let mut last = None;

    for (i, step) in plan.iter().enumerate() {
        ticker.tick().await;
        if last != Some(step.watts) {
            tx.send(TrainerCommand::SetTargetPower(step.watts)).await?;
            last = Some(step.watts);
        }
        let _ = status_tx.send(Some(WorkoutStatus {
            elapsed_s: i as u32,
            total_s,
            target_watts: step.watts,
            segment: step.segment,
            segment_total: step.segment_total,
            segment_remaining_s: step.segment_remaining_s,
            label: step.label.clone(),
            finished: false,
        }));
    }

    tx.send(TrainerCommand::Stop).await?;
    status_tx.send_modify(|s| {
        if let Some(s) = s {
            s.finished = true;
            s.elapsed_s = total_s;
        }
    });
    Ok(())
}