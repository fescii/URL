# Issue: Storage Optimization — Block-Level Chunk Compression & Domain Normalization (Target: 30–50% Disk Reduction)

## Description

In the initial release, the sealed Minimal Perfect Hash Function (MPHF) index achieves an **A+ memory compaction grade** (**3.62 Bytes / Key**, 14.9 MB RAM for 4.31M URLs), and point query lookups operate with **zero heap allocations** using memory-mapped `bytes::Bytes`.

However, the **stored on-disk footprint** currently achieves **~10.06% net reduction** (713.97 MB stored vs 793.83 MB raw, saving 79.86 MB).

This issue tracks the implementation of **Block-Level Chunk Compression** and **Domain Normalization** to reach **30%–50% net disk storage reduction** while preserving microsecond point query speeds.

---

## Root Cause Analysis

1. **Per-Record Bitcask Header Overhead**:
   - Each individual entry currently incurs 15–17 bytes of framing metadata (Record Mode `1B` + Key Digest `8B` + Payload Length `2-4B` + CRC32 `4B`).
   - For an average URL size of ~180 bytes, per-entry framing accounts for ~9% of the storage volume.

2. **Single-Record Compression Boundary Limits**:
   - Compressing each URL individually (via Myers Positional Delta, RLE, or FSST) achieves 20%–30% raw reduction on isolated segments, but lacks dictionary context across neighboring records.
   - When subtracted from the per-record framing overhead, net savings plateau at ~10.1%.

---

## Proposed Architecture & Enhancements

### 1. Chunked Block Compression (4KB – 64KB Block Pages)
- Group Bitcask records into aligned chunks (e.g. 16KB or 32KB blocks) compressed with shard-trained Zstandard or FSST block dictionaries.
- The sealed MPHF index stores `(block_id: u32, offset_in_block: u16)`.
- Decompressing a single 16KB block takes $< 2\text{ µs}$ on modern hardware and enables cross-URL token deduplication.

### 2. Domain / Scheme Normalization Table
- Extract schemes and domain roots (e.g. `https://`, `shop.google.com`, `api.ezviz7.com`) into a compact, deduplicated global string table ($< 1\text{ MB}$).
- Replace redundant prefix strings in log records with a 2-byte integer ID, reducing the raw input payload size by 40%–60% before block compression.

### 3. Bitcask Header Compaction
- In sealed shards, strip individual CRC32 and hash digest fields from data pages, relying on block-level checksums and the MPHF fingerprint array.

---

## Target Metrics

| Metric | Current v0.1.0 | Target v0.2.0 |
|---|:---:|:---:|
| **Stored Disk (4.31M URLs)** | 713.97 MB (10.06% saved) | **400 MB – 500 MB (40%–50% saved)** |
| **Index RAM per Key** | 3.62 Bytes / key | $\le 4.0\text{ Bytes / key}$ |
| **Lookup Latency** | 3.0 µs – 111.9 µs | $< 50\text{ µs}$ |
| **Lossless Integrity** | 100.00% | 100.00% |

---

## Tasks

- [ ] Design block-level chunk container format in `src/stores/block.rs`.
- [ ] Implement global domain extraction and normalization table in `src/codecs/domain.rs`.
- [ ] Update `src/stores/shard.rs` and `src/stores/log.rs` to support block-addressed reads.
- [ ] Integrate block-level Zstandard / FSST trained dictionary codec.
- [ ] Add regression and scale benchmarks in `tests/` and run 4.31M verification pass.
