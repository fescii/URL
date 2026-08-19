use crate::hashes::digest;

/// Linear frequency sketch (Count-Min sketch) for sub-linear memory frequency estimation.
#[derive(Debug, Clone)]
pub struct Sketch {
  pub rows: usize,
  pub cols: usize,
  pub table: Vec<u32>,
}

impl Sketch {
  pub fn new(rows: usize, cols: usize) -> Self {
    let r = rows.max(1);
    let c = cols.max(1);
    Self {
      rows: r,
      cols: c,
      table: vec![0; r * c],
    }
  }

  fn hash_index(&self, row: usize, item: &[u8]) -> usize {
    let mut key = Vec::with_capacity(item.len() + 1);
    key.push(row as u8);
    key.extend_from_slice(item);
    let hash = digest(&key);
    let bytes = hash.bytes();
    let val = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    val % self.cols
  }

  /// Record item occurrence into count-min sketch.
  pub fn record(&mut self, item: &[u8]) {
    for row in 0..self.rows {
      let col = self.hash_index(row, item);
      let idx = row * self.cols + col;
      self.table[idx] = self.table[idx].saturating_add(1);
    }
  }

  /// Point query frequency estimate (upper bound with high probability).
  pub fn query(&self, item: &[u8]) -> u32 {
    let mut min_count = u32::MAX;
    for row in 0..self.rows {
      let col = self.hash_index(row, item);
      let idx = row * self.cols + col;
      min_count = min_count.min(self.table[idx]);
    }
    if min_count == u32::MAX { 0 } else { min_count }
  }

  /// G-Counter additive merge of two Count-Min sketches.
  pub fn merge(&mut self, other: &Self) {
    assert_eq!(self.rows, other.rows);
    assert_eq!(self.cols, other.cols);
    for i in 0..self.table.len() {
      self.table[i] = self.table[i].saturating_add(other.table[i]);
    }
  }
}
