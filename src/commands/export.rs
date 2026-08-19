use crate::{Blob, Manifest, Profile, encode, export};
use clap::Args;
use std::fs;

#[derive(Args, Debug)]
pub struct Command {
  /// URLs to package into container
  pub refs: Vec<String>,

  /// Path of output .urls container file
  #[arg(short, long)]
  pub output: String,

  /// Profile reference (optional)
  #[arg(short, long)]
  pub profile: Option<String>,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  if cmd.refs.is_empty() {
    return Err("no URLs specified to export".into());
  }

  let profile = Profile::generic();
  let mut blobs = Vec::with_capacity(cmd.refs.len());
  let mut item_hashes = Vec::with_capacity(cmd.refs.len());

  for url in &cmd.refs {
    let code = encode(url, Some(&profile))?;
    let blob = Blob::new(code.into_bytes());
    item_hashes.push(blob.hash);
    blobs.push(blob);
  }

  let manifest = Manifest::new(Some(profile.hash), item_hashes);
  let container_bytes = export(&profile.hash, &manifest, &blobs)?;

  fs::write(&cmd.output, &container_bytes)?;
  crate::export!(
    "exported {} URLs into container {} ({} bytes)",
    cmd.refs.len(),
    cmd.output,
    container_bytes.len()
  );

  Ok(())
}
