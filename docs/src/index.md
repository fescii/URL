# Introduction to URLs

> **Modern URLs are obnoxious.** They look like someone threw a dictionary at a query parameter, attached 14 UTM trackers, and set fire to your clipboard.
> 
> `urls` is an aggressive URL shrink ray written in Rust. It crushes bloated web links into 5-character nuggets without a database if you don't want one, or stuffs **4.3 million URLs into less RAM than a single browser tab** (literally ~14 MB) if you do.

---

## Why URLs?

Traditional link shorteners make a classic trade-off:
- They require a heavy database (Postgres, Redis, DynamoDB).
- Every redirect causes a network query or cache lookup.
- Memory usage grows uncontrollably with millions of stored keys.

`urls` is designed from first principles with two distinct paradigms:

1. **Stateless Algorithmic Compression (Tier 1 / 1.5)**:
   - Encodes raw URL strings into standalone mathematical shortcodes using bit-parallel positional deltas and structural symbol dictionaries.
   - Requires **zero databases, zero memory lookup tables, and zero state**.
   - Decompresses in microseconds on any CPU.

2. **Succinct Memory-Mapped Storage (Tier 2)**:
   - Generates ultra-short **5 to 9 character** shortcodes (`Urh7.`).
   - Uses a sharded, append-only Bitcask log where lookups slice zero-copy `bytes::Bytes` directly from memory-mapped disk pages.
   - Employs Minimal Perfect Hash Functions (MPHF) to achieve **3.62 Bytes per key RAM footprint** at 4.31M scale.

---

## Key Highlights

- **4.31M URLs in 14.9 MB RAM**: $>85\%$ RAM reduction compared to standard hash maps.
- **Microsecond Lookups**: Warm point queries execute in **~3 µs**, cold reads in **< 308 µs**.
- **Zero-Copy Pipeline**: No intermediate heap allocations on read/cache paths.
- **100% Lossless Verification**: Every single character, parameter, and fragment is preserved byte-for-byte.
