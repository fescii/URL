use super::atlas::Atlas;
use super::sketch::Sketch;
use crate::design::{Error, Result};
use crate::entropies::Table;
use crate::hashes::{Hash, digest};
use std::fs;
use std::path::Path;

/// Profile type discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
  Generic,
  Single,
  Multi,
}

/// Profile containing structural atlas dictionary, frequency tables, and statistical sketch.
#[derive(Debug, Clone)]
pub struct Profile {
  pub hash: Hash,
  pub kind: Kind,
  pub atlas: Atlas,
  pub table: Table,
  pub sketch: Sketch,
}

impl Profile {
  /// Built-in generic profile available everywhere by default with zero configuration.
  pub fn generic() -> Self {
    let atlas = Atlas::new();
    let mut counts = [1u32; 256];
    // ASCII alphanumeric & punctuation
    for b in b'a'..=b'z' {
      counts[b as usize] = 10;
    }
    for b in b'0'..=b'9' {
      counts[b as usize] = 8;
    }
    for b in b'A'..=b'Z' {
      counts[b as usize] = 6;
    }
    for &b in b"-._~/:?&=%#+@" {
      counts[b as usize] = 15;
    }
    // Token markers (0x80..0xFF)
    for i in 0x80..=0xFF {
      counts[i] = 12;
    }

    let table = Table::from_counts(&counts);
    let sketch = Sketch::new(4, 256);
    let hash = digest(b"urls-v1-generic-profile");

    Self {
      hash,
      kind: Kind::Generic,
      atlas,
      table,
      sketch,
    }
  }

  /// Single-domain profile.
  pub fn single(domain: &str) -> Self {
    let mut prof = Self::generic();
    prof.kind = Kind::Single;
    let mut bytes = b"urls-v1-single-profile:".to_vec();
    bytes.extend_from_slice(domain.as_bytes());
    prof.hash = digest(&bytes);
    prof
  }

  /// Multi-domain profile.
  pub fn multi(domains: &[&str]) -> Self {
    let mut prof = Self::generic();
    prof.kind = Kind::Multi;
    let mut bytes = b"urls-v1-multi-profile:".to_vec();
    for &d in domains {
      bytes.extend_from_slice(d.as_bytes());
    }
    prof.hash = digest(&bytes);
    prof
  }

  /// Train a statistical profile from a corpus of URL strings.
  pub fn train(corpus: &[&str]) -> Result<Self> {
    if corpus.is_empty() {
      return Ok(Self::generic());
    }

    let atlas = Atlas::new();
    let mut counts = [1u32; 256];

    for &url in corpus {
      let normalized = atlas.normalize(url);
      let tokenized = atlas.tokenize(&normalized);
      for &byte in &tokenized {
        counts[byte as usize] += 1;
      }
    }

    let table = Table::from_counts(&counts);
    let mut sketch = Sketch::new(4, 256);
    for &url in corpus {
      sketch.record(url.as_bytes());
    }

    let table_bytes = table.serialize();
    let hash = digest(&table_bytes);

    crate::train!(
      "trained profile from {} URLs -> hash={:?}",
      corpus.len(),
      hash
    );

    Ok(Self {
      hash,
      kind: Kind::Multi,
      atlas,
      table,
      sketch,
    })
  }

  /// Save profile binary representation to file.
  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let serialized = self.table.serialize();
    fs::write(path, serialized).map_err(Error::from)
  }

  /// Load profile binary representation from file.
  pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    let bytes = fs::read(path).map_err(Error::from)?;
    let table = Table::deserialize(&bytes)
      .ok_or_else(|| Error::Profile("failed to deserialize profile table".to_string()))?;
    let atlas = Atlas::new();
    let sketch = Sketch::new(4, 256);
    let hash = digest(&bytes);

    Ok(Self {
      hash,
      kind: Kind::Multi,
      atlas,
      table,
      sketch,
    })
  }
}
