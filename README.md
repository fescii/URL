# URLs

A high-performance, zero-copy, multi-tier URL compression and succinct storage engine built with Rust.

`urls` compacts web identifiers and massive URL corpora into ultra-short codes and minimal memory structures using layered algorithmic codecs, bit-parallel positional deltas, and minimal perfect hash function (MPHF) indexing.

---

## Features

- **Multi-Tier Compression Pipeline**:
  - **Tier 1 (Algorithmic / Stateless)**: Myers bit-parallel positional deltas against structural centroid anchors.
  - **Tier 1.5 (Entropy / Structural)**: Fast Static Symbol Tables (FSST), Re-Pair grammar compaction, and Asymmetric Numeral Systems (rANS).
  - **Tier 2 (Succinct Sharded Store)**: Dynamic 5–11 character shortcodes (`0-9A-Za-z.~-_`) providing up to **-98.0% size reduction** per URL string.
- **Succinct MPHF In-Memory Index**:
  - Compacts 4.31+ million URLs into **14.9 MB of RAM** (**3.62 Bytes / Key**), delivering an **>85% RAM reduction** compared to traditional hash tables.
- **Zero-Copy Memory-Mapped Storage**:
  - Sharded Bitcask append-only data logs with memory-mapped readers (`Mmap`) returning zero-copy `bytes::Bytes` slices directly from disk cache.
  - Zero heap allocations on the point query read path.
- **Adaptive Run-Length Encoding & Front-Coding**:
  - Deduplicates repetitive query parameter sequences, long alphanumeric runs, and sorted chunk prefixes without expansion overhead.
- **High-Throughput CLI & HTTP Server**:
  - Batch ingestion and reporting sustaining **>2,200 URLs/sec** with sub-microsecond in-memory lookups.

---

## Benchmark Results (1K → 4.31M URLs)

Benchmarked on real-world web corpora (`list.csv`) across 10 distinct scale increments:

| Scale (URLs) | Ingested Raw | Stored Disk Footprint | Disk Savings | Index RAM (Active) | Index RAM (Sealed MPHF) | Ingestion Rate | Lookup Latency |
|:---|:---|:---|:---:|:---|:---|:---|:---|
| **1,000** | 183.95 KB | 150.84 KB | **18.00%** | 70.31 KB (72.0 B/key) | 11.63 KB (**11.91 B/key**) | 1,209 URLs/s | 6.75 µs |
| **5,000** | 931.84 KB | 819.36 KB | **12.07%** | 292.53 KB (59.9 B/key) | 46.47 KB (**9.52 B/key**) | 2,094 URLs/s | 2.91 µs |
| **10,000** | 1.82 MB | 1.61 MB | **11.13%** | 397.90 KB (40.7 B/key) | 58.15 KB (**5.95 B/key**) | 2,102 URLs/s | 5.65 µs |
| **50,000** | 9.09 MB | 8.16 MB | **10.27%** | 2.80 MB (58.8 B/key) | 465.22 KB (**9.53 B/key**) | 2,241 URLs/s | 7.91 µs |
| **100,000** | 18.23 MB | 16.35 MB | **10.27%** | 3.89 MB (40.7 B/key) | 581.55 KB (**5.96 B/key**) | 2,292 URLs/s | 3.29 µs |
| **500,000** | 91.50 MB | 82.23 MB | **10.13%** | 28.01 MB (58.8 B/key) | 4.54 MB (**9.53 B/key**) | 2,286 URLs/s | 3.35 µs |
| **1,000,000** | 183.23 MB | 164.72 MB | **10.10%** | 38.85 MB (40.7 B/key) | 5.68 MB (**5.95 B/key**) | 2,309 URLs/s | 17.08 µs |
| **2,000,000** | 367.21 MB | 330.24 MB | **10.07%** | 74.29 MB (38.9 B/key) | 11.36 MB (**5.95 B/key**) | 2,302 URLs/s | 3.17 µs |
| **3,000,000** | 551.62 MB | 496.13 MB | **10.06%** | 79.97 MB (28.0 B/key) | 11.36 MB (**3.97 B/key**) | 2,266 URLs/s | 3.06 µs |
| **4,313,006** | 793.83 MB | 713.97 MB | **10.06%** | 101.44 MB (24.7 B/key) | 14.91 MB (**3.62 B/key**) | 2,224 URLs/s | 111.93 µs |

Detailed per-scale reports and measurement datasets are available in `reports/` (e.g. `reports/1000/`, `reports/4313006/`, `reports/summary.md`, and `reports/summary.csv`).

---

## Installation & Build

Ensure you have Rust (stable 1.80+) installed.

```bash
git clone https://github.com/your-org/urls.git
cd urls
cargo build --release
```

---

## CLI Usage

### 1. Stateless Encoding & Decoding (Tier 1 / 1.5)

```bash
# Encode URL into stateless algorithmic representation
cargo run --release -- encode "https://shop.google.com/products/deals?id=123"

# Decode stateless shortcode back to original URL
cargo run --release -- decode "0CEbn5mgfiOdL~..."
```

### 2. State-Backed Shortening & Retrieval (Tier 2)

```bash
# Generate compact 5-11 character shortcode stored in Bitcask database
cargo run --release -- shorten "https://shop.google.com/products/deals?id=123"

# Expand shortcode to raw URL
cargo run --release -- expand "Urh7."
```

### 3. Batch Corpus Ingestion

```bash
# Ingest CSV / TSV dataset into sharded storage and seal into succinct MPHF bitvectors
cargo run --release -- ingest data/list.csv --dir .urls_store
```

### 4. Running the HTTP Server

```bash
# Launch high-performance redirect and lookup HTTP service
cargo run --release -- serve --host 127.0.0.1 --port 8080 --store .urls_store
```

Endpoint examples:
- `GET /:key` → HTTP 301/302 redirect or raw URL payload
- `POST /shorten` → JSON payload `{"url": "https://..."}` returns `{"key": "Urh7."}`

### 5. Multi-Scale Reporting Suite

```bash
# Run comprehensive benchmark suite across customized scales
cargo run --release -- report --input data/list.csv --out reports --scales 1000,5000,10000,50000,100000,500000,1000000,2000000,3000000,4313006
```

---

## Architecture Overview

```
                   +-----------------------------------------------+
                   |                 Input URL                     |
                   +-----------------------+-----------------------+
                                           |
                                           v
                       +---------------------------------------+
                       |      Tier Selector & Router           |
                       +-------------------+-------------------+
                                           |
                +--------------------------+--------------------------+
                |                                                     |
                v                                                     v
   +---------------------------+                         +---------------------------+
   |   Stateless Path (T1/1.5) |                         |  Stateful Path (Tier 2)   |
   | - Myers Positional Delta  |                         | - 5-11 Char Base66 Code   |
   | - FSST Static Symbols     |                         | - 4-Way Sharded Bitcask   |
   | - Tagged Stream RLE       |                         | - Zero-Copy Mmap Logs     |
   | - rANS Entropy / Grammar  |                         | - Succinct MPHF Index     |
   +---------------------------+                         +---------------------------+
```

## Testing

Run the full suite of 55 unit and integration tests:

```bash
cargo test --release
```

---

## License

Apache-2.0 / MIT
