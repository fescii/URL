use crate::design::{Error, Result};

/// Bit-level stream writer for packing variable-width bit sequences.
pub struct Writer {
  bytes: Vec<u8>,
  byte: u8,
  bits: u8,
}

impl Writer {
  pub const fn new() -> Self {
    Self {
      bytes: Vec::new(),
      byte: 0,
      bits: 0,
    }
  }

  /// Write `count` bits (up to 32 bits) from `val` into the bitstream.
  pub fn write(&mut self, val: u32, count: u8) {
    assert!(count <= 32);
    for i in (0..count).rev() {
      let bit = ((val >> i) & 1) as u8;
      self.byte = (self.byte << 1) | bit;
      self.bits += 1;
      if self.bits == 8 {
        self.bytes.push(self.byte);
        self.byte = 0;
        self.bits = 0;
      }
    }
  }

  /// Write a full byte slice directly into the bitstream.
  pub fn write_bytes(&mut self, data: &[u8]) {
    for &b in data {
      self.write(b as u32, 8);
    }
  }

  /// Flush remaining bits padded with zeros and return the packed byte buffer.
  pub fn finish(mut self) -> Vec<u8> {
    if self.bits > 0 {
      self.byte <<= 8 - self.bits;
      self.bytes.push(self.byte);
    }
    self.bytes
  }
}

/// Bit-level stream reader for unpacking variable-width bit sequences.
pub struct Reader<'a> {
  bytes: &'a [u8],
  offset: usize,
  bits: u8,
}

impl<'a> Reader<'a> {
  pub const fn new(bytes: &'a [u8]) -> Self {
    Self {
      bytes,
      offset: 0,
      bits: 0,
    }
  }

  /// Read `count` bits (up to 32 bits) from the bitstream.
  pub fn read(&mut self, count: u8) -> Result<u32> {
    assert!(count <= 32);
    let mut val = 0u32;
    for _ in 0..count {
      let byte_idx = self.offset / 8;
      if byte_idx >= self.bytes.len() {
        return Err(Error::Codec("unexpected end of bitstream".to_string()));
      }
      let bit_idx = 7 - (self.offset % 8);
      let bit = (self.bytes[byte_idx] >> bit_idx) & 1;
      val = (val << 1) | (bit as u32);
      self.offset += 1;
    }
    Ok(val)
  }

  /// Read full byte slice.
  pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
      result.push(self.read(8)? as u8);
    }
    Ok(result)
  }

  /// Remaining bits available to read.
  pub fn remaining(&self) -> usize {
    let total = self.bytes.len() * 8;
    if self.offset < total {
      total - self.offset
    } else {
      0
    }
  }
}
