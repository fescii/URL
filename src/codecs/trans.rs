use crate::design::{Error, Result};

const TOKENS: &[(&str, u8)] = &[
  ("https://www.", 0x01),
  ("https://shop.", 0x05),
  ("https://portal.", 0x06),
  ("https://api.", 0x07),
  ("https://dev.", 0x08),
  ("https://media.", 0x09),
  ("https://blog.", 0x0A),
  ("https://m.", 0x0B),
  ("https://mail.", 0x0C),
  ("https://", 0x02),
  ("http://www.", 0x03),
  ("http://", 0x04),
  ("?utm_source=", 0x10),
  ("&utm_medium=", 0x11),
  ("&utm_campaign=", 0x12),
  ("&utm_content=", 0x13),
  ("&utm_term=", 0x14),
  ("&gclid=", 0x15),
  ("&fbclid=", 0x16),
  ("&session_id=", 0x17),
  ("&timestamp=", 0x18),
  (".com/", 0x20),
  (".org/", 0x21),
  (".net/", 0x22),
  (".io/", 0x23),
  ("/user/profile/", 0x30),
  ("/shop/products/deals/", 0x31),
  ("/shop/products/", 0x32),
  ("/wiki/articles/", 0x33),
  ("/explore/tags/", 0x34),
  ("/feed/trending/", 0x35),
  ("/blob/main/", 0x36),
  ("/track/event/", 0x37),
  ("/questions/", 0x38),
  ("/search?q=", 0x39),
];

/// Transcoder for sub-byte packing of numeric strings, hex digests, and common URL tokens.
pub struct Trans;

impl Trans {
  pub const fn new() -> Self {
    Self
  }

  /// Pack high-entropy substrings and common URL boilerplates into compact binary markers.
  /// Marker format:
  /// - 0x7E 0x01 [u8 len] [u64 le bytes]: decimal number (e.g. Snowflake ID)
  /// - 0x7E 0x02 [u8 byte_len] [raw bytes]: hex decoded bytes
  /// - 0x7E 0x03 [u8 token_id]: common URL token
  /// - 0x7E 0x7E: literal 0x7E byte
  pub fn pack(&self, input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
      // 1. Check for common boilerplate tokens
      let mut matched_token = None;
      for &(token_str, token_id) in TOKENS {
        let token_bytes = token_str.as_bytes();
        if i + token_bytes.len() <= input.len() && &input[i..i + token_bytes.len()] == token_bytes {
          matched_token = Some((token_bytes.len(), token_id));
          break;
        }
      }

      if let Some((t_len, t_id)) = matched_token {
        out.push(0x7E);
        out.push(0x03);
        out.push(t_id);
        i += t_len;
        continue;
      }

      // 2. Check for numeric sequence >= 8 digits
      if input[i].is_ascii_digit() {
        let start = i;
        while i < input.len() && input[i].is_ascii_digit() {
          i += 1;
        }
        let len = i - start;
        if len >= 8 && len <= 20 {
          if let Ok(s) = std::str::from_utf8(&input[start..i]) {
            if let Ok(val) = s.parse::<u64>() {
              out.push(0x7E);
              out.push(0x01);
              out.push(len as u8);
              out.extend_from_slice(&val.to_le_bytes());
              continue;
            }
          }
        }
        // Fallback to literal
        out.extend_from_slice(&input[start..i]);
        continue;
      }

      // 3. Check for hex string >= 8 chars (even length)
      if is_hex_char(input[i]) {
        let start = i;
        while i < input.len() && is_hex_char(input[i]) {
          i += 1;
        }
        let len = i - start;
        if len >= 8 && len % 2 == 0 {
          if let Ok(s) = std::str::from_utf8(&input[start..i]) {
            if let Ok(hex_bytes) = decode_hex(s) {
              out.push(0x7E);
              out.push(0x02);
              out.push(hex_bytes.len() as u8);
              out.extend_from_slice(&hex_bytes);
              continue;
            }
          }
        }
        out.extend_from_slice(&input[start..i]);
        continue;
      }

      // 4. Literal byte with escape handling
      if input[i] == 0x7E {
        out.push(0x7E);
        out.push(0x7E);
      } else {
        out.push(input[i]);
      }
      i += 1;
    }

    out
  }

  /// Unpack binary markers back into original strings.
  pub fn unpack(&self, input: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
      if input[i] == 0x7E && i + 1 < input.len() {
        let kind = input[i + 1];
        if kind == 0x7E {
          out.push(0x7E);
          i += 2;
          continue;
        } else if kind == 0x03 && i + 2 < input.len() {
          let token_id = input[i + 2];
          if let Some(&(token_str, _)) = TOKENS.iter().find(|(_, id)| *id == token_id) {
            out.extend_from_slice(token_str.as_bytes());
            i += 3;
            continue;
          }
        } else if kind == 0x01 && i + 2 + 8 <= input.len() {
          let len = input[i + 2] as usize;
          let val = u64::from_le_bytes([
            input[i + 3],
            input[i + 4],
            input[i + 5],
            input[i + 6],
            input[i + 7],
            input[i + 8],
            input[i + 9],
            input[i + 10],
          ]);
          let num_str = format!("{:0width$}", val, width = len);
          out.extend_from_slice(num_str.as_bytes());
          i += 11;
          continue;
        } else if kind == 0x02 && i + 2 < input.len() {
          let byte_len = input[i + 2] as usize;
          if i + 3 + byte_len <= input.len() {
            let hex_str = encode_hex(&input[i + 3..i + 3 + byte_len]);
            out.extend_from_slice(hex_str.as_bytes());
            i += 3 + byte_len;
            continue;
          }
        }
      }

      out.push(input[i]);
      i += 1;
    }

    Ok(out)
  }
}

fn is_hex_char(b: u8) -> bool {
  b.is_ascii_digit() || (b >= b'a' && b <= b'f')
}

fn decode_hex(s: &str) -> Result<Vec<u8>> {
  if s.len() % 2 != 0 {
    return Err(Error::Codec("odd hex length".to_string()));
  }
  (0..s.len())
    .step_by(2)
    .map(|i| {
      u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| Error::Codec(format!("invalid hex: {e}")))
    })
    .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
  let mut s = String::with_capacity(bytes.len() * 2);
  for &b in bytes {
    s.push_str(&format!("{:02x}", b));
  }
  s
}
