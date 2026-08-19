use crate::design::{Error, Result};

/// Run-Length Encoding on tokenized byte streams.
///
/// Prefix flag 0x00 = raw passthrough (no expansion penalty).
/// Prefix flag 0x01 = run-length compressed.
pub struct Rle;

const MARKER: u8 = 0xFF;
const MIN_RUN: usize = 3;

impl Rle {
  /// Pack a byte slice using run-length encoding.
  pub fn pack(src: &[u8]) -> Vec<u8> {
    if src.is_empty() {
      return Vec::new();
    }

    let mut encoded = Vec::with_capacity(src.len());
    let mut i = 0;

    while i < src.len() {
      let b = src[i];
      let mut run = 1usize;

      while i + run < src.len() && src[i + run] == b && run < 255 {
        run += 1;
      }

      if run >= MIN_RUN {
        encoded.push(MARKER);
        encoded.push(run as u8);
        encoded.push(b);
        i += run;
      } else {
        if b == MARKER {
          encoded.push(MARKER);
          encoded.push(0x00);
          encoded.push(MARKER);
        } else {
          encoded.push(b);
        }
        i += 1;
      }
    }

    // Only apply if strictly smaller with the 1-byte header
    if encoded.len() + 1 < src.len() + 1 {
      let mut out = Vec::with_capacity(1 + encoded.len());
      out.push(0x01); // Mode 1: Compressed
      out.extend_from_slice(&encoded);
      out
    } else {
      let mut out = Vec::with_capacity(1 + src.len());
      out.push(0x00); // Mode 0: Raw
      out.extend_from_slice(src);
      out
    }
  }

  /// Unpack a run-length encoded byte slice.
  pub fn unpack(src: &[u8]) -> Result<Vec<u8>> {
    if src.is_empty() {
      return Ok(Vec::new());
    }

    let mode = src[0];
    let payload = &src[1..];

    if mode == 0x00 {
      // Raw passthrough
      return Ok(payload.to_vec());
    }

    if mode != 0x01 {
      return Err(Error::Codec(format!("invalid RLE mode byte: {mode}")));
    }

    let mut out = Vec::with_capacity(payload.len() * 2);
    let mut i = 0;

    while i < payload.len() {
      let b = payload[i];

      if b == MARKER {
        if i + 2 >= payload.len() {
          return Err(Error::Codec("truncated RLE sequence".into()));
        }
        let count = payload[i + 1];
        let byte = payload[i + 2];

        if count == 0x00 && byte == MARKER {
          out.push(MARKER);
        } else {
          for _ in 0..count {
            out.push(byte);
          }
        }
        i += 3;
      } else {
        out.push(b);
        i += 1;
      }
    }

    Ok(out)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rle_roundtrip_runs() {
    let input = b"https://https://https://aaa/bbbbbbb/ccc";
    let packed = Rle::pack(input);
    let unpacked = Rle::unpack(&packed).unwrap();
    assert_eq!(unpacked, input);
  }

  #[test]
  fn test_rle_no_expansion_on_random() {
    let input = b"aB3xQy7Z9k-AbCdEfGhIj";
    let packed = Rle::pack(input);
    assert_eq!(packed[0], 0x00); // Raw mode
    let unpacked = Rle::unpack(&packed).unwrap();
    assert_eq!(unpacked, input);
  }

  #[test]
  fn test_rle_literal_marker() {
    let input = vec![0xFF, 0x01, 0xFF, 0xFF, 0x00, 0xFF];
    let packed = Rle::pack(&input);
    let unpacked = Rle::unpack(&packed).unwrap();
    assert_eq!(unpacked, input);
  }
}
