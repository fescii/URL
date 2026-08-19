pub mod batch;
pub mod format;
pub mod parser;

pub use batch::{Batch, Stats};
pub use format::Format;
pub use parser::Parser;

use crate::design::Result;
use crate::stores::Store;
use std::path::Path;

/// Ingestion configuration parameters.
#[derive(Debug, Clone)]
pub struct Config {
  pub path: String,
  pub format: Format,
  pub batch_size: usize,
  pub limit: Option<usize>,
  pub col_idx: Option<usize>,
  pub seal: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      path: String::new(),
      format: Format::Auto,
      batch_size: 10_000,
      limit: None,
      col_idx: None,
      seal: false,
    }
  }
}

/// Top-level coordinator for high-speed file and stream ingestion.
pub struct Ingest;

impl Ingest {
  /// Ingest URLs from file path using provided configuration.
  pub fn run<P: AsRef<Path>>(path: P, config: &Config, store: &mut Store) -> Result<Stats> {
    let fmt = if config.format == Format::Auto {
      Format::detect(&path)
    } else {
      config.format
    };

    let parser = Parser::new(fmt, config.col_idx);
    let mut urls = parser.parse_file(path)?;

    if let Some(lim) = config.limit {
      urls.truncate(lim);
    }

    let mut stats = Batch::process(&urls, store, config.batch_size)?;

    if config.seal {
      store.seal();
      stats.ram_bytes = store.memory_size();
      crate::store!("sealed store into succinct MPHF index post-ingest");
    }

    Ok(stats)
  }
}
