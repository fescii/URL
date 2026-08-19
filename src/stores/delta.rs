use crate::design::{Error, Result};

/// Record containing bitmask of matched positions (up to 256 bytes) and differing bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
  pub anchor: u8,
  pub target_len: u16,
  pub mask: [u64; 4],
  pub diffs: Vec<u8>,
}

/// Myers Bit-Parallel Positional Delta Engine for character and substring deduplication.
#[derive(Debug, Default, Clone, Copy)]
pub struct Delta;

impl Delta {
  pub const fn new() -> Self {
    Self
  }

  /// Compute positional difference between target and anchor.
  /// Matched byte positions (up to 256 bytes) take 0 bytes of storage in diffs.
  pub fn diff(&self, anchor_idx: u8, anchor: &[u8], target: &[u8]) -> Record {
    let mut mask = [0u64; 4];
    let mut diffs = Vec::with_capacity(target.len());

    let min_len = anchor.len().min(target.len());

    for i in 0..min_len {
      if i < 256 && anchor[i] == target[i] {
        let word = i / 64;
        let bit = i % 64;
        mask[word] |= 1u64 << bit;
      } else {
        diffs.push(target[i]);
      }
    }

    // Tail bytes beyond anchor length
    if target.len() > min_len {
      for i in min_len..target.len() {
        diffs.push(target[i]);
      }
    }

    Record {
      anchor: anchor_idx,
      target_len: target.len() as u16,
      mask,
      diffs,
    }
  }

  /// Reconstruct original target bytes from anchor and delta record in O(N) bitwise operations.
  pub fn apply(&self, anchor: &[u8], record: &Record) -> Result<Vec<u8>> {
    let len = record.target_len as usize;
    let mut output = Vec::with_capacity(len);
    let mut diff_ptr = 0;

    let min_len = anchor.len().min(len);

    for i in 0..min_len {
      let is_match = if i < 256 {
        let word = i / 64;
        let bit = i % 64;
        (record.mask[word] & (1u64 << bit)) != 0
      } else {
        false
      };

      if is_match {
        output.push(anchor[i]);
      } else {
        if diff_ptr >= record.diffs.len() {
          return Err(Error::Codec("truncated delta diff stream".into()));
        }
        output.push(record.diffs[diff_ptr]);
        diff_ptr += 1;
      }
    }

    if len > min_len {
      while diff_ptr < record.diffs.len() && output.len() < len {
        output.push(record.diffs[diff_ptr]);
        diff_ptr += 1;
      }
    }

    if output.len() != len {
      return Err(Error::Codec(format!(
        "delta reconstruction length mismatch: expected {} got {}",
        len,
        output.len()
      )));
    }

    Ok(output)
  }

  /// Measure similarity ratio (0.0 to 1.0) between target and anchor.
  pub fn similarity(&self, anchor: &[u8], target: &[u8]) -> f32 {
    if anchor.is_empty() || target.is_empty() {
      return 0.0;
    }

    let min_len = anchor.len().min(target.len());
    let max_len = anchor.len().max(target.len());

    let mut matches = 0;
    for i in 0..min_len {
      if anchor[i] == target[i] {
        matches += 1;
      }
    }

    matches as f32 / max_len as f32
  }
}
