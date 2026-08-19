use super::table::{SCALE, SCALE_BITS, Table};
use crate::design::{Error, Result};

const L: u32 = 1 << 16; // 65536

/// Range Asymmetric Numeral Systems (rANS) 32-bit entropy codec.
pub struct Rans;

impl Rans {
  pub const fn new() -> Self {
    Self
  }

  /// Encode symbols in reverse order into compressed byte buffer.
  pub fn encode(&self, symbols: &[u8], table: &Table) -> Vec<u8> {
    if symbols.is_empty() {
      return Vec::new();
    }

    let mut state = L;
    let mut out = Vec::new();

    // Encode in reverse order (LIFO) so decoder reads in forward order
    for &sym in symbols.iter().rev() {
      let s = sym as usize;
      let freq = table.freqs[s];
      let start = table.cumul[s];

      // Renormalize
      let max_state = ((L >> SCALE_BITS) << 8) * freq;
      while state >= max_state {
        out.push((state & 0xFF) as u8);
        state >>= 8;
      }

      // State update
      state = ((state / freq) << SCALE_BITS) + start + (state % freq);
    }

    // Emit final state (4 bytes)
    out.extend_from_slice(&state.to_le_bytes());
    out
  }

  /// Decode compressed bytes back into original symbol slice.
  pub fn decode(&self, bytes: &[u8], count: usize, table: &Table) -> Result<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }
    if bytes.len() < 4 {
      return Err(Error::Entropy("rANS stream truncated".to_string()));
    }

    let mut stream_idx = bytes.len() - 4;
    let mut state = u32::from_le_bytes([
      bytes[stream_idx],
      bytes[stream_idx + 1],
      bytes[stream_idx + 2],
      bytes[stream_idx + 3],
    ]);

    let mut symbols = Vec::with_capacity(count);

    for _ in 0..count {
      let slot = (state & (SCALE - 1)) as usize;
      let sym = table.slots[slot];
      let s = sym as usize;
      let freq = table.freqs[s];
      let start = table.cumul[s];

      symbols.push(sym);

      // State decode update
      state = freq * (state >> SCALE_BITS) + ((slot as u32) - start);

      // Renormalize
      while state < L && stream_idx > 0 {
        stream_idx -= 1;
        state = (state << 8) | (bytes[stream_idx] as u32);
      }
    }

    Ok(symbols)
  }
}
