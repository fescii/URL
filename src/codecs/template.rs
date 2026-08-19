use crate::design::{Error, Result};

/// Structural schema template descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
  pub id: u8,
  pub pattern: &'static str,
  pub slots: usize,
}

/// Known parametric URL templates for high-parameter platforms.
pub const SCHEMAS: &[Schema] = &[
  Schema {
    id: 1,
    pattern: "https://www.amazon.com/dp/{0}?ref={1}&utm_source={2}&utm_medium={3}&utm_campaign={4}&utm_content={5}&gclid={6}",
    slots: 7,
  },
  Schema {
    id: 2,
    pattern: "https://store.steampowered.com/app/{0}/{1}/?utm_source={2}&utm_medium={3}&utm_campaign={4}&fbclid={5}",
    slots: 6,
  },
  Schema {
    id: 3,
    pattern: "https://www.instagram.com/p/{0}/?igshid={1}",
    slots: 2,
  },
  Schema {
    id: 4,
    pattern: "https://www.reddit.com/r/{0}/comments/{1}/{2}/",
    slots: 3,
  },
  Schema {
    id: 5,
    pattern: "https://www.linkedin.com/posts/{0}",
    slots: 1,
  },
  Schema {
    id: 6,
    pattern: "https://api.github.com/repos/{0}/{1}/issues?state={2}&sort={3}&direction={4}",
    slots: 5,
  },
  Schema {
    id: 7,
    pattern: "https://github.com/{0}/{1}/blob/{2}/{3}",
    slots: 4,
  },
  Schema {
    id: 8,
    pattern: "https://github.com/{0}/{1}/pull/{2}",
    slots: 3,
  },
  Schema {
    id: 9,
    pattern: "mailto:{0}?subject={1}",
    slots: 2,
  },
  Schema {
    id: 10,
    pattern: "bitcoin:{0}?amount={1}&label={2}",
    slots: 3,
  },
  Schema {
    id: 11,
    pattern: "magnet:?xt=urn:btih:{0}&dn={1}",
    slots: 2,
  },
  Schema {
    id: 12,
    pattern: "https://x.com/{0}/status/{1}",
    slots: 2,
  },
  Schema {
    id: 13,
    pattern: "https://www.youtube.com/watch?v={0}",
    slots: 1,
  },
];

/// Parametric template decomposition and reconstruction engine.
pub struct Template;

impl Template {
  pub const fn new() -> Self {
    Self
  }

  /// Match a URL against known structural schemas and extract slot substrings.
  pub fn extract<'a>(&self, url: &'a str) -> Option<(u8, Vec<&'a str>)> {
    for schema in SCHEMAS {
      if let Some(slots) = self.match_pattern(schema.pattern, url) {
        if slots.len() == schema.slots {
          return Some((schema.id, slots));
        }
      }
    }
    None
  }

  fn match_pattern<'a>(&self, pattern: &str, url: &'a str) -> Option<Vec<&'a str>> {
    let mut slots = Vec::new();
    let mut p_idx = 0;
    let mut u_idx = 0;

    let p_bytes = pattern.as_bytes();
    let u_bytes = url.as_bytes();

    while p_idx < p_bytes.len() {
      if p_bytes[p_idx] == b'{' {
        // Find closing brace '}'
        let close = pattern[p_idx..].find('}')? + p_idx;
        p_idx = close + 1;

        if p_idx == p_bytes.len() {
          // Placeholder extends to end of URL
          let slot = &url[u_idx..];
          slots.push(slot);
          u_idx = u_bytes.len();
          break;
        }

        // Find next literal delimiter segment in pattern
        let next_placeholder = pattern[p_idx..].find('{').map(|pos| pos + p_idx);
        let literal_end = next_placeholder.unwrap_or(p_bytes.len());
        let literal = &pattern[p_idx..literal_end];

        // Search for literal delimiter in remaining URL
        let match_pos = url[u_idx..].find(literal)?;
        let slot = &url[u_idx..u_idx + match_pos];
        slots.push(slot);
        u_idx += match_pos + literal.len();
        p_idx += literal.len();
      } else {
        // Match literal character
        if u_idx >= u_bytes.len() || p_bytes[p_idx] != u_bytes[u_idx] {
          return None;
        }
        p_idx += 1;
        u_idx += 1;
      }
    }

    if u_idx == u_bytes.len() {
      Some(slots)
    } else {
      None
    }
  }

  /// Pack template ID and extracted slot strings into a compact byte payload.
  pub fn pack(&self, id: u8, slots: &[&str]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(id);
    buf.push(slots.len() as u8);

    for slot in slots {
      let bytes = slot.as_bytes();
      if bytes.len() < 128 {
        buf.push(bytes.len() as u8);
      } else {
        // 2-byte length encoding
        buf.push(0x80 | ((bytes.len() >> 8) as u8 & 0x7F));
        buf.push((bytes.len() & 0xFF) as u8);
      }
      buf.extend_from_slice(bytes);
    }

    buf
  }

  /// Unpack byte payload into template ID and slot strings.
  pub fn unpack(&self, bytes: &[u8]) -> Result<(u8, Vec<String>)> {
    if bytes.len() < 2 {
      return Err(Error::Codec("template payload truncated".to_string()));
    }

    let id = bytes[0];
    let count = bytes[1] as usize;
    let mut slots = Vec::with_capacity(count);
    let mut offset = 2;

    for _ in 0..count {
      if offset >= bytes.len() {
        return Err(Error::Codec("slot length header truncated".to_string()));
      }

      let len = if bytes[offset] & 0x80 == 0 {
        let l = bytes[offset] as usize;
        offset += 1;
        l
      } else {
        if offset + 1 >= bytes.len() {
          return Err(Error::Codec("slot 2-byte length truncated".to_string()));
        }
        let l = (((bytes[offset] & 0x7F) as usize) << 8) | (bytes[offset + 1] as usize);
        offset += 2;
        l
      };

      if offset + len > bytes.len() {
        return Err(Error::Codec("slot data buffer truncated".to_string()));
      }

      let slot_str = std::str::from_utf8(&bytes[offset..offset + len])
        .map_err(|e| Error::Codec(format!("invalid utf-8 slot: {e}")))?
        .to_string();
      slots.push(slot_str);
      offset += len;
    }

    Ok((id, slots))
  }

  /// Reconstruct original URL from template ID and slot strings.
  pub fn expand(&self, id: u8, slots: &[String]) -> Result<String> {
    let schema = SCHEMAS
      .iter()
      .find(|s| s.id == id)
      .ok_or_else(|| Error::Codec(format!("unknown schema template id {id}")))?;

    if slots.len() != schema.slots {
      return Err(Error::Codec(format!(
        "slot count mismatch: expected {}, got {}",
        schema.slots,
        slots.len()
      )));
    }

    let mut out = schema.pattern.to_string();
    for (i, slot) in slots.iter().enumerate() {
      let placeholder = format!("{{{i}}}");
      out = out.replace(&placeholder, slot);
    }

    Ok(out)
  }
}
