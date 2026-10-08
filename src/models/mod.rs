pub mod bike_data;
pub mod byte_reader;
pub mod heart_rate;
pub mod snapshot;

use bike_data::BikeData;

/// One decoded message from any device.
#[derive(Debug, Clone)]
pub enum Reading {
    Bike(BikeData),
    HeartRate(u16),
}