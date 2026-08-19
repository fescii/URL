use crate::reports::Runner;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Command {
  /// Path to CSV file containing URLs (default: data/list.csv)
  #[arg(short = 'i', long, default_value = "data/list.csv")]
  pub input: PathBuf,

  /// Output directory for multi-scale reports (default: reports)
  #[arg(short = 'o', long, default_value = "reports")]
  pub out: PathBuf,

  /// Custom scale checkpoints (comma-separated, e.g. 1000,5000,10000)
  #[arg(short = 's', long, value_delimiter = ',')]
  pub scales: Option<Vec<usize>>,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let input_path = if cmd.input.exists() {
    cmd.input
  } else if std::path::Path::new("list.csv").exists() {
    PathBuf::from("list.csv")
  } else {
    cmd.input
  };

  crate::ingest!(
    "running full multi-scale analysis: input={} out_dir={}",
    input_path.display(),
    cmd.out.display()
  );

  let mut runner = Runner::new(&input_path, &cmd.out);
  if let Some(custom_scales) = cmd.scales {
    runner = runner.with_scales(custom_scales);
  }
  let results = runner.run()?;

  crate::ingest!(
    "multi-scale report generation complete across {} checkpoints",
    results.len()
  );

  Ok(())
}
