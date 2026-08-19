/// Context extractor for URL string sequence modeling.
pub struct Context {
  pub order: usize,
}

impl Context {
  pub const fn new(order: usize) -> Self {
    Self { order }
  }

  /// Extract context hash from preceding bytes.
  pub fn extract(&self, text: &[u8], index: usize) -> u32 {
    let start = if index >= self.order {
      index - self.order
    } else {
      0
    };
    let slice = &text[start..index];
    let mut hash = 0u32;
    for &b in slice {
      hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash
  }
}
