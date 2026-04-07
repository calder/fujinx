use anyhow::{Result, ensure};

/// Reader for PTP dataset fields (little-endian, PTP string format).
pub(super) struct DatasetReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> DatasetReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        ensure!(self.remaining() >= n, "unexpected end of dataset");
        self.pos += n;

        Ok(())
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        ensure!(self.remaining() >= 2, "unexpected end of dataset");
        let v = u16::from_le_bytes(self.data[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;

        Ok(v)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        ensure!(self.remaining() >= 4, "unexpected end of dataset");
        let v = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;

        Ok(v)
    }

    /// Read a PTP string: u8 length (in chars including null), then UTF-16LE chars.
    pub fn read_ptp_string(&mut self) -> Result<String> {
        ensure!(self.remaining() >= 1, "unexpected end of dataset");
        let num_chars = self.data[self.pos] as usize;
        self.pos += 1;
        if num_chars == 0 {
            return Ok(String::new());
        }
        let byte_len = num_chars * 2;
        ensure!(
            self.remaining() >= byte_len,
            "string extends past end of dataset"
        );
        let u16s: Vec<u16> = (0..num_chars)
            .map(|i| {
                let off = self.pos + i * 2;
                u16::from_le_bytes([self.data[off], self.data[off + 1]])
            })
            .collect();
        self.pos += byte_len;

        let end = u16s.iter().position(|&c| c == 0).unwrap_or(u16s.len());

        Ok(String::from_utf16_lossy(&u16s[..end]))
    }

    /// Read a PTP u16 array: u32 count, then count u16 values.
    pub fn read_u16_array(&mut self) -> Result<Vec<u16>> {
        let count = self.read_u32()? as usize;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_u16()?);
        }

        Ok(values)
    }

    /// Skip a PTP u16 array: u32 count, then count * 2 bytes.
    pub fn skip_u16_array(&mut self) -> Result<()> {
        let count = self.read_u32()? as usize;

        self.skip(count * 2)
    }
}

/// Writer for PTP dataset fields (little-endian, PTP string format).
pub(super) struct DatasetWriter {
    buf: Vec<u8>,
}

impl DatasetWriter {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(128),
        }
    }

    pub fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Write a PTP string: u8 length (in chars including null), then UTF-16LE chars.
    pub fn write_ptp_string(&mut self, s: &str) {
        if s.is_empty() {
            self.buf.push(0);
            return;
        }
        let u16s: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
        self.buf.push(u16s.len() as u8);
        for c in &u16s {
            self.buf.extend_from_slice(&c.to_le_bytes());
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}
