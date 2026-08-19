use super::hash::{Hash, digest};
use crate::design::{Error, Result};

/// Short profile version tag prefixed on compressed codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag {
  pub id: u8,
  pub hash: Hash,
}

impl Tag {
  pub const fn new(id: u8, hash: Hash) -> Self {
    Self { id, hash }
  }

  /// Generic default tag (ID 0).
  pub fn generic() -> Self {
    Self {
      id: 0,
      hash: digest(b"urls-v1-generic-profile"),
    }
  }

  /// Single-domain profile tag (ID 1).
  pub fn single(domain: &str) -> Self {
    let mut bytes = b"urls-v1-single-profile:".to_vec();
    bytes.extend_from_slice(domain.as_bytes());
    Self {
      id: 1,
      hash: digest(&bytes),
    }
  }

  /// Template AST profile tag (ID 3).
  pub fn template() -> Self {
    Self {
      id: 3,
      hash: digest(b"urls-v1-template-profile"),
    }
  }

  /// Prefix character corresponding to profile tag ID.
  pub fn prefix(&self) -> char {
    match self.id {
      0 => '0',
      1 => '1',
      2 => '2',
      3 => '3',
      n if n < 26 => (b'a' + (n - 4)) as char,
      _ => '0',
    }
  }

  /// Parse tag from leading character.
  pub fn parse(ch: char) -> Result<Self> {
    match ch {
      '0' => Ok(Self::generic()),
      '1' => Ok(Self::new(1, digest(b"urls-v1-single-profile"))),
      '2' => Ok(Self::new(2, digest(b"urls-v1-multi-profile"))),
      '3' => Ok(Self::template()),
      'a'..='z' => {
        let id = (ch as u8 - b'a') + 4;
        Ok(Self::new(id, digest(&[id])))
      }
      _ => Err(Error::Profile(format!(
        "unrecognized profile version tag prefix '{ch}'"
      ))),
    }
  }
}
