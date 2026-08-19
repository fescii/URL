use crate::stat;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
  /// Shortcode to query statistics and health for
  pub code: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let (hits, state) = stat(&cmd.code);
  crate::stat!("code={} hits={} state={:?}", cmd.code, hits, state);
  Ok(())
}
