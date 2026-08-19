# `urls ingest`

High-speed streaming ingestion pipeline for processing multi-million URL datasets in CSV, TSV, or plain text formats.

---

## Usage

```bash
urls ingest <FILE> [OPTIONS]
```

### Options

| Flag | Long Flag | Default | Description |
|---|---|---|---|
| `<FILE>` | *(positional)* | *(required)* | Path to input dataset (e.g. `data/list.csv`). |
| `-b` | `--batch` | `10000` | Chunk batch size for write batching. |
| `-l` | `--limit` | `None` | Optional ceiling on total rows to ingest. |
| `-c` | `--col` | `None` | Specific 0-indexed column containing URL (auto-detected if omitted). |
| `-d` | `--dir` | `.urls_store` | Target storage directory. |
| `-s` | `--seal` | `false` | Automatically seals store into succinct MPHF bitvectors upon completion. |

---

## Examples

### Ingest Full 4.31M Dataset

```bash
urls ingest data/list.csv --batch 50000 --seal --dir .urls_store
```

### Ingest 100K Rows for Quick Testing

```bash
urls ingest data/list.csv --limit 100000 --batch 10000 --dir tmp_store
```
