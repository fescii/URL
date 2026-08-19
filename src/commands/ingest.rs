use crate::ingests::{Config, Format, Ingest};
use crate::stores::Store;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Command {
  /// Path to CSV, TSV, or TXT file containing URLs
  #[arg(value_name = "FILE")]
  pub path: PathBuf,

  /// Batch chunk size (default: 10,000)
  #[arg(short = 'b', long, default_value = "10000")]
  pub batch: usize,

  /// Maximum total lines to ingest (optional)
  #[arg(short = 'l', long)]
  pub limit: Option<usize>,

  /// Target URL column index in CSV/TSV (0-indexed)
  #[arg(short = 'c', long)]
  pub col: Option<usize>,

  /// Storage directory path (default: .urls_store)
  #[arg(short = 'd', long, default_value = ".urls_store")]
  pub dir: PathBuf,

  /// Seal in-memory index into succinct MPHF bitvectors after ingestion
  #[arg(short = 's', long)]
  pub seal: bool,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let mut store = Store::open(&cmd.dir)?;

  let config = Config {
    path: cmd.path.to_string_lossy().to_string(),
    format: Format::detect(&cmd.path),
    batch_size: cmd.batch,
    limit: cmd.limit,
    col_idx: cmd.col,
    seal: cmd.seal,
  };

  crate::ingest!(
    "started ingest file={} batch={} limit={:?} dir={}",
    cmd.path.display(),
    cmd.batch,
    cmd.limit,
    cmd.dir.display()
  );

  let stats = Ingest::run(&cmd.path, &config, &mut store)?;

  crate::ingest!(
    "completed ingest count={} raw_bytes={} mb={:.2} duration_ms={} rate_per_sec={:.0} ram_per_key={:.2}",
    stats.count,
    stats.raw_bytes,
    stats.raw_bytes as f64 / (1024.0 * 1024.0),
    stats.duration_millis,
    stats.rate,
    stats.ram_bytes as f64 / stats.count.max(1) as f64
  );

  Ok(())
}
