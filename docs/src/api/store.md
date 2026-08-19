# Rust API: Sharded Storage & Bitcask

The storage subsystem (`urls::stores`) coordinates 4-way sharded Bitcask append logs, memory-mapped I/O, and Adaptive Replacement Caching (ARC).

---

## The `Store` Coordinator

```rust
use urls::stores::Store;
use bytes::Bytes;

let mut store = Store::open(".urls_store")?;

// Insert key and value bytes
store.put_key("Urh7.", b"https://shop.google.com/products/deals")?;

// Zero-copy lookup returning bytes::Bytes
let value: Option<Bytes> = store.get_key("Urh7.")?;
if let Some(bytes) = value {
  println!("Found URL: {}", std::str::from_utf8(&bytes)?);
}

// Seal active open-addressing HashMaps into succinct MPHF bitvectors
store.seal();
```

---

## Memory-Mapped Log Reader (`Log`)

```rust
use urls::stores::Log;
use bytes::Bytes;

let mut log = Log::open(".urls_store/shard_0.log")?;

// Zero-copy slice directly from memory map
let payload: Bytes = log.read_slice(1024, 64)?;
assert_eq!(payload.len(), 64);
```

---

## Adaptive Replacement Cache (`Cache`)

```rust
use urls::stores::Cache;
use bytes::Bytes;

let mut cache = Cache::new(50_000);

let key = urls::stores::digest(b"test_key");
cache.put(key, Bytes::from_static(b"https://example.com/cached"));

let hit: Option<Bytes> = cache.get(&key);
assert!(hit.is_some());
```
