use super::{Reading, bike_data::BikeData};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const POWER_WINDOW: Duration = Duration::from_secs(3);

#[derive(Debug, Default)]
pub struct Snapshot {
    pub bike: Option<BikeData>,
    pub heart_rate: Option<u16>,
    power_history: VecDeque<(Instant, i16)>,
}

impl Snapshot {
    /// Average power over the last 3 seconds.
    pub fn power_avg_w(&self) -> Option<i16> {
        if self.power_history.is_empty() {
            return None;
        }
        let sum: i32 = self.power_history.iter().map(|(_, w)| *w as i32).sum();
        Some((sum / self.power_history.len() as i32) as i16)
    }

    pub fn apply(&mut self, reading: Reading) {
        match reading {
            Reading::Bike(b) => {
                if let Some(w) = b.power_w {
                    let now = Instant::now();
                    self.power_history.push_back((now, w));
                    while self
                        .power_history
                        .front()
                        .is_some_and(|(t, _)| now.duration_since(*t) > POWER_WINDOW)
                    {
                        self.power_history.pop_front();
                    }
                }
                self.bike = Some(b);
            }
            Reading::HeartRate(h) => self.heart_rate = Some(h),
        }
    }

    /// Speed from the 3 s average power, so it doesn't jitter.
    pub fn virtual_speed_kmh(&self) -> Option<f32> {
        self.power_avg_w().map(|w| super::speed::speed_kmh(w as f64) as f32)
    }
}