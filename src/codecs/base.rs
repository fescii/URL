use crate::design::{Error, Result};

const ALPHABET: &[u8; 66] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

/// Bijective base-66 encoder and decoder using RFC 3986 unreserved characters.
pub struct Base;

impl Base {
  pub const fn new() -> Self {
    Self
  }

  /// Encode arbitrary bytes into bijective base-66 string without padding.
  pub fn encode(&self, bytes: &[u8]) -> String {
    if bytes.is_empty() {
      return String::new();
    }

    let mut big = Big::from_bytes_with_sentinel(bytes);
    let mut indices = Vec::new();
    while !big.is_zero() {
      big.sub_one();
      let rem = big.div_rem_u32(66);
      indices.push(ALPHABET[rem as usize] as char);
    }

    indices.reverse();
    indices.into_iter().collect()
  }

  /// Encode bytes into exactly `len` Base66 chars by taking the first `len` chars
  /// of the full encoding (or padding with 'A' if shorter). Used for variable-length shortcuts.
  pub fn encode_fixed(&self, bytes: &[u8], len: usize) -> String {
    let full = self.encode(bytes);
    if full.len() >= len {
      full.chars().take(len).collect()
    } else {
      let mut s = full;
      while s.len() < len {
        s.push('A');
      }
      s
    }
  }

  /// Decode bijective base-66 string back into original bytes.
  pub fn decode(&self, text: &str) -> Result<Vec<u8>> {
    if text.is_empty() {
      return Ok(Vec::new());
    }

    let mut big = Big::zero();
    for ch in text.chars() {
      let index = char_to_index(ch)
        .ok_or_else(|| Error::Codec(format!("invalid base-66 character '{}'", ch)))?;
      big.mul_u32(66);
      big.add_u32((index + 1) as u32);
    }

    big.to_bytes_strip_sentinel()
  }
}

fn char_to_index(c: char) -> Option<usize> {
  ALPHABET.iter().position(|&b| b == c as u8)
}

/// Simple BigUint representation for exact arbitrary-length conversions.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Big {
  limbs: Vec<u32>,
}

impl Big {
  fn zero() -> Self {
    Self { limbs: Vec::new() }
  }

  fn from_bytes_with_sentinel(bytes: &[u8]) -> Self {
    let mut big = Self::zero();
    big.add_u32(1);
    for &b in bytes {
      big.mul_u32(256);
      big.add_u32(b as u32);
    }
    big
  }

  fn is_zero(&self) -> bool {
    self.limbs.is_empty() || self.limbs.iter().all(|&limb| limb == 0)
  }

  fn normalize(&mut self) {
    while let Some(&0) = self.limbs.last() {
      self.limbs.pop();
    }
  }

  fn add_u32(&mut self, val: u32) {
    if val == 0 {
      return;
    }
    let mut carry = val as u64;
    for limb in &mut self.limbs {
      let sum = (*limb as u64) + carry;
      *limb = sum as u32;
      carry = sum >> 32;
      if carry == 0 {
        break;
      }
    }
    if carry > 0 {
      self.limbs.push(carry as u32);
    }
  }

  fn sub_one(&mut self) {
    for limb in &mut self.limbs {
      if *limb > 0 {
        *limb -= 1;
        break;
      } else {
        *limb = u32::MAX;
      }
    }
    self.normalize();
  }

  fn mul_u32(&mut self, factor: u32) {
    if factor == 0 {
      self.limbs.clear();
      return;
    }
    let mut carry = 0u64;
    for limb in &mut self.limbs {
      let prod = (*limb as u64) * (factor as u64) + carry;
      *limb = prod as u32;
      carry = prod >> 32;
    }
    if carry > 0 {
      self.limbs.push(carry as u32);
    }
  }

  fn div_rem_u32(&mut self, divisor: u32) -> u32 {
    let mut rem = 0u64;
    for limb in self.limbs.iter_mut().rev() {
      let cur = (rem << 32) | (*limb as u64);
      *limb = (cur / (divisor as u64)) as u32;
      rem = cur % (divisor as u64);
    }
    self.normalize();
    rem as u32
  }

  fn to_bytes_strip_sentinel(&mut self) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    while !self.is_zero() {
      let rem = self.div_rem_u32(256);
      bytes.push(rem as u8);
    }
    match bytes.pop() {
      Some(1) => {
        bytes.reverse();
        Ok(bytes)
      }
      Some(other) => Err(Error::Codec(format!(
        "invalid sentinel byte in decoded stream: {other}"
      ))),
      None => Err(Error::Codec(
        "empty decoded stream missing sentinel".to_string(),
      )),
    }
  }
}
