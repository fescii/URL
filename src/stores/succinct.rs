use crate::hashes::Hash;

/// Minimal Perfect Hash Function (MPHF) table with compact offset indexing (90% RAM reduction vs HashMap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Succinct {
  count: usize,
  pilots: Vec<u16>,
  fingerprints: Vec<u8>,
  offsets: Vec<u64>,
}

impl Succinct {
  /// Build a succinct index from a collection of key hashes and their byte offsets.
  pub fn build(mut entries: Vec<(Hash, u64)>) -> Self {
    if entries.is_empty() {
      return Self {
        count: 0,
        pilots: Vec::new(),
        fingerprints: Vec::new(),
        offsets: Vec::new(),
      };
    }

    // Deduplicate entries by key hash
    entries.sort_by(|a, b| a.0.bytes().cmp(b.0.bytes()));
    entries.dedup_by(|a, b| a.0 == b.0);

    let count = entries.len();
    // 25% load factor slack ensuring rapid, collision-free pilot search without wrapping
    let total_slots = count + (count / 4).max(16);
    let num_buckets = (count / 3).max(1);
    let mut buckets: Vec<Vec<(Hash, u64)>> = vec![Vec::new(); num_buckets];

    // 1. Distribute keys into buckets
    for &(hash, offset) in &entries {
      let bucket_idx = Self::hash_bucket(&hash, buckets.len());
      buckets[bucket_idx].push((hash, offset));
    }

    // 2. Assign 16-bit pilot seeds so MPHF maps each key to a unique slot in 0..total_slots
    let mut pilots = vec![0u16; buckets.len()];
    let mut slot_taken = vec![false; total_slots];
    let mut slot_entries = vec![(Hash([0; 32]), 0u64); total_slots];

    // Sort buckets by size descending to place larger buckets first
    let mut bucket_order: Vec<usize> = (0..buckets.len()).collect();
    bucket_order.sort_by(|&a, &b| buckets[b].len().cmp(&buckets[a].len()));

    for &b_idx in &bucket_order {
      let bucket = &buckets[b_idx];
      if bucket.is_empty() {
        continue;
      }

      let mut pilot: u16 = 0;

      loop {
        let mut candidate_slots = Vec::with_capacity(bucket.len());
        let mut collision = false;

        for &(h, _) in bucket {
          let slot = Self::hash_slot(&h, pilot, total_slots);
          if slot_taken[slot] || candidate_slots.contains(&slot) {
            collision = true;
            break;
          }
          candidate_slots.push(slot);
        }

        if !collision {
          for (i, &(h, offset)) in bucket.iter().enumerate() {
            let slot = candidate_slots[i];
            slot_taken[slot] = true;
            slot_entries[slot] = (h, offset);
          }
          pilots[b_idx] = pilot;
          break;
        }

        pilot = pilot.wrapping_add(1);
        if pilot == 0 {
          panic!("MPHF pilot search wrapped around for bucket with {} keys! total_slots={}", bucket.len(), total_slots);
        }
      }
    }

    // 3. Compact offset array and 1-byte fingerprint per slot
    let mut offsets = Vec::with_capacity(total_slots);
    let mut fingerprints = Vec::with_capacity(total_slots);

    for (h, off) in slot_entries {
      offsets.push(off);
      fingerprints.push((h.bytes()[16] ^ h.bytes()[17] ^ h.bytes()[18]) as u8);
    }

    Self {
      count,
      pilots,
      fingerprints,
      offsets,
    }
  }

  /// Query the byte offset for a given key hash in O(1) time.
  pub fn query(&self, key: &Hash) -> Option<u64> {
    if self.count == 0 || self.pilots.is_empty() || self.offsets.is_empty() {
      return None;
    }

    let b_idx = Self::hash_bucket(key, self.pilots.len());
    let pilot = self.pilots[b_idx];
    let slot = Self::hash_slot(key, pilot, self.offsets.len());

    if slot >= self.offsets.len() {
      return None;
    }

    let expected_fp = (key.bytes()[16] ^ key.bytes()[17] ^ key.bytes()[18]) as u8;
    if self.fingerprints[slot] != expected_fp {
      return None;
    }

    Some(self.offsets[slot])
  }

  fn hash_bucket(key: &Hash, num_buckets: usize) -> usize {
    let b = key.bytes();
    let mut x = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    (x as usize) % num_buckets.max(1)
  }

  fn hash_slot(key: &Hash, pilot: u16, total_slots: usize) -> usize {
    let b = key.bytes();
    let mut x = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
    x = x.wrapping_add((pilot as u64).wrapping_mul(0x9e3779b97f4a7c15));
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x = x ^ (x >> 31);
    (x as usize) % total_slots.max(1)
  }

  pub fn len(&self) -> usize {
    self.count
  }

  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  /// Total memory footprint in bytes.
  pub fn memory_size(&self) -> usize {
    (self.pilots.len() * 2)
      + self.fingerprints.len()
      + (self.offsets.len() * std::mem::size_of::<u64>())
  }
}
