use crate::hashes::Hash;
use std::collections::HashMap;

/// Storage tier placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
  Hot,
  Warm,
  Cold,
}

/// Placement metadata pointing to on-disk payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
  pub file: u32,
  pub offset: u64,
  pub length: u32,
  pub tier: Tier,
}

impl Location {
  pub const fn new(file: u32, offset: u64, length: u32, tier: Tier) -> Self {
    Self {
      file,
      offset,
      length,
      tier,
    }
  }
}

/// In-memory hash index mapping BLAKE3 hash to location record.
#[derive(Default)]
pub struct Index {
  map: HashMap<Hash, Location>,
}

impl Index {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
    }
  }

  pub fn insert(&mut self, hash: Hash, location: Location) {
    self.map.insert(hash, location);
  }

  pub fn get(&self, hash: &Hash) -> Option<&Location> {
    self.map.get(hash)
  }

  pub fn entries(&self) -> impl Iterator<Item = (&Hash, &Location)> {
    self.map.iter()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn clear(&mut self) {
    self.map.clear();
  }
}
