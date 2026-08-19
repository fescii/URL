use crate::design::Result;
use crate::shorten;
use crate::stores::Store;
use std::time::Instant;

/// Statistics report for ingested URL batches.
#[derive(Debug, Clone, Default)]
pub struct Stats {
  pub count: usize,
  pub raw_bytes: usize,
  pub disk_bytes: u64,
  pub ram_bytes: usize,
  pub duration_millis: u128,
  pub rate: f64,
}

/// Batch chunking processor for high-speed store ingestion.
pub struct Batch;

impl Batch {
  /// Ingest a collection of URLs in configurable chunk sizes (e.g. 10k per batch).
  pub fn process(urls: &[String], store: &mut Store, chunk_size: usize) -> Result<Stats> {
    let size = chunk_size.max(1);
    let start = Instant::now();

    let mut total_raw_bytes = 0usize;
    let mut total_ingested = 0usize;

    let total_chunks = (urls.len() + size - 1) / size;

    for (chunk_idx, chunk) in urls.chunks(size).enumerate() {
      let chunk_start = Instant::now();
      let mut chunk_raw_bytes = 0usize;

      for url in chunk {
        chunk_raw_bytes += url.len();
        let _ = shorten(url, Some(store))?;
        total_ingested += 1;
      }

      total_raw_bytes += chunk_raw_bytes;
      let chunk_dur = chunk_start.elapsed().as_secs_f64();
      let chunk_rate = chunk.len() as f64 / chunk_dur.max(0.0001);

      crate::store!(
        "ingested chunk {}/{} ({} items) at {:.0} URLs/sec (total: {}/{})",
        chunk_idx + 1,
        total_chunks,
        chunk.len(),
        chunk_rate,
        total_ingested,
        urls.len()
      );
    }

    let total_dur = start.elapsed();
    let total_millis = total_dur.as_millis();
    let rate = total_ingested as f64 / total_dur.as_secs_f64().max(0.0001);

    Ok(Stats {
      count: total_ingested,
      raw_bytes: total_raw_bytes,
      disk_bytes: 0,
      ram_bytes: store.memory_size(),
      duration_millis: total_millis,
      rate,
    })
  }
}
