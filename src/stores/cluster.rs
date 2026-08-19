use crate::stores::delta::Delta;

/// Fast MinHash sketch and anchor centroid clustering for string collections.
#[derive(Debug, Default, Clone)]
pub struct Cluster {
  anchors: Vec<Vec<u8>>,
  sketches: Vec<u32>,
  delta: Delta,
}

impl Cluster {
  pub fn new() -> Self {
    Self {
      anchors: Vec::new(),
      sketches: Vec::new(),
      delta: Delta::new(),
    }
  }

  /// Compute 32-bit character multiset sketch (MinHash-style fingerprint).
  pub fn sketch(data: &[u8]) -> u32 {
    if data.is_empty() {
      return 0;
    }

    let mut h1 = 0x811c9dc5u32;
    let mut h2 = 0x517cc1b7u32;

    for window in data.windows(2) {
      let val = ((window[0] as u32) << 8) | (window[1] as u32);
      h1 = h1.wrapping_mul(0x01000193) ^ val;
      h2 = h2.wrapping_add(val).rotate_left(5);
    }

    h1 ^ h2
  }

  /// Find the best matching anchor for an incoming payload.
  /// Returns Some((anchor_idx, &anchor_bytes)) if similarity >= threshold, or None if new anchor needed.
  pub fn locate(&mut self, payload: &[u8]) -> Option<(u8, &[u8])> {
    if self.anchors.is_empty() {
      self.register(payload);
      return None;
    }

    let mut best_idx = None;
    let mut best_score = 0.0f32;

    for (idx, anchor) in self.anchors.iter().enumerate() {
      let score = self.delta.similarity(anchor, payload);
      if score > best_score {
        best_score = score;
        best_idx = Some(idx);
      }
    }

    // 35% minimum positional similarity threshold to justify delta encoding
    if best_score >= 0.35 {
      let idx = best_idx.unwrap();
      Some((idx as u8, &self.anchors[idx]))
    } else if self.anchors.len() < 255 {
      self.register(payload);
      None
    } else {
      // Fallback to highest available anchor if capacity reached
      let idx = best_idx.unwrap_or(0);
      Some((idx as u8, &self.anchors[idx]))
    }
  }

  /// Register a new anchor string into the cluster.
  pub fn register(&mut self, anchor: &[u8]) -> u8 {
    let idx = self.anchors.len() as u8;
    self.sketches.push(Self::sketch(anchor));
    self.anchors.push(anchor.to_vec());
    idx
  }

  /// Retrieve an anchor by index.
  pub fn get(&self, idx: u8) -> Option<&[u8]> {
    self.anchors.get(idx as usize).map(|v| v.as_slice())
  }

  pub fn len(&self) -> usize {
    self.anchors.len()
  }

  pub fn is_empty(&self) -> bool {
    self.anchors.is_empty()
  }
}
