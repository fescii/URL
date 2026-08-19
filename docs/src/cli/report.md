# `urls report`

Automated multi-scale benchmark orchestration suite. Ingests, measures, seals, validates, and writes comparative Markdown and CSV reports across customizable corpus scale checkpoints.

---

## Usage

```bash
urls report [OPTIONS]
```

### Options

| Flag | Long Flag | Default | Description |
|---|---|---|---|
| `-i` | `--input` | `data/list.csv` | Path to CSV/TSV input corpus. |
| `-o` | `--out` | `reports` | Target directory for generated report artifacts. |
| `-s` | `--scales` | *(all defaults)* | Comma-separated list of integer checkpoints. |

---

## Examples

### Run Default 10-Scale Suite (1K → 4.31M)

```bash
urls report --input data/list.csv --out reports
```

### Run Custom Micro-Benchmarks (1K, 5K, 10K)

```bash
urls report --input data/list.csv --out reports --scales 1000,5000,10000
```

---

## Generated Artifact Structure

```
reports/
├── 1000/
│   ├── analysis.md    # In-depth technical breakdown for 1K scale
│   └── data.csv       # Granular per-URL measurement dataset
├── 5000/
│   ├── analysis.md
│   └── data.csv
├── ...
├── 4313006/
│   ├── analysis.md
│   └── data.csv
├── summary.md         # Consolidated markdown progression table
└── summary.csv        # Consolidated CSV matrix
```
