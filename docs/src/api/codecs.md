# Rust API: Compression Codecs

Detailed reference for algorithmic codecs residing in `urls::codecs`.

---

## 1. Bit-Parallel Myers Positional Delta (`Delta`)

```rust
use urls::codecs::Delta;

let delta_codec = Delta::new();

let base = "https://shop.google.com/items/details";
let target = "https://shop.google.com/items/pricing";

// Compute 128-bit match bitmask and compact byte substitution delta
let diff = delta_codec.encode(base, target);

// Decompress target purely from base + diff stream
let reconstructed = delta_codec.decode(base, &diff)?;
assert_eq!(reconstructed, target);
```

---

## 2. Tagged Run-Length Encoding (`Rle`)

```rust
use urls::codecs::Rle;

let raw_data = b"https://api.domain.com/search?q=test0000000000000000";

// Packs runs of repeating bytes (>3 repeats) with tagged headers
let packed = Rle::pack(raw_data);

// Decompress with zero expansion on uncompressed streams
let unpacked = Rle::unpack(&packed)?;
assert_eq!(unpacked, raw_data);
```

---

## 3. Fast Static Symbol Tables (`Fsst`)

```rust
use urls::codecs::Fsst;

let corpus = vec![
  "https://shop.google.com/item/1".to_string(),
  "https://shop.google.com/item/2".to_string(),
];

// Train 256-symbol static dictionary from sample strings
let table = Fsst::train(&corpus);

let compressed = table.compress("https://shop.google.com/item/1");
let decompressed = table.decompress(&compressed);
assert_eq!(decompressed, "https://shop.google.com/item/1");
```

---

## 4. Range Asymmetric Numeral Systems (`Rans`)

```rust
use urls::codecs::{FreqTable, Rans};

let freqs = FreqTable::build(b"https://example.com/stream");
let rans = Rans::new(freqs);

let encoded = rans.encode(b"https://example.com/stream");
let decoded = rans.decode(&encoded)?;
assert_eq!(decoded, b"https://example.com/stream");
```
