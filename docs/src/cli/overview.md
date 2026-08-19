# CLI Reference: Overview

The `urls` CLI provides a set of subcommands for encoding, shortening, batch ingestion, HTTP serving, and benchmarking.

---

## Command Matrix

| Command | Category | Description |
|---|---|---|
| `urls encode` | Stateless | Algorithmic compression into standalone base-encoded string. |
| `urls decode` | Stateless | Reverses algorithmic encoding without any database. |
| `urls shorten` | Stateful | Generates 5–11 char shortcode backed by Bitcask store. |
| `urls expand` | Stateful | Resolves shortcode back to raw URL from local store. |
| `urls ingest` | Batch | High-speed batch streaming ingestion of CSV/TSV datasets. |
| `urls serve` | Network | High-throughput HTTP redirect and lookup API server. |
| `urls report` | Benchmark | Runs multi-scale progression analysis (1K → 4.31M URLs). |

---

## Global Help & Version

```bash
urls --help
urls --version
```
