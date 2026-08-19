use crate::decode;
use clap::Args;
use std::fs;
use std::io::{self, BufRead};

#[derive(Args, Debug)]
pub struct Command {
  /// Shortcode to decode (pass '-' to read from standard input)
  #[arg(default_value = "")]
  pub code: String,

  /// Optional batch file containing one shortcode per line
  #[arg(short, long)]
  pub file: Option<String>,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  if let Some(path) = cmd.file {
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        let url = decode(trimmed)?;
        crate::decode!("{url}");
      }
    }
    return Ok(());
  }

  if cmd.code == "-" {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
      let line = line?;
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        let url = decode(trimmed)?;
        crate::decode!("{url}");
      }
    }
    return Ok(());
  }

  if cmd.code.is_empty() {
    return Err(
      "missing shortcode to decode; specify <code>, '--file <path>', or '-' for stdin".into(),
    );
  }

  let url = decode(&cmd.code)?;
  crate::decode!("{url}");
  Ok(())
}
