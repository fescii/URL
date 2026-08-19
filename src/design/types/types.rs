use crate::design::results::Result;

/// Common URL target string wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target(pub String);

impl Target {
  pub const fn new(url: String) -> Self {
    Self(url)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Compressed shortcode representation wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Code(pub String);

impl Code {
  pub const fn new(code: String) -> Self {
    Self(code)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Compression performance and ratio metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score {
  pub raw: usize,
  pub compressed: usize,
  pub ratio: f32,
}

impl Score {
  pub fn new(raw: usize, compressed: usize) -> Self {
    let ratio = if raw == 0 {
      1.0
    } else {
      compressed as f32 / raw as f32
    };
    Self {
      raw,
      compressed,
      ratio,
    }
  }
}

/// Core trait for encoding transformations.
pub trait Encode {
  fn encode(&self) -> Result<Vec<u8>>;
}

/// Core trait for decoding transformations.
pub trait Decode: Sized {
  fn decode(bytes: &[u8]) -> Result<Self>;
}
