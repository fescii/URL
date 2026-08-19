# URLs

> **Modern URLs are obnoxious.** They look like someone threw a dictionary at a query parameter, attached 14 UTM trackers, and set fire to your clipboard.
> 
> `urls` is an aggressive URL shrink ray written in Rust. It crushes bloated web links into 5-character nuggets without a database if you don't want one, or stuffs **4.3 million URLs into less RAM than a single browser tab** (literally ~14 MB) if you do.

---

## Live Production Service

The web application and telemetry dashboard are deployed live in production on Fly.io:

| Resource | URL | Description |
| :--- | :--- | :--- |
| **Interactive Dashboard** | [https://urls.aduki.pro](https://urls.aduki.pro) | Web UI, Single URL Shrink Ray, Batch Ingestion & PDF Export |
| **Telemetry Dashboard** | [https://urls.aduki.pro/stats](https://urls.aduki.pro/stats) | Live RAM, latency breakdown ($3.06\,\mu\text{s}$ $p50$), and shard diagnostics |
| **Health JSON Endpoint** | [https://urls.aduki.pro/health](https://urls.aduki.pro/health) | Lightweight health & engine readiness probe |

---

## What is this witchcraft?

Most URL shorteners cheat: they take a URL, throw an auto-incrementing ID into Postgres, slap Redis in front, and pray the server doesn't run out of memory when real traffic arrives.

`urls` does it differently:

- **The "Look Ma, No Database" Mode (Stateless)**:
  Squeezes monstrous 250-character links into self-contained algorithmic codes using bitmask positional deltas and structural grammars. No store, no lookups, no state. Just pure decompression on the fly.
- **The "Hold My RAM" Mode (Stateful Store)**:
  If you want ultra-tiny 5-character links (`058qV`), it stores them in an append-only log and indexes millions of entries with Minimal Perfect Hashing.
  - **4,313,006 URLs fit inside 14.9 MB of RAM**. That's **3.6 bytes per key**. Your smart fridge could index the top 5 million websites without flinching.
- **Direct Memory Reads**:
  When looking up links, bytes are sliced straight from memory-mapped disk logs without copying buffers or allocating heap memory.
- **Adaptive String Squashing**:
  Repetitive query strings, long IDs, and repeated paths get collapsed with tagged run-length and prefix deduping so you don't store the same `https://` a million times.

---

## Empirical Benchmarks (1K → 4.31M Scale)

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

---

## Live API Reference

All HTTP endpoints are available at `https://urls.aduki.pro` (or locally at `http://localhost:3000`):

### 1. Shorten URL (Stateful)
Generate a compact Base66 5-character key and index it with zero-copy mmap.

- **Endpoint**: `POST /api/shorten`
- **Request**:
  ```bash
  curl -s -X POST https://urls.aduki.pro/api/shorten \
    -H "Content-Type: application/json" \
    -d '{"url":"https://github.com/rust-lang/rust"}'
  ```
- **Response**:
  ```json
  {
    "key": "058qV",
    "url": "https://github.com/rust-lang/rust",
    "shortUrl": "https://urls.aduki.pro/058qV",
    "statelessCode": "058qV",
    "savedBytes": 5,
    "ratio": "84.8%"
  }
  ```

---

### 2. Follow Redirect
Resolves a 5-character shortcode directly to the destination URL.

- **Endpoint**: `GET /:key`
- **Request**:
  ```bash
  curl -sI https://urls.aduki.pro/058qV
  ```
- **Response**:
  ```http
  HTTP/2 302 Found
  location: https://github.com/rust-lang/rust
  ```

---

### 3. Streaming Batch Ingestion
Stream large datasets (CSV, TSV, TXT up to 100 MB) in chunked batches into sharded logs.

- **Endpoint**: `POST /api/batch`
- **Request**:
  ```bash
  curl -s -X POST https://urls.aduki.pro/api/batch \
    -H "Content-Type: application/json" \
    -d '{"urls":["https://github.com/rust-lang/rust", "https://news.ycombinator.com"]}'
  ```
- **Response**:
  ```json
  {
    "success": true,
    "count": 2,
    "durationMs": "1.45",
    "ratePerSec": 1379,
    "items": [
      {
        "url": "https://github.com/rust-lang/rust",
        "key": "058qV",
        "shortUrl": "https://urls.aduki.pro/058qV"
      }
    ],
    "stats": {
      "keys": 2,
      "ramBytes": 140,
      "diskBytes": 301,
      "bytesPerKey": 70.0
    }
  }
  ```

---

### 4. Algorithmic Codec (Stateless)
Encode and decode URLs algorithmically with zero database storage.

- **Endpoint**: `POST /api/encode`
- **Encode Request**:
  ```bash
  curl -s -X POST https://urls.aduki.pro/api/encode \
    -H "Content-Type: application/json" \
    -d '{"input":"https://github.com/rust-lang/rust","action":"encode"}'
  ```
- **Decode Request**:
  ```bash
  curl -s -X POST https://urls.aduki.pro/api/encode \
    -H "Content-Type: application/json" \
    -d '{"input":"058qV","action":"decode"}'
  ```

---

### 5. Live Memory & Shard Telemetry
Retrieve real-time RAM consumption, indexed key counts, and disk footprint.

- **Endpoint**: `GET /api/stats`
- **Request**:
  ```bash
  curl -s https://urls.aduki.pro/api/stats
  ```
- **Response**:
  ```json
  {
    "success": true,
    "keys": 4313006,
    "ramBytes": 15634432,
    "diskBytes": 748650496,
    "bytesPerKey": 3.62,
    "ramMb": "14.91",
    "diskMb": "713.97"
  }
  ```

---

### 6. Health Probe
Fast, unauthenticated health check probe.

- **Endpoint**: `GET /health`
- **Request**:
  ```bash
  curl -s https://urls.aduki.pro/health
  ```
- **Response**:
  ```json
  {
    "status": "ok",
    "engine": "ready",
    "version": "0.1.0-ffi",
    "keys": 4313006,
    "ramBytes": 15634432,
    "diskBytes": 748650496,
    "bytesPerKey": 3.62
  }
  ```

---

## Running Live Benchmark Tests

### 1. Pre-generated Test Datasets (5,000 URLs)
Test datasets extracted directly from `common.csv` & `list.csv` are located in `data/`:
- **CSV Dataset**: [data/urls_5000.csv](file:///home/femar/Downloads/URL/data/urls_5000.csv) (5,000 records with `id,domain,url`)
- **TSV Dataset**: [data/urls_5000.tsv](file:///home/femar/Downloads/URL/data/urls_5000.tsv) (5,000 tab-separated records)
- **TXT Dataset**: [data/urls_5000.txt](file:///home/femar/Downloads/URL/data/urls_5000.txt) (5,000 raw newline-delimited URLs)

To re-generate or modify the 5,000 test files:
```bash
python3 data/generate.py
```

### 2. Ingest via Web UI
1. Navigate to [https://urls.aduki.pro](https://urls.aduki.pro) (or `http://localhost:3000`).
2. Click on the **Batch Ingest** tab.
3. Upload `data/urls_5000.csv` (or `.tsv` / `.txt`).
4. Click **Stream Ingest Batch** to monitor live ingestion speed (`URLs/s`) and generate a printable PDF report upon completion.

### 3. Run Native Rust Test Suite
```bash
cargo test --release
```

---

## CLI Usage

### 1. Stateless Encoding & Decoding
```bash
# Encode URL into stateless algorithmic code
cargo run --release -- encode "https://shop.google.com/products/deals?id=123"

# Decode stateless shortcode back to original URL
cargo run --release -- decode "0CEbn5mgfiOdL~..."
```

### 2. State-Backed Shortening & Retrieval
```bash
# Generate compact shortcode stored in Bitcask database
cargo run --release -- shorten "https://shop.google.com/products/deals?id=123"

# Expand shortcode to raw URL
cargo run --release -- expand "058qV"
```

### 3. Batch Corpus Ingestion & Sealing
```bash
# Ingest CSV dataset and seal into succinct MPHF bitvectors
cargo run --release -- ingest data/list.csv --dir .store
```

### 4. Standalone CLI HTTP Server
```bash
cargo run --release -- serve --host 127.0.0.1 --port 8080 --store .store
```

---

## 🏛️ Architecture Overview

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
   | - Myers Positional Delta  |                         | - 5-Char Base66 Code      |
   | - FSST Static Symbols     |                         | - 64-Way Sharded Bitcask  |
   | - Tagged Stream RLE       |                         | - Zero-Copy Mmap Logs     |
   | - rANS Entropy / Grammar  |                         | - Succinct MPHF (3.6 B/key)|
   +---------------------------+                         +---------------------------+
```

---

## 📄 License

Apache-2.0 / MIT
