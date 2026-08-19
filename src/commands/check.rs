use crate::check;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
  /// Target URL or link to probe
  pub target: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let state = check(&cmd.target);
  crate::check!("target={} -> state={:?}", cmd.target, state);
  Ok(())
}
