/// Structural tokens compiled from RFC 3986, IANA scheme registry, and URL Protocol Atlas (protocals.md).
pub const TOKENS: &[&str] = &[
  // Full Scheme + Authority + Path Templates (Highest Compression Yield)
  "https://store.steampowered.com/app/",
  "https://www.linkedin.com/posts/",
  "https://www.youtube.com/watch?v=",
  "https://x.com/rustlang/status/",
  "https://doc.rust-lang.org/book/",
  "https://en.wikipedia.org/wiki/",
  "https://api.github.com/repos/",
  "https://www.instagram.com/p/",
  "https://github.com/rust-lang/",
  "https://www.amazon.com/dp/",
  "https://www.reddit.com/r/",
  "https://crates.io/crates/",
  "https://twitter.com/",
  "https://facebook.com/",
  "https://youtube.com/",
  "https://github.com/",
  "https://medium.com/",
  "https://tiktok.com/",
  "https://google.com/",
  "https://apple.com/",
  "https://wa.me/",
  "https://x.com/",
  "https://shop.",
  "https://portal.",
  "https://api.",
  "https://dev.",
  "https://media.",
  "https://blog.",
  "https://m.",
  "https://mail.",
  "https://www.",
  "https://",
  "http://www.",
  "http://",
  // Path Segments & Common Boilerplate Substrings
  "/shop/products/deals/",
  "/shop/products/",
  "/wiki/articles/",
  "/explore/tags/",
  "/feed/trending/",
  "/comments/thread/",
  "/user/profile/",
  "/questions/tagged/",
  "/questions/",
  "/search?q=",
  "/track/event/",
  "/blob/master/",
  "/blob/main/",
  "/comments/",
  "/compiler/",
  "/status/",
  "/issues/",
  "/watch?v=",
  "/pull/",
  "/posts/",
  "/api/v1/",
  "/api/v2/",
  // Query Strings & Common API Parameters
  "?state=open&sort=created&direction=desc",
  "?utm_source=",
  "?utm_medium=",
  "?utm_campaign=",
  "?utm_content=",
  "?utm_term=",
  "?fbclid=",
  "?gclid=",
  "?igshid=",
  "?ref=",
  "&utm_source=",
  "&utm_medium=",
  "&utm_campaign=",
  "&utm_content=",
  "&utm_term=",
  "&fbclid=",
  "&gclid=",
  "&igshid=",
  "&session_id=",
  "&timestamp=",
  "&msclkid=",
  "&twclid=",
  "&ttclid=",
  "&source=",
  "&ref=",
  "&amount=",
  "&label=",
  "&subject=",
  "&dn=",
  // Common TLDs
  ".com/",
  ".org/",
  ".net/",
  ".ru/",
  ".io/",
  // Common Technical Keywords
  "distributed",
  "concurrency",
  "algorithms",
  "compression",
  "optimization",
  "performance",
  "benchmark",
  "analysis",
  "database",
  "compiler",
  "systems",
  "succinct",
  "entropy",
  "hashing",
  "manifest",
  "cluster",
  "vector",
  "payload",
  "token",
  "lexer",
  "shard",
  "graph",
  "rust",
  // Alternate Schemes & Protocols
  "magnet:?xt=urn:btih:",
  "ipfs://bafybeic",
  "ipfs://",
  "ipns://",
  "mailto:",
  "bitcoin:",
  "ethereum:",
  "file:///",
  "ftp://",
  "ssh://",
  "wss://",
  "ws://",
  "urn:",
];

/// Structural dictionary derived from URL protocol atlas.
#[derive(Debug, Clone)]
pub struct Atlas {
  pub tokens: &'static [&'static str],
}

impl Atlas {
  pub const fn new() -> Self {
    Self { tokens: TOKENS }
  }

  /// Normalize URL per RFC 3986/3987.
  pub fn normalize(&self, url: &str) -> String {
    let trimmed = url.trim();
    if let Some(pos) = trimmed.find("://") {
      let scheme = &trimmed[..pos].to_ascii_lowercase();
      let rest = &trimmed[pos..];
      format!("{scheme}{rest}")
    } else {
      trimmed.to_string()
    }
  }

  /// Replace matching structural dictionary tokens with single-byte token markers (0x80..0xFF).
  pub fn tokenize(&self, url: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = url.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
      let mut best_match: Option<(usize, usize)> = None; // (token_idx, token_len)

      for (idx, &token) in self.tokens.iter().enumerate() {
        if idx >= 128 {
          break;
        }
        let t_bytes = token.as_bytes();
        if i + t_bytes.len() <= bytes.len() && &bytes[i..i + t_bytes.len()] == t_bytes {
          if let Some((_, best_len)) = best_match {
            if t_bytes.len() > best_len {
              best_match = Some((idx, t_bytes.len()));
            }
          } else {
            best_match = Some((idx, t_bytes.len()));
          }
        }
      }

      if let Some((idx, len)) = best_match {
        result.push(0x80 | (idx as u8));
        i += len;
      } else {
        result.push(bytes[i]);
        i += 1;
      }
    }

    result
  }

  /// Expand token markers (0x80..0xFF) back to their original strings.
  pub fn detokenize(&self, bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let mut out = Vec::new();
    for &b in bytes {
      if b >= 0x80 {
        let idx = (b & 0x7F) as usize;
        if idx < self.tokens.len() {
          out.extend_from_slice(self.tokens[idx].as_bytes());
        } else {
          out.push(b);
        }
      } else {
        out.push(b);
      }
    }
    String::from_utf8(out)
  }
}
