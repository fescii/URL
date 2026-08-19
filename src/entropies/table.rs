pub const SCALE_BITS: u32 = 12;
pub const SCALE: u32 = 1 << SCALE_BITS; // 4096

/// Cumulative frequency distribution table for rANS.
#[derive(Debug, Clone)]
pub struct Table {
  pub freqs: Vec<u32>,
  pub cumul: Vec<u32>,
  pub slots: Vec<u8>,
}

impl Table {
  /// Create normalized rANS frequency table from raw symbol counts.
  pub fn from_counts(counts: &[u32; 256]) -> Self {
    let total: u64 = counts.iter().map(|&c| c as u64).sum();
    let mut freqs = vec![1u32; 256];

    if total > 0 {
      // Allocate proportionally ensuring each seen symbol has at least frequency 1
      let remaining_scale = SCALE - 256;
      for i in 0..256 {
        if counts[i] > 0 {
          let share = ((counts[i] as u64 * remaining_scale as u64) / total) as u32;
          freqs[i] += share;
        }
      }
      // Adjust any rounding discrepancy to ensure sum == SCALE
      let current_sum: u32 = freqs.iter().sum();
      if current_sum < SCALE {
        freqs[0] += SCALE - current_sum;
      } else if current_sum > SCALE {
        let mut diff = current_sum - SCALE;
        for i in (0..256).rev() {
          if freqs[i] > 1 {
            let sub = (freqs[i] - 1).min(diff);
            freqs[i] -= sub;
            diff -= sub;
            if diff == 0 {
              break;
            }
          }
        }
      }
    } else {
      // Equal distribution
      let per = SCALE / 256;
      for i in 0..256 {
        freqs[i] = per;
      }
    }

    let mut cumul = Vec::with_capacity(257);
    let mut sum = 0;
    cumul.push(0);
    for &f in &freqs {
      sum += f;
      cumul.push(sum);
    }

    // Direct lookup table for slot -> symbol (O(1) decode)
    let mut slots = vec![0u8; SCALE as usize];
    for (sym, (&start, &end)) in cumul.iter().zip(cumul.iter().skip(1)).enumerate() {
      for slot in start..end {
        slots[slot as usize] = sym as u8;
      }
    }

    Self {
      freqs,
      cumul,
      slots,
    }
  }

  /// Uniform table for fallback default encoding.
  pub fn uniform() -> Self {
    let counts = [1u32; 256];
    Self::from_counts(&counts)
  }

  /// Serialize frequency distribution into compact byte format.
  pub fn serialize(&self) -> Vec<u8> {
    let mut out = Vec::with_capacity(256 * 2);
    for &f in &self.freqs {
      out.extend_from_slice(&(f as u16).to_le_bytes());
    }
    out
  }

  /// Deserialize frequency distribution.
  pub fn deserialize(bytes: &[u8]) -> Option<Self> {
    if bytes.len() < 512 {
      return None;
    }
    let mut freqs = Vec::with_capacity(256);
    for i in 0..256 {
      let f = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]) as u32;
      freqs.push(f.max(1));
    }

    let mut cumul = Vec::with_capacity(257);
    let mut sum = 0;
    cumul.push(0);
    for &f in &freqs {
      sum += f;
      cumul.push(sum);
    }

    let mut slots = vec![0u8; SCALE as usize];
    for (sym, (&start, &end)) in cumul.iter().zip(cumul.iter().skip(1)).enumerate() {
      for slot in (start as usize)..(end as usize).min(SCALE as usize) {
        slots[slot] = sym as u8;
      }
    }

    Some(Self {
      freqs,
      cumul,
      slots,
    })
  }
}
