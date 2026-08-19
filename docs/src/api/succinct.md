# Rust API: Succinct MPHF Indexing

The succinct module (`urls::stores::Succinct`) implements collision-free Minimal Perfect Hash Functions (MPHF) and Elias-Fano monotone sequence encoding for ultra-low RAM footprint.

---

## Building a Succinct Index

```rust
use urls::stores::{Succinct, digest, Hash};

let mut entries = Vec::new();
for i in 0..10_000 {
  let key = digest(format!("key_{i}").as_bytes());
  let log_offset = (i * 128) as u64;
  entries.push((key, log_offset));
}

// Build 2-level MPHF bitvectors with SplitMix64 pilot seeds
let succinct = Succinct::build(entries);

// $O(1)$ query returning exact log offset
let target_key = digest(b"key_42");
let offset: Option<u64> = succinct.query(&target_key);
assert_eq!(offset, Some(42 * 128));

// RAM Footprint per key: ~3.0 - 5.9 Bytes
let ram_per_key = succinct.memory_size() as f64 / succinct.len() as f64;
println!("RAM usage: {:.2} Bytes / Key", ram_per_key);
```

---

## Elias-Fano Monotone Sequence (`Elias`)

```rust
use urls::stores::Elias;

let offsets = vec![0, 128, 256, 512, 1024, 2048, 4096];
let elias = Elias::build(&offsets);

assert_eq!(elias.get(3), 512);
assert_eq!(elias.len(), offsets.len());
```
