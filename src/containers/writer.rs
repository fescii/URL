use super::format::{MAGIC, VERSION};
use crate::hashes::{Hash, digest};
use crate::objects::{Blob, Manifest};
use std::io::{self, Write as IoWrite};

/// .urls container file exporter.
pub struct Writer;

impl Writer {
  /// Export manifest, referenced blobs, and pinned prerequisite profile into a portable .urls container.
  pub fn pack(prereq: &Hash, manifest: &Manifest, blobs: &[Blob]) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // 1. Magic bytes & version
    buffer.write_all(MAGIC)?;
    buffer.write_all(&[VERSION])?;

    // 2. Prerequisite profile hash (32 bytes)
    buffer.write_all(prereq.bytes())?;

    // 3. Manifest items
    buffer.write_all(&(manifest.items.len() as u32).to_le_bytes())?;
    for item in &manifest.items {
      buffer.write_all(item.bytes())?;
    }

    // 4. Blobs
    buffer.write_all(&(blobs.len() as u32).to_le_bytes())?;
    for blob in blobs {
      buffer.write_all(blob.hash.bytes())?;
      buffer.write_all(&(blob.data.len() as u32).to_le_bytes())?;
      buffer.write_all(&blob.data)?;
    }

    // 5. Append trailing BLAKE3 integrity hash
    let integrity = digest(&buffer);
    buffer.write_all(integrity.bytes())?;

    Ok(buffer)
  }
}
