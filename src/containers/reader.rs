use super::format::{MAGIC, VERSION};
use crate::hashes::{Hash, digest};
use crate::objects::{Blob, Manifest};
use std::io::{self, Error, ErrorKind};

/// .urls container file parser and verifier.
pub struct Reader;

impl Reader {
  /// Verify container integrity hash and unpack prerequisite profile hash, manifest, and blobs.
  pub fn unpack(bytes: &[u8]) -> io::Result<(Hash, Manifest, Vec<Blob>)> {
    // Minimum container length: Magic (4) + Version (1) + Prereq (32) + ManifestCount (4) + BlobCount (4) + Integrity (32) = 77
    if bytes.len() < 77 {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "container payload truncated",
      ));
    }

    // 1. Check magic bytes
    if &bytes[0..4] != MAGIC {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "invalid container magic signature",
      ));
    }

    // 2. Check version
    let version = bytes[4];
    if version != VERSION {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("unsupported container format version {version}"),
      ));
    }

    // 3. Verify integrity hash
    let payload_len = bytes.len() - 32;
    let payload = &bytes[..payload_len];
    let expected_integrity = &bytes[payload_len..];
    let actual_integrity = digest(payload);

    if actual_integrity.bytes() != expected_integrity {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "container integrity hash mismatch: file corrupted or modified",
      ));
    }

    // 4. Parse prerequisite profile hash
    let mut offset = 5;
    let mut prereq_bytes = [0u8; 32];
    prereq_bytes.copy_from_slice(&bytes[offset..offset + 32]);
    let prereq = Hash::new(prereq_bytes);
    offset += 32;

    // 5. Parse manifest items
    let manifest_count = u32::from_le_bytes([
      bytes[offset],
      bytes[offset + 1],
      bytes[offset + 2],
      bytes[offset + 3],
    ]) as usize;
    offset += 4;

    let mut manifest_items = Vec::with_capacity(manifest_count);
    for _ in 0..manifest_count {
      if offset + 32 > payload_len {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "manifest item table truncated",
        ));
      }
      let mut item_bytes = [0u8; 32];
      item_bytes.copy_from_slice(&bytes[offset..offset + 32]);
      manifest_items.push(Hash::new(item_bytes));
      offset += 32;
    }
    let manifest = Manifest::new(Some(prereq), manifest_items);

    // 6. Parse blobs
    if offset + 4 > payload_len {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "blob count header truncated",
      ));
    }
    let blob_count = u32::from_le_bytes([
      bytes[offset],
      bytes[offset + 1],
      bytes[offset + 2],
      bytes[offset + 3],
    ]) as usize;
    offset += 4;

    let mut blobs = Vec::with_capacity(blob_count);
    for _ in 0..blob_count {
      if offset + 36 > payload_len {
        return Err(Error::new(ErrorKind::InvalidData, "blob header truncated"));
      }
      let mut blob_hash_bytes = [0u8; 32];
      blob_hash_bytes.copy_from_slice(&bytes[offset..offset + 32]);
      let blob_hash = Hash::new(blob_hash_bytes);
      offset += 32;

      let data_len = u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
      ]) as usize;
      offset += 4;

      if offset + data_len > payload_len {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "blob data stream truncated",
        ));
      }
      let data = bytes[offset..offset + data_len].to_vec();
      offset += data_len;

      blobs.push(Blob {
        hash: blob_hash,
        data,
      });
    }

    Ok((prereq, manifest, blobs))
  }
}
