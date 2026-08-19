/// Front-coding encoder and decoder for sorted URL byte blocks.
///
/// Stores the shared prefix length and the unique suffix only.
/// A full anchor is stored every BLOCK bytes to allow random scan.
pub struct Front;

const BLOCK: usize = 512;

impl Front {
  /// Encode a sorted slice of byte slices using front-coding.
  /// Returns a flat byte buffer that can be appended to a log block.
  pub fn encode(entries: &[&[u8]]) -> Vec<u8> {
    if entries.is_empty() {
      return Vec::new();
    }

    let mut out = Vec::new();
    let mut prev: &[u8] = b"";

    for (i, entry) in entries.iter().enumerate() {
      if i % BLOCK == 0 {
        // Anchor: store full entry with u16 length prefix + 0x00 shared-prefix marker
        let len = entry.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.push(0x00); // shared prefix = 0 (full entry)
        out.extend_from_slice(entry);
        prev = entry;
      } else {
        // Delta: shared prefix length + suffix
        let shared = common_prefix_len(prev, entry);
        let suffix = &entry[shared..];
        let prefix_len = shared as u16;
        let suffix_len = suffix.len() as u16;
        out.extend_from_slice(&prefix_len.to_le_bytes());
        out.extend_from_slice(&suffix_len.to_le_bytes());
        out.extend_from_slice(suffix);
        prev = entry;
      }
    }

    out
  }

  /// Decode a front-coded block back into the original entries.
  pub fn decode(data: &[u8]) -> Vec<Vec<u8>> {
    let mut entries = Vec::new();
    let mut pos = 0;
    let mut prev: Vec<u8> = Vec::new();
    let mut block_pos = 0usize;

    while pos + 2 <= data.len() {
      if block_pos % BLOCK == 0 {
        // Anchor: read full length, skip 0x00, read entry
        if pos + 3 > data.len() {
          break;
        }
        let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        // data[pos + 2] == 0x00 (prefix marker)
        pos += 3;
        if pos + len > data.len() {
          break;
        }
        let entry = data[pos..pos + len].to_vec();
        pos += len;
        prev = entry.clone();
        entries.push(entry);
      } else {
        // Delta: read prefix_len + suffix_len + suffix
        if pos + 4 > data.len() {
          break;
        }
        let prefix_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        let suffix_len = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if pos + suffix_len > data.len() {
          break;
        }
        let mut entry = prev[..prefix_len.min(prev.len())].to_vec();
        entry.extend_from_slice(&data[pos..pos + suffix_len]);
        pos += suffix_len;
        prev = entry.clone();
        entries.push(entry);
      }

      block_pos += 1;
    }

    entries
  }
}

fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
  a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_front_roundtrip_sorted() {
    let urls: Vec<&[u8]> = vec![
      b"https://shop.google.com/user/profile/abc",
      b"https://shop.google.com/user/profile/def",
      b"https://shop.google.com/wiki/articles/xyz",
      b"https://www.amazon.com/dp/B08N5WRWNW",
    ];
    let encoded = Front::encode(&urls);
    let decoded = Front::decode(&encoded);
    assert_eq!(decoded.len(), urls.len());
    for (orig, dec) in urls.iter().zip(decoded.iter()) {
      assert_eq!(*orig, dec.as_slice());
    }
  }

  #[test]
  fn test_front_encode_smaller_than_raw() {
    let prefix = b"https://shop.google.com/user/profile/";
    let urls: Vec<Vec<u8>> = (0..100)
      .map(|i| {
        let mut v = prefix.to_vec();
        v.extend_from_slice(format!("item{i:04}").as_bytes());
        v
      })
      .collect();
    let refs: Vec<&[u8]> = urls.iter().map(|v| v.as_slice()).collect();
    let raw_size: usize = urls.iter().map(|v| v.len()).sum();
    let encoded = Front::encode(&refs);
    assert!(encoded.len() < raw_size, "front-coding must reduce size for domain-clustered URLs");
  }
}
