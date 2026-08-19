use crate::{decode, open};
use clap::Args;
use std::fs;

#[derive(Args, Debug)]
pub struct Command {
  /// Path to .urls container file to open and decode
  pub file: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let bytes = fs::read(&cmd.file)?;
  let (prereq, manifest, blobs) = open(&bytes)?;

  crate::container!(
    "opened container file={} prereq={:?} items={}",
    cmd.file,
    prereq,
    blobs.len()
  );

  for (i, blob) in blobs.iter().enumerate() {
    if let Ok(code_str) = std::str::from_utf8(&blob.data) {
      match decode(code_str) {
        Ok(url) => crate::decode!("[#{}] code={} -> url={}", i + 1, code_str, url),
        Err(e) => crate::warn!("[#{}] failed to decode code {}: {}", i + 1, code_str, e),
      }
    }
  }

  Ok(())
}
