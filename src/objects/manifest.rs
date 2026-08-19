use crate::hashes::{Hash, digest};
use std::collections::BTreeSet;

/// Manifest containing prerequisite profile hash and list of child blob content hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
  pub hash: Hash,
  pub prereq: Option<Hash>,
  pub items: Vec<Hash>,
}

impl Manifest {
  /// Construct canonical, deterministically-sorted manifest.
  pub fn new(prereq: Option<Hash>, items: Vec<Hash>) -> Self {
    let set: BTreeSet<Hash> = items.into_iter().collect();
    let canonical_items: Vec<Hash> = set.into_iter().collect();

    let mut buffer = Vec::new();
    if let Some(p) = &prereq {
      buffer.extend_from_slice(p.bytes());
    }
    for item in &canonical_items {
      buffer.extend_from_slice(item.bytes());
    }
    let hash = digest(&buffer);
    Self {
      hash,
      prereq,
      items: canonical_items,
    }
  }

  /// CRDT G-Set merge of two manifests: commutative, associative, idempotent set union.
  pub fn merge(&self, other: &Manifest) -> Self {
    let mut items = self.items.clone();
    items.extend_from_slice(&other.items);
    let merged_prereq = self.prereq.or(other.prereq);
    Self::new(merged_prereq, items)
  }
}
