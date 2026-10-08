use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::Sender;

use super::model::Workout;
use crate::bluetooth::control::TrainerCommand;

pub async fn run(workout: Workout, tx: Sender<TrainerCommand>) -> Result<()> {
    let plan = workout.plan();
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    let mut last = None;

    for watts in plan {
        ticker.tick().await;
        if last != Some(watts) {
            tx.send(TrainerCommand::SetTargetPower(watts)).await?;
            last = Some(watts);
        }
    }
    tx.send(TrainerCommand::Stop).await?;
    Ok(())
}