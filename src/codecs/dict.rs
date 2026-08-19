use crate::design::{Error, Result};
use std::fs;
use std::path::Path;

/// Zstandard dictionary compression for shard log blocks.
///
/// A dictionary is trained on the first SAMPLE_COUNT URL payloads and
/// saved to `dict.bin` inside the store directory. All 4MB log blocks
/// are compressed and decompressed using this shared dictionary,
/// achieving 40-60% savings on top of tokenization.
pub struct Dict {
  dict: Vec<u8>,
}

const SAMPLE_COUNT: usize = 1024;
const LEVEL: i32 = 3;

impl Dict {
  /// Create a Dict wrapper from raw dictionary bytes.
  pub fn from_bytes(dict: Vec<u8>) -> Self {
    Self { dict }
  }

  /// Train a Zstd dictionary from a collection of sample byte slices.
  pub fn train(samples: &[Vec<u8>]) -> Result<Self> {
    if samples.is_empty() {
      return Err(Error::Codec("no samples for dictionary training".into()));
    }
    let refs: Vec<&[u8]> = samples.iter().map(|v| v.as_slice()).collect();
    let dict_bytes = zstd::dict::from_samples(&refs, 112 * 1024)
      .map_err(|e| Error::Codec(format!("dict train error: {e}")))?;
    crate::store!(
      "trained zstd dictionary size={} bytes from {} samples",
      dict_bytes.len(),
      samples.len()
    );
    Ok(Self { dict: dict_bytes })
  }

  /// Save dictionary bytes to disk.
  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    fs::write(path, &self.dict).map_err(Error::from)
  }

  /// Load dictionary bytes from disk.
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let bytes = fs::read(path).map_err(Error::from)?;
    Ok(Self { dict: bytes })
  }

  /// Whether this dict has been initialized (non-empty).
  pub fn ready(&self) -> bool {
    !self.dict.is_empty()
  }

  /// Compress a data block using the trained dictionary.
  pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
    if self.dict.is_empty() {
      // Fallback: plain zstd without dictionary
      let out = zstd::encode_all(data, LEVEL)
        .map_err(|e| Error::Codec(format!("zstd compress error: {e}")))?;
      return Ok(out);
    }
    let mut enc = zstd::Encoder::with_dictionary(Vec::new(), LEVEL, &self.dict)
      .map_err(|e| Error::Codec(format!("zstd encoder error: {e}")))?;
    std::io::Write::write_all(&mut enc, data)
      .map_err(|e| Error::Codec(format!("zstd write error: {e}")))?;
    enc
      .finish()
      .map_err(|e| Error::Codec(format!("zstd finish error: {e}")))
  }

  /// Decompress a data block using the trained dictionary.
  pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
    if self.dict.is_empty() {
      let out = zstd::decode_all(data)
        .map_err(|e| Error::Codec(format!("zstd decompress error: {e}")))?;
      return Ok(out);
    }
    let dec = zstd::Decoder::with_dictionary(data, &self.dict)
      .map_err(|e| Error::Codec(format!("zstd decoder error: {e}")))?;
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut { dec }, &mut out)
      .map_err(|e| Error::Codec(format!("zstd read error: {e}")))?;
    Ok(out)
  }

  /// How many samples to collect before training.
  pub const fn samples() -> usize {
    SAMPLE_COUNT
  }

  /// Raw dict bytes.
  pub fn bytes(&self) -> &[u8] {
    &self.dict
  }
}
