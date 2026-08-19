use std::collections::HashMap;

/// Statistical context frequency table.
#[derive(Debug, Clone)]
pub struct Stats {
  pub counts: HashMap<u32, [u32; 256]>,
}

impl Stats {
  pub fn new() -> Self {
    Self {
      counts: HashMap::new(),
    }
  }

  /// Record symbol occurrence under context hash.
  pub fn record(&mut self, ctx: u32, sym: u8) {
    let entry = self.counts.entry(ctx).or_insert([1; 256]);
    entry[sym as usize] = entry[sym as usize].saturating_add(1);
  }

  /// Query symbol probability under context hash.
  pub fn prob(&self, ctx: u32, sym: u8) -> f32 {
    if let Some(entry) = self.counts.get(&ctx) {
      let total: u32 = entry.iter().sum();
      entry[sym as usize] as f32 / total as f32
    } else {
      1.0 / 256.0
    }
  }
}
