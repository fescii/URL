# Datasets

This directory contains large benchmarking datasets used for multi-scale corpus ingestion and performance evaluation.

---

## Dataset Files

- **`list.csv`** (~945 MB): Full corpus of **4,313,006 real-world URLs** with metadata.
- **`common.csv`** (~104 MB): Secondary web corpus dataset.

> [!NOTE]
> Large CSV datasets in this folder are intentionally ignored by `.gitignore` to avoid bloating the Git repository history.

---

## Download & Setup Instructions

1. Download the datasets from the shared Drive storage link:
   - **Google Drive Dataset Mirror**: `[Insert Shared Drive Link Here]`
2. Place the uncompressed `list.csv` (and optionally `common.csv`) directly inside this directory:
   ```bash
   data/list.csv
   data/common.csv
   ```
3. Run the benchmark or ingestion pipeline:
   ```bash
   # Run full multi-scale benchmark
   cargo run --release -- report --input data/list.csv --out reports

   # Ingest dataset into storage
   cargo run --release -- ingest data/list.csv --dir .urls_store
   ```
