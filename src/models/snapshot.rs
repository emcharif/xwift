use super::{bike_data::BikeData, Reading};

/// The latest known state across all connected devices.
#[derive(Debug, Default)]
pub struct Snapshot {
    pub bike: Option<BikeData>,
    pub heart_rate: Option<u16>,
}

impl Snapshot {
    pub fn apply(&mut self, reading: Reading) {
        match reading {
            Reading::Bike(b) => self.bike = Some(b),
            Reading::HeartRate(h) => self.heart_rate = Some(h),
        }
    }
}