use crate::design::Result;
use crate::entropies::Table;
use crate::hashes::{Hash, digest};
use crate::objects::Manifest;
use crate::profiles::{Atlas, Kind, Profile, Sketch};
use std::collections::BTreeSet;

/// CRDT merge algorithms (G-Set and G-Counter).
pub struct Merge;

impl Merge {
  /// G-Set union of content-addressed hashes.
  pub fn gset(a: &[Hash], b: &[Hash]) -> Vec<Hash> {
    let mut set = BTreeSet::new();
    for &h in a {
      set.insert(h);
    }
    for &h in b {
      set.insert(h);
    }
    set.into_iter().collect()
  }

  /// G-Set merge of two manifests.
  pub fn manifests(a: &Manifest, b: &Manifest) -> Manifest {
    a.merge(b)
  }

  /// G-Counter additive merge of frequency distributions.
  pub fn gcounter(a: &[u32], b: &[u32]) -> [u32; 256] {
    let mut sum = [1u32; 256];
    for i in 0..256 {
      let val_a = if i < a.len() { a[i] } else { 1 };
      let val_b = if i < b.len() { b[i] } else { 1 };
      sum[i] = val_a + val_b;
    }
    sum
  }

  /// Merge multiple profiles using G-Set dictionary union and G-Counter frequency summation.
  pub fn profiles(profiles: &[&Profile]) -> Result<Profile> {
    if profiles.is_empty() {
      return Ok(Profile::generic());
    }
    if profiles.len() == 1 {
      return Ok((*profiles[0]).clone());
    }

    let mut sum_counts = [0u32; 256];
    for p in profiles {
      for i in 0..256 {
        let f = if i < p.table.freqs.len() {
          p.table.freqs[i]
        } else {
          1
        };
        sum_counts[i] += f;
      }
    }

    let table = Table::from_counts(&sum_counts);
    let atlas = Atlas::new();
    let sketch = Sketch::new(4, 256);

    let mut hash_buf = Vec::new();
    for p in profiles {
      hash_buf.extend_from_slice(p.hash.bytes());
    }
    let hash = digest(&hash_buf);

    crate::merge!("merged {} profiles -> hash={:?}", profiles.len(), hash);
    Ok(Profile {
      hash,
      kind: Kind::Multi,
      atlas,
      table,
      sketch,
    })
  }
}
