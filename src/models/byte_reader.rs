/// Minimal little-endian reader. Every method returns None if the packet is too short.
pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.data.get(self.pos..self.pos + n)?;
        self.pos += n;
        Some(s)
    }

    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn i16(&mut self) -> Option<i16> {
        let b = self.take(2)?;
        Some(i16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u24(&mut self) -> Option<u32> {
        let b = self.take(3)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], 0]))
    }
}