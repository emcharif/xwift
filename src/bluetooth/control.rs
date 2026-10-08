use anyhow::Result;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

#[derive(Debug, Clone, Copy)]
pub enum TrainerCommand {
    SetTargetPower(i16),
    Stop,
}

const REQUEST_CONTROL: u8 = 0x00;
const SET_TARGET_POWER: u8 = 0x05;
const START: u8 = 0x07;
const STOP_OR_PAUSE: u8 = 0x08;

async fn write(p: &Peripheral, cp: &Characteristic, bytes: &[u8]) -> Result<()> {
    p.write(cp, bytes, WriteType::WithResponse).await?;
    Ok(())
}

pub async fn take_control(p: &Peripheral, cp: &Characteristic) -> Result<()> {
    write(p, cp, &[REQUEST_CONTROL]).await?;
    write(p, cp, &[START]).await
}

pub async fn set_target_power(p: &Peripheral, cp: &Characteristic, watts: i16) -> Result<()> {
    let b = watts.to_le_bytes();
    write(p, cp, &[SET_TARGET_POWER, b[0], b[1]]).await
}

pub async fn stop(p: &Peripheral, cp: &Characteristic) -> Result<()> {
    write(p, cp, &[STOP_OR_PAUSE, 0x01]).await // 0x01 = stop, 0x02 = pause
}