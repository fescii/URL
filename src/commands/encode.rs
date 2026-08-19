use crate::design::Score;
use crate::{Profile, encode};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
  pub url: String,
  #[arg(short, long)]
  pub profile: Option<String>,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let profile = Profile::generic();
  let code = encode(&cmd.url, Some(&profile))?;
  let score = Score::new(cmd.url.len(), code.len());

  crate::encode!("{code}");
  crate::info!(
    "raw={} compressed={} ratio={:.2}%",
    score.raw,
    score.compressed,
    score.ratio * 100.0
  );
  Ok(())
}
