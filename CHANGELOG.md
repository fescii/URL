# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-08-19

### Added
- **Multi-Tier URL Compression Engine**:
  - **Tier 1 (Stateless)**: Myers bit-parallel positional deltas against structural centroid anchors with MinHash clustering.
  - **Tier 1.5 (Entropy & Symbol Grammars)**: Fast Static Symbol Tables (FSST), Re-Pair grammar compaction, and Range Asymmetric Numeral Systems (rANS).
  - **Tier 2 (Stateful Sharded Store)**: Dynamic 5–11 character Base66 shortcodes backed by an append-only Bitcask storage engine.
- **Succinct In-Memory Index (MPHF)**:
  - Two-level pilot seed search with SplitMix64 mixing and Elias-Fano monotone sequence encoding.
  - Squeezes **4,313,006 URLs into 14.91 MB of RAM** (**3.62 Bytes / Key**), achieving $>85\%$ RAM savings compared to standard hash tables.
- **Zero-Copy Memory-Mapped Storage Engine**:
  - Sharded Bitcask append-only logs with memory-mapped readers (`memmap2::Mmap`).
  - Zero-heap-allocation point query reads returning `bytes::Bytes` directly from OS page cache.
  - Integrated 2-tier Adaptive Replacement Cache (ARC) for sub-microsecond warm point queries.
- **CLI Commands**:
  - `urls encode`: Stateless algorithmic compression to portable base codes.
  - `urls decode`: Reverses stateless codes back to exact target URLs.
  - `urls shorten`: Stateful 5–11 character shortening.
  - `urls expand`: Zero-copy resolution of shortcodes from local Bitcask store.
  - `urls ingest`: High-speed batch streaming parser supporting CSV, TSV, and plain text.
  - `urls serve`: Asynchronous HTTP server for 301/302 link redirects, health probes, and JSON shorten API.
  - `urls report`: Automated multi-scale empirical benchmark orchestrator with Markdown and CSV export.
- **Documentation & CI/CD Infrastructure**:
  - Full mdBook / GitBook documentation suite in `docs/`.
  - GitHub Pages deployment workflow (`.github/workflows/docs.yml`).
  - Multi-platform cross-compilation release workflow (`.github/workflows/release.yml`) producing Linux, macOS, and Windows binary bundles.
  - One-line automated install scripts for Linux/macOS (`install.sh`) and Windows (`install.ps1`).
- **Empirical Benchmarks (1K → 4.31M Scale)**:
  - 10 distinct validation checkpoints from 1,000 to 4,313,006 URLs.
  - 100.00% lossless query validation across the entire dataset.
