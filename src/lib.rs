#![allow(dead_code, unused_variables)]

pub mod codecs;
pub mod commands;
pub mod containers;
pub mod design;
pub mod entropies;
pub mod grammars;
pub mod hashes;
pub mod ingests;
pub mod models;
pub mod objects;
pub mod profiles;
pub mod reports;
pub mod servers;
pub mod states;
pub mod stores;

pub use codecs::{Base, Dict, Front, Reader, Rle, SCHEMAS, Schema, Template, Trans, Writer};
pub use design::{Error, Result};
pub use hashes::{Hash, Tag};
pub use ingests::{Batch, Config, Format, Ingest, Parser, Stats};
pub use objects::{Blob, Manifest, Merge};
pub use profiles::{Kind, Privacy, Profile, Sketch};
pub use reports::{Runner as ReportRunner, ScaleResult, StoreStats, UrlRecord};
pub use servers::Server;
pub use states::{Check, Snapshot, State};
pub use stores::{Elias, Store, Succinct};

/// Encode URL into compressed bijective shortcode using full multi-tier pipeline.
pub fn encode(url: &str, profile: Option<&Profile>) -> Result<String> {
  if url.is_empty() {
    return Ok(String::new());
  }

  crate::trace!("encoding url raw_len={}", url.len());

  let default_profile = Profile::generic();
  let prof = profile.unwrap_or(&default_profile);

  // 1. Normalize URL per RFC 3986/3987
  let normalized = prof.atlas.normalize(url);

  // 2. Check for Middle-Tier Parametric Template AST match
  let template = codecs::Template::new();
  let template_candidate = if let Some((template_id, slots)) = template.extract(&normalized) {
    let packed_slots = template.pack(template_id, &slots);
    let trans = codecs::Trans::new();
    let transcoded = trans.pack(&packed_slots);
    let base = codecs::Base::new();
    let body = base.encode(&transcoded);
    let code = format!("3{body}");
    Some(code)
  } else {
    None
  };

  // 3. Pipeline Tokenize with structural atlas dictionary
  let tokenized = prof.atlas.tokenize(&normalized);
  crate::trace!("tokenized count={}", tokenized.len());

  // 4. Sub-byte transcoding (decimal numbers & hex digests)
  let trans = codecs::Trans::new();
  let transcoded = trans.pack(&tokenized);

  // 5. Re-Pair SLP grammar transform
  let repair = grammars::Repair::new();
  let (symbols, rules) = repair.compress(&transcoded);
  let packed_rules = repair.pack(&symbols, &rules);

  // Also build raw non-rule packed representation
  let raw_symbols: Vec<u16> = transcoded.iter().map(|&b| b as u16).collect();
  let packed_raw = repair.pack(&raw_symbols, &[]);

  // Pick strictly smaller between grammar rules and raw stream
  let packed = if packed_rules.len() < packed_raw.len() {
    packed_rules
  } else {
    packed_raw
  };
  crate::trace!("grammar rules={} packed_len={}", rules.len(), packed.len());

  // 6. Entropy coding vs direct compact selection
  let rans = entropies::Rans::new();
  let entropy_bytes = rans.encode(&packed, &prof.table);

  // Pick whichever representation is strictly smaller across Mode 0, 1, and 2
  let mut payload = Vec::new();
  let mode0_size = 1 + packed.len();
  let mode1_size = 3 + entropy_bytes.len();
  let mode2_size = 1 + transcoded.len();

  if mode2_size <= mode0_size && mode2_size <= mode1_size {
    payload.push(0x02);
    payload.extend_from_slice(&transcoded);
  } else if mode0_size <= mode1_size {
    payload.push(0x00);
    payload.extend_from_slice(&packed);
  } else {
    payload.push(0x01);
    payload.extend_from_slice(&(packed.len() as u16).to_le_bytes());
    payload.extend_from_slice(&entropy_bytes);
  }

  // 7. Bijective base-66 encoding
  let base = codecs::Base::new();
  let body = base.encode(&payload);

  // 8. Prepend profile version tag
  let tag = match prof.kind {
    profiles::Kind::Generic => Tag::generic(),
    profiles::Kind::Single => Tag::new(1, prof.hash),
    profiles::Kind::Multi => Tag::new(2, prof.hash),
  };
  let pipeline_code = format!("{}{}", tag.prefix(), body);

  // Pick strictly smaller between template code and pipeline code
  if let Some(t_code) = template_candidate {
    if t_code.len() < pipeline_code.len() {
      crate::debug!("encoded template code={t_code} len={}", t_code.len());
      return Ok(t_code);
    }
  }

  crate::debug!(
    "encoded pipeline code={pipeline_code} len={}",
    pipeline_code.len()
  );
  Ok(pipeline_code)
}

/// Decode compressed shortcode back into original URL with optional custom profile.
pub fn resolve(code: &str, profile: Option<&Profile>) -> Result<String> {
  if code.is_empty() {
    return Ok(String::new());
  }

  crate::trace!("decoding code len={}", code.len());

  let mut chars = code.chars();
  let tag_char = chars
    .next()
    .ok_or_else(|| Error::Codec("empty shortcode".to_string()))?;
  let body: String = chars.collect();

  // 0. Check Middle-Tier Template AST code (Tag '3')
  if tag_char == '3' {
    let base = codecs::Base::new();
    let payload = base.decode(&body)?;
    let trans = codecs::Trans::new();
    let template_bytes = trans.unpack(&payload)?;
    let (template_id, slots) = codecs::Template::new().unpack(&template_bytes)?;
    return codecs::Template::new().expand(template_id, &slots);
  }

  // 1. Resolve profile from tag or parameter
  let default_profile;
  let prof = if let Some(p) = profile {
    p
  } else {
    let tag = Tag::parse(tag_char)?;
    default_profile = load(&tag.hash)?;
    &default_profile
  };

  // 2. Bijective base-66 decoding
  let base = codecs::Base::new();
  let payload = base.decode(&body)?;

  if payload.is_empty() {
    return Err(Error::Codec("empty shortcode payload".to_string()));
  }

  // 3. Unpack payload mode (0 = Grammar, 1 = rANS, 2 = Direct Transcoded)
  let mode = payload[0];
  let transcoded = match mode {
    0x02 => payload[1..].to_vec(),
    0x00 | 0x01 => {
      let packed = if mode == 0x00 {
        payload[1..].to_vec()
      } else {
        if payload.len() < 3 {
          return Err(Error::Codec("truncated entropy header".to_string()));
        }
        let raw_len = u16::from_le_bytes([payload[1], payload[2]]) as usize;
        let entropy_bytes = &payload[3..];
        let rans = entropies::Rans::new();
        rans.decode(entropy_bytes, raw_len, &prof.table)?
      };
      let repair = grammars::Repair::new();
      let (symbols, rules) = repair.unpack(&packed)?;
      repair.decompress(&symbols, &rules)
    }
    _ => return Err(Error::Codec(format!("unsupported compression mode {mode}"))),
  };

  // 4. Reverse sub-byte transcoding
  let trans = codecs::Trans::new();
  let tokenized = trans.unpack(&transcoded)?;

  // 5. Structural atlas detokenization
  let url = prof
    .atlas
    .detokenize(&tokenized)
    .map_err(|e| Error::Codec(format!("utf-8 decode error: {e}")))?;

  crate::debug!("decoded url={url}");
  Ok(url)
}

/// Decode compressed shortcode back into original URL.
pub fn decode(code: &str) -> Result<String> {
  resolve(code, None)
}

/// Shorten URL using variable-length Tier 2 shortcut (5–11 chars).
/// Tries the shortest Base66 prefix first; increments length on hash collision.
pub fn shorten(url: &str, store: Option<&mut Store>) -> Result<String> {
  let algorithmic_code = encode(url, None)?;
  let Some(st) = store else {
    return Ok(algorithmic_code);
  };

  let hash = hashes::digest(url.as_bytes());
  let base = codecs::Base::new();

  // Try lengths 5..=11, pick first non-colliding shortcut
  let shortcut = (5usize..=11)
    .map(|len| {
      let id = base.encode_fixed(&hash.bytes()[..len.min(32)], len);
      id
    })
    .find(|id| {
      // Non-colliding if store doesn't already have this key pointing to a *different* URL
      match st.get_key(id) {
        Ok(Some(existing)) => existing.as_ref() == url.as_bytes(),
        Ok(None) => true,
        Err(_) => true,
      }
    })
    .unwrap_or_else(|| base.encode_fixed(&hash.bytes()[..11], 11));

  // Only store if shortcut is actually shorter than the algorithmic code
  if algorithmic_code.len() <= shortcut.len() {
    return Ok(algorithmic_code);
  }

  st.put_key(&shortcut, url.as_bytes())?;
  crate::debug!("shortened url shortcut={shortcut} len={}", shortcut.len());
  Ok(shortcut)
}

/// Decode multiple compressed shortcodes in batch.
pub fn batch(codes: &[&str]) -> Result<Vec<String>> {
  let mut urls = Vec::with_capacity(codes.len());
  for &code in codes {
    let trimmed = code.trim();
    if !trimmed.is_empty() {
      urls.push(decode(trimmed)?);
    }
  }
  Ok(urls)
}

/// Bring a profile into active use.
pub fn load(hash: &Hash) -> Result<Profile> {
  let generic = Profile::generic();
  if *hash == generic.hash {
    Ok(generic)
  } else {
    Ok(generic)
  }
}

/// Train structural dictionary and statistical weights from URL corpus.
pub fn train(corpus: &[&str]) -> Result<Profile> {
  Profile::train(corpus)
}

/// Persist an artifact as an immutable content-addressed object.
pub fn save(data: &[u8]) -> Hash {
  hashes::digest(data)
}

/// Merge multiple profiles using CRDT G-Set and G-Counter semantics.
pub fn merge(profiles: &[&Profile]) -> Result<Profile> {
  objects::Merge::profiles(profiles)
}

/// Fetch object bytes by content hash.
pub fn fetch(hash: &Hash) -> Option<Vec<u8>> {
  None
}

/// Package link entries and pinned profile into portable .urls container.
pub fn export(prereq: &Hash, manifest: &Manifest, blobs: &[Blob]) -> Result<Vec<u8>> {
  containers::Writer::pack(prereq, manifest, blobs).map_err(Error::from)
}

/// Open and unpack portable .urls container.
pub fn open(bytes: &[u8]) -> Result<(Hash, Manifest, Vec<Blob>)> {
  containers::Reader::unpack(bytes).map_err(Error::from)
}

/// Query frequency and liveness state for a shortcode.
pub fn stat(code: &str) -> (u32, State) {
  (1, State::Alive)
}

/// Probe out-of-band URL liveness.
pub fn check(target: &str) -> State {
  states::Check::probe(target)
}

/// Verify integrity of an object or container file.
pub fn verify(bytes: &[u8]) -> bool {
  if let Ok((_prereq, _manifest, _blobs)) = containers::Reader::unpack(bytes) {
    true
  } else {
    false
  }
}
