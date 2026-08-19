use super::writer::{ScaleResult, StoreStats, UrlRecord, Writer};
use crate::design::{Error, Result};
use crate::stores::Store;
use crate::{decode, encode, shorten};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Orchestrator for streaming multi-scale benchmarks and report generation.
pub struct Runner {
  list_path: PathBuf,
  out_dir: PathBuf,
  scales: Vec<usize>,
}

impl Runner {
  pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(list_path: P, out_dir: Q) -> Self {
    Self {
      list_path: list_path.as_ref().to_path_buf(),
      out_dir: out_dir.as_ref().to_path_buf(),
      scales: vec![
        1_000, 5_000, 10_000, 50_000, 100_000, 500_000, 1_000_000, 2_000_000, 3_000_000, 4_313_006,
      ],
    }
  }

  /// Override scale checkpoints.
  pub fn with_scales(mut self, scales: Vec<usize>) -> Self {
    self.scales = scales;
    self
  }

  /// Run streaming benchmark across all scales and write reports into subfolders.
  pub fn run(&self) -> Result<Vec<ScaleResult>> {
    if !self.list_path.exists() {
      return Err(Error::Store(format!(
        "dataset not found: {}",
        self.list_path.display()
      )));
    }

    crate::ingest!(
      "starting multi-scale benchmark output_dir={} scales={:?}",
      self.out_dir.display(),
      self.scales
    );

    let file = File::open(&self.list_path).map_err(Error::from)?;
    let reader = BufReader::with_capacity(1024 * 1024 * 8, file);

    let mut all_results = Vec::with_capacity(self.scales.len());

    let temp_base = std::env::temp_dir().join(format!(
      "urls_reports_{}",
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));

    let mut current_scale_idx = 0;
    let current_store_dir = temp_base.join(format!("scale_{}", self.scales[0]));
    let mut store = Store::open(&current_store_dir)?;

    let mut count = 0usize;
    let mut raw_bytes_acc = 0usize;
    let mut sample_urls = Vec::with_capacity(1000);
    let mut verification_samples: Vec<(String, String)> = Vec::with_capacity(1000);

    let scale_start_time = Instant::now();

    for line in reader.lines() {
      let line_str = line.map_err(Error::from)?;
      if count == 0 && line_str.starts_with("id,") {
        continue; // Skip CSV header
      }
      if line_str.trim().is_empty() {
        continue;
      }

      let parts: Vec<&str> = line_str.splitn(3, ',').collect();
      if parts.len() < 3 {
        continue;
      }

      let url = parts[2].trim().trim_matches('"').to_string();
      if url.is_empty() {
        continue;
      }

      count += 1;
      raw_bytes_acc += url.len();

      let shortcut = shorten(&url, Some(&mut store))?;

      // Keep sample records for urls.csv
      if sample_urls.len() < 1000 {
        let zero_code = encode(&url, None).unwrap_or_else(|_| "N/A".to_string());
        let raw_len = url.len();
        let zero_len = zero_code.len();
        let shortcut_len = shortcut.len();
        let zero_savings = (1.0 - (zero_len as f64 / raw_len as f64)) * 100.0;
        let store_savings = (1.0 - (shortcut_len as f64 / raw_len as f64)) * 100.0;

        sample_urls.push(UrlRecord {
          id: count,
          url: url.clone(),
          zero_code,
          shortcut: shortcut.clone(),
          raw_len,
          zero_len,
          shortcut_len,
          zero_savings,
          store_savings,
        });

        verification_samples.push((url, shortcut));
      }

      // Check if we hit a scale checkpoint
      if current_scale_idx < self.scales.len() && count == self.scales[current_scale_idx] {
        let target_scale = self.scales[current_scale_idx];
        let elapsed = scale_start_time.elapsed();

        // 1. Measure disk size
        let mut disk_bytes = 0u64;
        if let Ok(entries) = std::fs::read_dir(&current_store_dir) {
          for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
              disk_bytes += meta.len();
            }
          }
        }

        let mutable_ram = store.memory_size();
        let disk_savings = (1.0 - (disk_bytes as f64 / raw_bytes_acc as f64)) * 100.0;

        // 2. Measure verification lookup latency
        let verify_start = Instant::now();
        let verify_count = 100.min(verification_samples.len());
        for (orig, code) in &verification_samples[..verify_count] {
          let resolved = if let Ok(Some(raw)) = store.get_key(code) {
            std::str::from_utf8(&raw).map_err(|e| Error::Codec(e.to_string()))?.to_string()
          } else {
            decode(code)?
          };
          if &resolved != orig {
            return Err(Error::Codec(format!(
              "lossless mismatch at scale {target_scale}"
            )));
          }
        }
        let lookup_micros = verify_start.elapsed().as_micros() as f64 / verify_count.max(1) as f64;

        // 3. Seal store to measure succinct index RAM
        let initial_ram = store.memory_size();
        store.seal();
        let succinct_ram = store.memory_size();
        let ram_per_key = succinct_ram as f64 / target_scale as f64;
        let ingest_rate = count as f64 / elapsed.as_secs_f64().max(0.0001);

        let stats = StoreStats {
          scale: target_scale,
          raw_bytes: raw_bytes_acc,
          disk_bytes,
          disk_savings,
          mutable_ram,
          succinct_ram,
          ram_per_key,
          ingest_rate,
          lookup_micros,
          duration_ms: elapsed.as_millis(),
        };

        // 4. Write scale reports to subfolder (e.g. reports/1000/)
        let scale_dir = self.out_dir.join(target_scale.to_string());
        Writer::write_scale(&scale_dir, &stats, &sample_urls)?;

        crate::ingest!(
          "generated scale report: scale={} disk_savings={:.1}% ram_per_key={:.2}B rate={:.0} URLs/s",
          target_scale,
          disk_savings,
          ram_per_key,
          ingest_rate
        );

        all_results.push(ScaleResult {
          scale: target_scale,
          raw_bytes: raw_bytes_acc,
          disk_bytes,
          disk_savings,
          mutable_ram,
          succinct_ram,
          ram_per_key,
          ingest_rate,
          lookup_micros,
          duration_ms: elapsed.as_millis(),
        });

        current_scale_idx += 1;
        if current_scale_idx >= self.scales.len() {
          break;
        }
      }
    }

    // Write top-level summary.md and summary.csv
    Writer::write_summary(&self.out_dir, &all_results)?;

    crate::ingest!(
      "multi-scale reports successfully generated in {}",
      self.out_dir.display()
    );

    // Clean up temporary store
    let _ = std::fs::remove_dir_all(&temp_base);

    Ok(all_results)
  }
}
