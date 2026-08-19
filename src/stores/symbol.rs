/// Fast Static Symbol Table (FSST) for compressing anchor records and small strings.
#[derive(Debug, Clone)]
pub struct Symbol {
  symbols: Vec<&'static [u8]>,
}

impl Default for Symbol {
  fn default() -> Self {
    Self::new()
  }
}

impl Symbol {
  pub fn new() -> Self {
    let symbols: Vec<&'static [u8]> = vec![
      b"https://www.",
      b"https://",
      b"http://www.",
      b"http://",
      b"github.com/",
      b"amazon.com/dp/",
      b"store.steampowered.com/",
      b"instagram.com/p/",
      b"reddit.com/r/",
      b"linkedin.com/posts/",
      b"wikipedia.org/wiki/",
      b"youtube.com/watch?v=",
      b"crates.io/crates/",
      b"doc.rust-lang.org/",
      b"?utm_source=",
      b"&utm_medium=",
      b"&utm_campaign=",
      b"&gclid=",
      b"&fbclid=",
      b"?ref=",
      b"?igshid=",
      b"/status/",
      b"/comments/",
      b"/compiler/",
      b"/blob/master/",
      b".html",
      b".org",
      b".com",
      b".io",
      b"ipfs://",
      b"magnet:?xt=urn:btih:",
      b"mailto:",
      b"bitcoin:",
      b"0IL6S",
      b"0CbWV",
      b"0BXdx",
      b"0ARjp",
      b"0Acons",
      b"0ODif",
      b"0Cbt0",
      b"0HHnY",
      b"0BFVy",
      b"0GLds",
      b"0CBIK",
      b"0ymfp",
      b"0irBQ",
    ];

    Self { symbols }
  }

  /// Compress string by replacing recognized n-grams with 1-byte symbol codes.
  pub fn compress(&self, input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
      let mut matched = false;

      // Greedy longest symbol match (tokens in table are indexed 128..255)
      for (idx, &sym) in self.symbols.iter().enumerate() {
        if input[i..].starts_with(sym) {
          // Escape byte 0xFF prefix followed by symbol index
          out.push(0xFF);
          out.push(idx as u8);
          i += sym.len();
          matched = true;
          break;
        }
      }

      if !matched {
        if input[i] == 0xFF {
          // Literal 0xFF escaped as [0xFF, 0xFE]
          out.push(0xFF);
          out.push(0xFE);
        } else {
          out.push(input[i]);
        }
        i += 1;
      }
    }

    out
  }

  /// Decompress symbol stream back into raw byte string.
  pub fn decompress(&self, input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() * 2);
    let mut i = 0;

    while i < input.len() {
      if input[i] == 0xFF && i + 1 < input.len() {
        let code = input[i + 1];
        if code == 0xFE {
          out.push(0xFF);
        } else if (code as usize) < self.symbols.len() {
          out.extend_from_slice(self.symbols[code as usize]);
        }
        i += 2;
      } else {
        out.push(input[i]);
        i += 1;
      }
    }

    out
  }
}
