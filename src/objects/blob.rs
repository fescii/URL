use crate::hashes::{Hash, digest};

/// Content-addressed immutable raw byte blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
  pub hash: Hash,
  pub data: Vec<u8>,
}

impl Blob {
  pub fn new(data: Vec<u8>) -> Self {
    let hash = digest(&data);
    Self { hash, data }
  }
}
