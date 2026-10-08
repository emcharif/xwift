//device.rs
use btleplug::platform::Peripheral;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceKind {
    Trainer,
    HeartRate,
}

pub struct FoundDevice {
    pub peripheral: Peripheral,
    pub name: String,
    pub kind: DeviceKind,
}