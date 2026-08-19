/// Move-To-Front (MTF) rank transform for concentrating character and token distributions near zero.
#[derive(Debug, Default, Clone, Copy)]
pub struct Mtf;

impl Mtf {
  pub fn new() -> Self {
    Self
  }

  /// Transform raw byte slice into Move-To-Front rank sequence.
  pub fn encode(&self, input: &[u8]) -> Vec<u8> {
    let mut table = [0u8; 256];
    for i in 0..256 {
      table[i] = i as u8;
    }

    let mut output = Vec::with_capacity(input.len());

    for &byte in input {
      let mut idx = 0;
      while idx < 256 && table[idx] != byte {
        idx += 1;
      }

      output.push(idx as u8);

      // Move byte to the front of table
      table.copy_within(0..idx, 1);
      table[0] = byte;
    }

    output
  }

  /// Invert Move-To-Front rank sequence back into original byte slice.
  pub fn decode(&self, ranks: &[u8]) -> Vec<u8> {
    let mut table = [0u8; 256];
    for i in 0..256 {
      table[i] = i as u8;
    }

    let mut output = Vec::with_capacity(ranks.len());

    for &rank in ranks {
      let idx = rank as usize;
      let byte = table[idx];
      output.push(byte);

      // Move byte to the front of table
      table.copy_within(0..idx, 1);
      table[0] = byte;
    }

    output
  }
}
