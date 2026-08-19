# Rust Library API: Overview

`urls` is available as a native Rust crate providing high-performance functions and structs for embedding directly in your applications.

---

## Adding to `Cargo.toml`

```toml
[dependencies]
urls = { git = "https://github.com/fescii/URL.git" }
bytes = "1"
```

---

## Top-Level Function Exports

```rust
use urls::{encode, decode, shorten, expand, train, resolve, Store, Profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // 1. Stateless Algorithmic Encode / Decode
  let code = encode("https://shop.google.com/deals", None)?;
  let url = decode(&code)?;
  assert_eq!(url, "https://shop.google.com/deals");

  // 2. Stateful Sharded Shortening & Expansion
  let mut store = Store::open(".urls_store")?;
  let key = shorten("https://shop.google.com/deals", &mut store)?;
  let resolved_bytes = expand(&key, &store)?;
  assert_eq!(resolved_bytes.as_ref(), b"https://shop.google.com/deals");

  Ok(())
}
```
