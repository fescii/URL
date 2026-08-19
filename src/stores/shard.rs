use super::cache::Cache;
use super::cluster::Cluster;
use super::delta::{Delta, Record};
use super::index::{Index, Location, Tier};
use super::log::Log;
use super::succinct::Succinct;
use super::symbol::Symbol;
use crate::codecs::Rle;
use crate::design::{Error, Result};
use crate::hashes::Hash;
use bytes::Bytes;
use std::path::Path;

/// Shard representing an isolated partition with positional delta compression, FSST, anchor clustering, succinct MPHF, and ARC cache.
pub struct Shard {
  pub log: Log,
  pub index: Index,
  pub succinct: Option<Succinct>,
  pub cache: Cache,
  pub cluster: Cluster,
  pub delta: Delta,
  pub symbol: Symbol,
}

impl Shard {
  /// Open a shard log and recover index.
  pub fn open<P: AsRef<Path>>(path: P, cache_capacity: usize) -> Result<Self> {
    let mut log = Log::open(path)?;
    let mut index = Index::new();
    log.recover(&mut index)?;
    let cache = Cache::new(cache_capacity);
    Ok(Self {
      log,
      index,
      succinct: None,
      cache,
      cluster: Cluster::new(),
      delta: Delta::new(),
      symbol: Symbol::new(),
    })
  }

  /// Route hash to a shard index in 0..count.
  pub fn route(hash: &Hash, count: usize) -> usize {
    let first_byte = hash.bytes()[0] as usize;
    let c = if count == 0 { 1 } else { count };
    first_byte % c
  }

  /// Put data with positional delta, RLE, FSST and tokenized compression.
  pub fn put(&mut self, hash: &Hash, data: &[u8]) -> Result<Location> {
    // Check cache first
    if let Some(loc) = self.locate(hash) {
      return Ok(loc);
    }

    // Algorithmic tokenization preprocessing for HTTP/HTTPS URLs
    let (processed, is_tokenized) = if let Ok(s) = std::str::from_utf8(data) {
      if s.starts_with("http://") || s.starts_with("https://") {
        let atlas = crate::profiles::Atlas::new();
        let trans = crate::codecs::Trans::new();
        let tok = atlas.tokenize(s);
        let packed = trans.pack(&tok);
        // Apply RLE on top of tokenized stream
        let rled = Rle::pack(&packed);
        (rled, true)
      } else {
        (data.to_vec(), false)
      }
    } else {
      (data.to_vec(), false)
    };

    // Check anchor similarity in cluster
    let packed_data = match self.cluster.locate(&processed) {
      Some((anchor_idx, anchor_bytes)) => {
        let rec = self.delta.diff(anchor_idx, anchor_bytes, &processed);
        let mut buf = Vec::with_capacity(38 + rec.diffs.len());
        let mode = if is_tokenized { 5u8 } else { 1u8 };
        buf.push(mode); // Mode 1 = Delta Raw, Mode 5 = Delta Tokenized+RLE
        buf.push(rec.anchor);
        buf.extend_from_slice(&rec.target_len.to_le_bytes());
        for word in &rec.mask {
          buf.extend_from_slice(&word.to_le_bytes());
        }
        buf.extend_from_slice(&(rec.diffs.len() as u16).to_le_bytes());
        buf.extend_from_slice(&rec.diffs);
        buf
      }
      None => {
        let compressed = self.symbol.compress(&processed);
        let mut buf = Vec::with_capacity(2 + compressed.len());
        let mode = if is_tokenized { 4u8 } else { 0u8 };
        buf.push(mode); // Mode 0 = Anchor Raw, Mode 4 = Anchor Tokenized+RLE
        buf.push((self.cluster.len().saturating_sub(1)) as u8);
        buf.extend_from_slice(&compressed);
        buf
      }
    };

    // 2. Append to log
    let loc = self.log.append(hash, &packed_data)?;
    self.index.insert(*hash, loc);
    self.cache.put(*hash, Bytes::copy_from_slice(data));

    Ok(loc)
  }

  /// Read and decode data record at location. Returns zero-copy `Bytes` on cache hit or raw mmap path.
  pub fn get(&mut self, hash: &Hash, loc: Location) -> Result<Bytes> {
    // Cache hit: zero-copy Bytes clone (ref-count bump, no allocation)
    if let Some(cached) = self.cache.get(hash) {
      return Ok(cached);
    }

    let raw: Bytes = self.log.read(loc.offset, loc.length)?;
    if raw.is_empty() {
      return Err(Error::Codec("empty store record".into()));
    }

    let mode = raw[0];
    let decoded = match mode {
      0 => {
        // FSST Anchor Raw — decompress only once
        let payload = &raw[2..];
        Bytes::from(self.symbol.decompress(payload))
      }
      4 => {
        // FSST Anchor Tokenized + RLE
        let payload = &raw[2..];
        let rle_bytes = self.symbol.decompress(payload);
        let tokenized_bytes = Rle::unpack(&rle_bytes)?;
        let trans = crate::codecs::Trans::new();
        let tok = trans.unpack(&tokenized_bytes)?;
        let atlas = crate::profiles::Atlas::new();
        Bytes::from(
          atlas
            .detokenize(&tok)
            .map(|s| s.into_bytes())
            .map_err(|e| Error::Codec(format!("detokenize error: {e}")))?,
        )
      }
      1 | 5 => {
        // Positional Delta 256-bit mask (mode 1 = raw, mode 5 = tokenized+RLE)
        if raw.len() < 38 {
          return Err(Error::Codec("truncated delta log record".into()));
        }
        let anchor_idx = raw[1];
        let target_len = u16::from_le_bytes([raw[2], raw[3]]);
        let mask = [
          u64::from_le_bytes(raw[4..12].try_into().map_err(|_| Error::Codec("slice error".into()))?),
          u64::from_le_bytes(raw[12..20].try_into().map_err(|_| Error::Codec("slice error".into()))?),
          u64::from_le_bytes(raw[20..28].try_into().map_err(|_| Error::Codec("slice error".into()))?),
          u64::from_le_bytes(raw[28..36].try_into().map_err(|_| Error::Codec("slice error".into()))?),
        ];
        let diff_len = u16::from_le_bytes([raw[36], raw[37]]) as usize;
        if raw.len() < 38 + diff_len {
          return Err(Error::Codec("truncated delta diffs".into()));
        }
        let diffs = raw[38..38 + diff_len].to_vec();

        let rec = Record {
          anchor: anchor_idx,
          target_len,
          mask,
          diffs,
        };

        let anchor_bytes = self
          .cluster
          .get(anchor_idx)
          .ok_or_else(|| Error::Codec(format!("missing cluster anchor {anchor_idx}")))?;

        let restored_bytes = self.delta.apply(anchor_bytes, &rec)?;

        if mode == 5 {
          // Undo RLE, then detokenize
          let tokenized_bytes = Rle::unpack(&restored_bytes)?;
          let trans = crate::codecs::Trans::new();
          let tok = trans.unpack(&tokenized_bytes)?;
          let atlas = crate::profiles::Atlas::new();
          Bytes::from(
            atlas
              .detokenize(&tok)
              .map(|s| s.into_bytes())
              .map_err(|e| Error::Codec(format!("detokenize error: {e}")))?,
          )
        } else {
          Bytes::from(restored_bytes)
        }
      }
      // Unknown mode: return raw bytes slice (zero-copy)
      _ => raw,
    };

    self.cache.put(*hash, decoded.clone());
    Ok(decoded)
  }

  /// Locate byte offset and length for key hash in active mutable index or succinct sealed index.
  pub fn locate(&self, hash: &Hash) -> Option<Location> {
    if let Some(loc) = self.index.get(hash) {
      return Some(*loc);
    }
    if let Some(ref succ) = self.succinct {
      if let Some(offset) = succ.query(hash) {
        return Some(Location {
          file: 0,
          offset,
          length: 0,
          tier: Tier::Hot,
        });
      }
    }
    None
  }

  /// Compact mutable memory index into a succinct Minimal Perfect Hash index (90% RAM reduction).
  pub fn seal(&mut self) {
    if self.index.is_empty() {
      return;
    }

    let entries: Vec<(Hash, u64)> = self
      .index
      .entries()
      .map(|(&h, loc)| (h, loc.offset))
      .collect();

    self.succinct = Some(Succinct::build(entries));
    self.index.clear();
  }

  /// Return total indexed keys in this shard.
  pub fn len(&self) -> usize {
    self.index.len() + self.succinct.as_ref().map_or(0, |s| s.len())
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Estimate memory size of index in bytes.
  pub fn memory_size(&self) -> usize {
    let mutable_ram =
      self.index.len() * (std::mem::size_of::<Hash>() + std::mem::size_of::<Location>() + 16);
    let succinct_ram = self.succinct.as_ref().map_or(0, |s| s.memory_size());
    mutable_ram + succinct_ram
  }
}
