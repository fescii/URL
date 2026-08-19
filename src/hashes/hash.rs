/// Fixed 32-byte BLAKE3 hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
  pub const fn new(bytes: [u8; 32]) -> Self {
    Self(bytes)
  }

  pub fn bytes(&self) -> &[u8; 32] {
    &self.0
  }
}

/// Compute BLAKE3 digest of bytes.
pub fn digest(data: &[u8]) -> Hash {
  let result = blake3::hash(data);
  Hash(*result.as_bytes())
}
