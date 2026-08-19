use crate::verify;
use clap::Args;
use std::fs;

#[derive(Args, Debug)]
pub struct Command {
  /// File to verify integrity for (e.g. .urls container or profile)
  pub file: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let bytes = fs::read(&cmd.file)?;
  let ok = verify(&bytes);
  if ok {
    crate::verify!("integrity check PASSED for {}", cmd.file);
    Ok(())
  } else {
    crate::error!("integrity check FAILED for {}", cmd.file);
    Err(format!("integrity verification failed for {}", cmd.file).into())
  }
}
