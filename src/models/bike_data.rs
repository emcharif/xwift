use super::byte_reader::ByteReader;

/// FTMS "Indoor Bike Data" (characteristic 0x2AD2).
/// Every field is optional because the trainer only sends what its flags announce.
#[derive(Debug, Clone, Default)]
pub struct BikeData {
    pub speed_kmh: Option<f32>,
    pub avg_speed_kmh: Option<f32>,
    pub cadence_rpm: Option<f32>,
    pub avg_cadence_rpm: Option<f32>,
    pub distance_m: Option<u32>,
    pub resistance: Option<i16>,
    pub power_w: Option<i16>,
    pub avg_power_w: Option<i16>,
    pub energy_kcal: Option<u16>,
    pub heart_rate_bpm: Option<u8>,
    pub elapsed_s: Option<u16>,
}

impl BikeData {
    /// Fields appear in a fixed order, each guarded by one flag bit.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let mut r = ByteReader::new(bytes);
        let flags = r.u16()?;
        let has = |bit: u16| flags & (1 << bit) != 0;
        let mut d = Self::default();

        // Bit 0 is inverted: 0 means speed IS present.
        if !has(0) {
            d.speed_kmh = Some(r.u16()? as f32 * 0.01);
        }
        if has(1) {
            d.avg_speed_kmh = Some(r.u16()? as f32 * 0.01);
        }
        if has(2) {
            d.cadence_rpm = Some(r.u16()? as f32 * 0.5);
        }
        if has(3) {
            d.avg_cadence_rpm = Some(r.u16()? as f32 * 0.5);
        }
        if has(4) {
            d.distance_m = Some(r.u24()?);
        }
        if has(5) {
            d.resistance = Some(r.i16()?);
        }
        if has(6) {
            d.power_w = Some(r.i16()?);
        }
        if has(7) {
            d.avg_power_w = Some(r.i16()?);
        }
        if has(8) {
            d.energy_kcal = Some(r.u16()?); // total energy
            r.u16()?; // energy per hour (skipped)
            r.u8()?; // energy per minute (skipped)
        }
        if has(9) {
            d.heart_rate_bpm = Some(r.u8()?);
        }
        if has(10) {
            r.u8()?; // metabolic equivalent (skipped)
        }
        if has(11) {
            d.elapsed_s = Some(r.u16()?);
        }
        Some(d)
    }
}