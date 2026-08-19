use crate::design::{Error, Result};

const L: u32 = 1 << 16;

/// Order-1 Markov Context rANS Entropy Coder.
/// Conditions symbol probability on the preceding byte P(c_i | c_{i-1}).
pub struct Markov;

impl Markov {
  pub fn new() -> Self {
    Self
  }

  /// Encode byte slice using Order-1 Markov conditioning.
  pub fn encode(&self, input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
      return Ok(Vec::new());
    }

    // Count Order-1 bigram transitions: [prev_byte][curr_byte]
    let mut counts = [[0u16; 256]; 256];
    let mut prev = 0u8;
    for &curr in input {
      counts[prev as usize][curr as usize] += 1;
      prev = curr;
    }

    // Build cumulative normalized probability tables per context
    // Scale to total frequency sum M = 256 (8-bit precision per context)
    let mut cdf = [[0u16; 257]; 256];
    let mut freqs = [[0u16; 256]; 256];

    for c in 0..256 {
      let total: u32 = counts[c].iter().map(|&v| v as u32).sum();
      if total == 0 {
        // Uniform fallback for unused context
        for s in 0..256 {
          freqs[c][s] = 1;
          cdf[c][s + 1] = (s + 1) as u16;
        }
        continue;
      }

      let mut cum = 0u32;
      for s in 0..256 {
        let cnt = counts[c][s] as u32;
        let freq = if cnt > 0 {
          ((cnt * 256 + total / 2) / total).max(1) as u16
        } else {
          0
        };
        freqs[c][s] = freq;
        cum += freq as u32;
      }

      // Adjust rounding to guarantee sum == 256
      if cum != 256 {
        let mut max_s = 0;
        let mut max_f = 0;
        for s in 0..256 {
          if freqs[c][s] > max_f {
            max_f = freqs[c][s];
            max_s = s;
          }
        }
        if cum < 256 {
          freqs[c][max_s] += (256 - cum) as u16;
        } else if freqs[c][max_s] > (cum - 256) as u16 {
          freqs[c][max_s] -= (cum - 256) as u16;
        }
      }

      // Compute prefix sums
      let mut prefix = 0u16;
      for s in 0..256 {
        cdf[c][s] = prefix;
        prefix += freqs[c][s];
      }
      cdf[c][256] = 256;
    }

    // Encode backwards using rANS with context conditioning
    let mut state: u32 = L;
    let mut out = Vec::new();

    for i in (0..input.len()).rev() {
      let curr = input[i];
      let c = if i > 0 { input[i - 1] as usize } else { 0 };
      let freq = freqs[c][curr as usize] as u32;
      let start = cdf[c][curr as usize] as u32;

      if freq == 0 {
        return Err(Error::Codec(format!(
          "zero frequency for symbol {curr} in context {c}"
        )));
      }

      // Renormalize
      let x_max = ((L >> 8) << 16) * freq;
      while state >= x_max {
        out.push((state & 0xFF) as u8);
        state >>= 8;
      }

      state = ((state / freq) << 8) + (state % freq) + start;
    }

    // Write final 4-byte state
    out.extend_from_slice(&state.to_le_bytes());
    out.reverse();

    Ok(out)
  }

  /// Decode Order-1 Markov rANS stream back into original raw bytes.
  pub fn decode(&self, compressed: &[u8], len: usize) -> Result<Vec<u8>> {
    if len == 0 || compressed.is_empty() {
      return Ok(Vec::new());
    }

    if compressed.len() < 4 {
      return Err(Error::Codec("truncated markov rans stream".into()));
    }

    let mut ptr = 0;
    let mut state = u32::from_le_bytes([
      compressed[ptr],
      compressed[ptr + 1],
      compressed[ptr + 2],
      compressed[ptr + 3],
    ]);
    ptr += 4;

    // Build uniform default tables for decoding
    let mut cdf = [[0u16; 257]; 256];
    let mut freqs = [[0u16; 256]; 256];
    for c in 0..256 {
      for s in 0..256 {
        freqs[c][s] = 1;
        cdf[c][s] = s as u16;
      }
      cdf[c][256] = 256;
    }

    let mut output = Vec::with_capacity(len);
    let mut prev = 0usize;

    for _ in 0..len {
      let slot = (state & 0xFF) as u16;

      // Binary search symbol in context prev
      let mut sym = 0u8;
      for s in 0..256 {
        if slot >= cdf[prev][s] && slot < cdf[prev][s + 1] {
          sym = s as u8;
          break;
        }
      }

      output.push(sym);

      let start = cdf[prev][sym as usize] as u32;
      let freq = freqs[prev][sym as usize] as u32;

      state = freq * (state >> 8) + (state & 0xFF) - start;

      while state < L && ptr < compressed.len() {
        state = (state << 8) | (compressed[ptr] as u32);
        ptr += 1;
      }

      prev = sym as usize;
    }

    Ok(output)
  }
}
