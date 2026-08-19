use super::shard::Shard;
use bytes::Bytes;
use crate::design::{Error, Result};
use crate::hashes::{Hash, digest};
use std::fs;
use std::path::{Path, PathBuf};

/// Embedded Bitcask-derived storage engine with sharded logs, dual succinct indexing, positional delta compression, and ARC caching.
pub struct Store {
  dir: PathBuf,
  shards: Vec<Shard>,
}

impl Store {
  /// Open or initialize embedded storage engine inside the specified directory.
  pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
    Self::open_sharded(dir, 4, 1024)
  }

  /// Open storage engine with custom shard count and per-shard ARC cache capacity.
  pub fn open_sharded<P: AsRef<Path>>(
    dir: P,
    shard_count: usize,
    cache_capacity: usize,
  ) -> Result<Self> {
    let dir_buf = dir.as_ref().to_path_buf();
    fs::create_dir_all(&dir_buf).map_err(Error::from)?;

    let count = shard_count.max(1);
    let mut shards = Vec::with_capacity(count);

    for i in 0..count {
      let log_path = dir_buf.join(format!("shard_{i}.log"));
      let shard = Shard::open(log_path, cache_capacity)?;
      shards.push(shard);
    }

    crate::store!(
      "opened store at {} with {} shards",
      dir_buf.display(),
      count
    );

    Ok(Self {
      dir: dir_buf,
      shards,
    })
  }

  /// Store raw bytes and return content-addressed BLAKE3 hash.
  pub fn put(&mut self, data: &[u8]) -> Result<Hash> {
    let hash = digest(data);
    self.put_with_hash(hash, data)
  }

  /// Store raw bytes under a specific custom key/shortcut string.
  pub fn put_key(&mut self, key: &str, data: &[u8]) -> Result<Hash> {
    let hash = digest(key.as_bytes());
    self.put_with_hash(hash, data)
  }

  fn put_with_hash(&mut self, hash: Hash, data: &[u8]) -> Result<Hash> {
    let shard_idx = Shard::route(&hash, self.shards.len());
    let shard = &mut self.shards[shard_idx];

    // Put with Delta + FSST symbol compression
    let _loc = shard.put(&hash, data)?;

    crate::trace!("stored blob hash={:?} len={}", hash, data.len());
    Ok(hash)
  }

  /// Retrieve zero-copy `Bytes` by custom key string. No allocation on cache hit.
  pub fn get_key(&mut self, key: &str) -> Result<Option<Bytes>> {
    let hash = digest(key.as_bytes());
    self.get(&hash)
  }

  /// Retrieve zero-copy `Bytes` by BLAKE3 hash. Returns `Bytes` sliced into mmap — no alloc on hit.
  pub fn get(&mut self, hash: &Hash) -> Result<Option<Bytes>> {
    let shard_idx = Shard::route(hash, self.shards.len());
    let shard = &mut self.shards[shard_idx];

    // 1. ARC cache: returns cloned Bytes (ref-count bump, zero alloc)
    if let Some(cached) = shard.cache.get(hash) {
      crate::cache!("arc cache hit for hash={:?}", hash);
      return Ok(Some(cached));
    }

    // 2. Index lookup → log read → zero-copy Bytes slice
    if let Some(loc) = shard.locate(hash) {
      let data: Bytes = shard.get(hash, loc)?;
      crate::log!("log read for hash={:?} offset={}", hash, loc.offset);
      return Ok(Some(data));
    }

    Ok(None)
  }

  /// Seal all shards into succinct Minimal Perfect Hash indexes (90% RAM reduction).
  pub fn seal(&mut self) {
    for shard in &mut self.shards {
      shard.seal();
    }
    crate::store!("sealed store into succinct MPHF bitvectors");
  }

  /// Total count of indexed objects across all shards.
  pub fn len(&self) -> usize {
    self.shards.iter().map(|s| s.len()).sum()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Total RAM footprint in bytes of in-memory indexes across all shards.
  pub fn memory_size(&self) -> usize {
    self.shards.iter().map(|s| s.memory_size()).sum()
  }

  /// Total stored on-disk footprint in bytes across all shard logs.
  pub fn disk_size(&self) -> u64 {
    let mut total = 0u64;
    for (i, _) in self.shards.iter().enumerate() {
      let path = self.dir.join(format!("shard_{i}.log"));
      if let Ok(meta) = fs::metadata(&path) {
        total += meta.len();
      }
    }
    total
  }
}
