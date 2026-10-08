/// Heart Rate Measurement (0x2A37). Bit 0 of the flags says whether the value is u8 or u16.
pub fn parse(bytes: &[u8]) -> Option<u16> {
    let flags = *bytes.first()?;
    if flags & 1 == 0 {
        bytes.get(1).map(|&b| b as u16)
    } else {
        Some(u16::from_le_bytes([*bytes.get(1)?, *bytes.get(2)?]))
    }
}