use crate::train;
use clap::Args;
use std::fs;

#[derive(Args, Debug)]
pub struct Command {
  /// File containing training corpus (one URL per line)
  pub corpus: String,

  /// Output path for serialized profile
  #[arg(short, long)]
  pub output: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let content = fs::read_to_string(&cmd.corpus)?;
  let urls: Vec<&str> = content
    .lines()
    .map(|l| l.trim())
    .filter(|l| !l.is_empty())
    .collect();

  let profile = train(&urls)?;
  profile.save(&cmd.output)?;

  crate::train!(
    "trained and saved profile with hash={:?} to {}",
    profile.hash,
    cmd.output
  );
  Ok(())
}
