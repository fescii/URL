use super::rule::Rule;
use crate::design::{Error, Result};
use std::collections::HashMap;

/// Re-Pair straight-line program (SLP) grammar transform engine.
pub struct Repair;

impl Repair {
  pub const fn new() -> Self {
    Self
  }

  /// Compress input bytes into a symbol sequence and a set of substitution rules.
  pub fn compress(&self, input: &[u8]) -> (Vec<u16>, Vec<Rule>) {
    if input.len() < 8 {
      let symbols = input.iter().map(|&b| b as u16).collect();
      return (symbols, Vec::new());
    }

    let mut symbols: Vec<u16> = input.iter().map(|&b| b as u16).collect();
    let mut rules = Vec::new();
    let mut next_id: u16 = 256;

    loop {
      if symbols.len() < 4 || next_id >= u16::MAX - 1 {
        break;
      }

      // Count adjacent pairs
      let mut counts: HashMap<(u16, u16), usize> = HashMap::new();
      let mut i = 0;
      while i + 1 < symbols.len() {
        let pair = (symbols[i], symbols[i + 1]);
        *counts.entry(pair).or_insert(0) += 1;
        i += 1;
      }

      let mut best_pair = None;
      let mut max_count = 1;
      for (pair, count) in counts {
        if count > max_count || (count == max_count && best_pair.map_or(true, |bp| pair < bp)) {
          max_count = count;
          best_pair = Some(pair);
        }
      }

      let Some((left, right)) = best_pair else {
        break;
      };

      let rule_id = next_id;
      next_id += 1;
      rules.push(Rule::new(rule_id, left, right));

      // Replace all non-overlapping occurrences of (left, right) with rule_id
      let mut new_symbols = Vec::with_capacity(symbols.len());
      let mut idx = 0;
      while idx < symbols.len() {
        if idx + 1 < symbols.len() && symbols[idx] == left && symbols[idx + 1] == right {
          new_symbols.push(rule_id);
          idx += 2;
        } else {
          new_symbols.push(symbols[idx]);
          idx += 1;
        }
      }
      symbols = new_symbols;
    }

    (symbols, rules)
  }

  /// Decompress symbol sequence and rules back into original raw bytes.
  pub fn decompress(&self, symbols: &[u16], rules: &[Rule]) -> Vec<u8> {
    if rules.is_empty() {
      return symbols.iter().map(|&s| s as u8).collect();
    }

    let mut rule_map: HashMap<u16, (u16, u16)> = HashMap::new();
    for rule in rules {
      rule_map.insert(rule.id, (rule.left, rule.right));
    }

    let mut result = Vec::new();
    for &sym in symbols {
      self.expand(sym, &rule_map, &mut result);
    }
    result
  }

  fn expand(&self, sym: u16, map: &HashMap<u16, (u16, u16)>, out: &mut Vec<u8>) {
    if sym < 256 {
      out.push(sym as u8);
    } else if let Some(&(left, right)) = map.get(&sym) {
      self.expand(left, map, out);
      self.expand(right, map, out);
    }
  }

  /// Compactly serialize symbols and grammar rules into a byte buffer.
  pub fn pack(&self, symbols: &[u16], rules: &[Rule]) -> Vec<u8> {
    let mut buf = Vec::new();
    if rules.is_empty() && symbols.iter().all(|&s| s < 256) {
      // Mode 0: Raw byte stream (zero overhead)
      buf.push(0x00);
      for &sym in symbols {
        buf.push(sym as u8);
      }
    } else {
      // Mode 1: SLP Grammar rules table
      buf.push(0x01);
      buf.extend_from_slice(&(rules.len() as u16).to_le_bytes());
      for rule in rules {
        buf.extend_from_slice(&rule.id.to_le_bytes());
        buf.extend_from_slice(&rule.left.to_le_bytes());
        buf.extend_from_slice(&rule.right.to_le_bytes());
      }
      buf.extend_from_slice(&(symbols.len() as u32).to_le_bytes());
      for &sym in symbols {
        buf.extend_from_slice(&sym.to_le_bytes());
      }
    }
    buf
  }

  /// Deserializes packed byte buffer into symbols and grammar rules.
  pub fn unpack(&self, bytes: &[u8]) -> Result<(Vec<u16>, Vec<Rule>)> {
    if bytes.is_empty() {
      return Ok((Vec::new(), Vec::new()));
    }

    let mode = bytes[0];
    if mode == 0x00 {
      // Mode 0: Raw byte stream
      let symbols = bytes[1..].iter().map(|&b| b as u16).collect();
      return Ok((symbols, Vec::new()));
    }

    if bytes.len() < 7 {
      return Err(Error::Grammar("grammar buffer truncated".to_string()));
    }

    let mut offset = 1;
    let rule_count = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
    offset += 2;

    let mut rules = Vec::with_capacity(rule_count);
    for _ in 0..rule_count {
      if offset + 6 > bytes.len() {
        return Err(Error::Grammar("rule data truncated".to_string()));
      }
      let id = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
      let left = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
      let right = u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]);
      offset += 6;
      rules.push(Rule::new(id, left, right));
    }

    if offset + 4 > bytes.len() {
      return Err(Error::Grammar("symbol length truncated".to_string()));
    }
    let sym_count = u32::from_le_bytes([
      bytes[offset],
      bytes[offset + 1],
      bytes[offset + 2],
      bytes[offset + 3],
    ]) as usize;
    offset += 4;

    if offset + sym_count * 2 > bytes.len() {
      return Err(Error::Grammar("symbol buffer truncated".to_string()));
    }

    let mut symbols = Vec::with_capacity(sym_count);
    for _ in 0..sym_count {
      let sym = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
      offset += 2;
      symbols.push(sym);
    }

    Ok((symbols, rules))
  }
}
