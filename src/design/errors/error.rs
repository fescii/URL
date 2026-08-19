use thiserror::Error;

/// Central error definition for the zero-storage URL compressor and storage engine.
#[derive(Error, Debug)]
pub enum Error {
  #[error("codec error: {0}")]
  Codec(String),

  #[error("grammar error: {0}")]
  Grammar(String),

  #[error("entropy coder error: {0}")]
  Entropy(String),

  #[error("statistical model error: {0}")]
  Model(String),

  #[error("hash verification error: {0}")]
  Hash(String),

  #[error("object error: {0}")]
  Object(String),

  #[error("profile error: {0}")]
  Profile(String),

  #[error("store error: {0}")]
  Store(String),

  #[error("container format error: {0}")]
  Container(String),

  #[error("state health error: {0}")]
  State(String),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
}
