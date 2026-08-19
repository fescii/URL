/// Elias-Fano quasi-succinct monotone sequence encoding for high-speed bitvector offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elias {
  count: usize,
  low_width: usize,
  low_bits: Vec<u64>,
  high_bits: Vec<u64>,
}

impl Elias {
  /// Build Elias-Fano representation from a monotonically increasing sequence of integers.
  pub fn build(seq: &[u64]) -> Self {
    if seq.is_empty() {
      return Self {
        count: 0,
        low_width: 0,
        low_bits: Vec::new(),
        high_bits: Vec::new(),
      };
    }

    let count = seq.len();
    let max_val = *seq.last().unwrap_or(&0);
    let low_width = if max_val > count as u64 && count > 0 {
      let ratio = (max_val as f64 / count as f64).log2().floor() as usize;
      ratio.min(63)
    } else {
      0
    };

    // 1. Pack lower bits
    let total_low_bits = count * low_width;
    let low_u64_count = (total_low_bits + 63) / 64;
    let mut low_bits = vec![0u64; low_u64_count.max(1)];

    if low_width > 0 {
      let mask = (1u64 << low_width) - 1;
      for (i, &val) in seq.iter().enumerate() {
        let low = val & mask;
        let bit_offset = i * low_width;
        let word_idx = bit_offset / 64;
        let bit_rem = bit_offset % 64;

        low_bits[word_idx] |= low << bit_rem;
        if bit_rem + low_width > 64 && word_idx + 1 < low_bits.len() {
          low_bits[word_idx + 1] |= low >> (64 - bit_rem);
        }
      }
    }

    // 2. Unary encode high bits
    let max_high = if low_width > 0 {
      max_val >> low_width
    } else {
      max_val
    };
    let total_high_bits = count + (max_high as usize) + 1;
    let high_u64_count = (total_high_bits + 63) / 64;
    let mut high_bits = vec![0u64; high_u64_count.max(1)];

    for (i, &val) in seq.iter().enumerate() {
      let high = if low_width > 0 { val >> low_width } else { val };
      let bit_pos = i + (high as usize);
      let word_idx = bit_pos / 64;
      let bit_rem = bit_pos % 64;
      if word_idx < high_bits.len() {
        high_bits[word_idx] |= 1u64 << bit_rem;
      }
    }

    Self {
      count,
      low_width,
      low_bits,
      high_bits,
    }
  }

  /// Retrieve the integer at the given index in O(1) operations.
  pub fn get(&self, index: usize) -> u64 {
    if index >= self.count {
      return 0;
    }

    // 1. Extract low bits
    let low = if self.low_width > 0 {
      let bit_offset = index * self.low_width;
      let word_idx = bit_offset / 64;
      let bit_rem = bit_offset % 64;
      let mask = (1u64 << self.low_width) - 1;

      let mut val = self.low_bits[word_idx] >> bit_rem;
      if bit_rem + self.low_width > 64 && word_idx + 1 < self.low_bits.len() {
        val |= self.low_bits[word_idx + 1] << (64 - bit_rem);
      }
      val & mask
    } else {
      0
    };

    // 2. Select index-th 1-bit in high bitvector
    let pos = self.select(index);
    let high = (pos - index) as u64;

    if self.low_width > 0 {
      (high << self.low_width) | low
    } else {
      high
    }
  }

  /// Find the bit position of the k-th 1-bit (0-indexed).
  fn select(&self, k: usize) -> usize {
    let mut target = k + 1;
    let mut bit_pos = 0;

    for &word in &self.high_bits {
      let ones = word.count_ones() as usize;
      if target <= ones {
        // The target 1-bit is in this word
        let w = word;
        for i in 0..64 {
          if (w & (1u64 << i)) != 0 {
            target -= 1;
            if target == 0 {
              return bit_pos + i;
            }
          }
        }
      }
      target -= ones;
      bit_pos += 64;
    }

    bit_pos
  }

  pub fn len(&self) -> usize {
    self.count
  }

  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  /// Total memory consumption in bits.
  pub fn bit_size(&self) -> usize {
    (self.low_bits.len() + self.high_bits.len()) * 64
  }
}
