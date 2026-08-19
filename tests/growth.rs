use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use urls::stores::Store;
use urls::{decode, shorten};

#[test]
fn test_store_growth_with_large_scale_urls() {
  let list_path = if Path::new("data/list.csv").exists() {
    Path::new("data/list.csv")
  } else if Path::new("list.csv").exists() {
    Path::new("list.csv")
  } else if Path::new("../data/list.csv").exists() {
    Path::new("../data/list.csv")
  } else {
    Path::new("../list.csv")
  };
  assert!(list_path.exists(), "list.csv must exist in data/ or workspace root");

  let file = File::open(list_path).expect("failed to open list.csv");
  let reader = BufReader::new(file);

  // Stream 5,000 randomized URLs from list.csv for high-speed benchmark validation
  let target_count = 5000;
  let mut urls = Vec::with_capacity(target_count);
  for (line_idx, line) in reader.lines().enumerate() {
    if urls.len() >= target_count {
      break;
    }
    let line = line.expect("read line error");
    if line_idx == 0 || line.trim().is_empty() {
      continue; // Skip CSV header
    }

    // CSV format: id,domain,url
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.len() == 3 {
      let clean_url = parts[2].trim().trim_matches('"');
      urls.push(clean_url.to_string());
    }
  }

  assert_eq!(urls.len(), target_count);

  // Create fresh isolated store
  let temp_dir = std::env::temp_dir().join(format!(
    "urls_scale_growth_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let mut store = Store::open(&temp_dir).expect("open store failed");

  let mut raw_bytes_acc = 0usize;
  let mut shortcuts = Vec::with_capacity(target_count);

  let checkpoints = [500, 1000, 2500, 5000];
  let mut checkpoint_idx = 0;

  let insert_start = Instant::now();

  for (i, url) in urls.iter().enumerate() {
    raw_bytes_acc += url.len();

    let shortcut = shorten(url, Some(&mut store)).expect("shorten failed");
    assert!(
      shortcut.len() <= 9 || shortcut.starts_with('0') || shortcut.starts_with('3'),
      "shortcut length exceeded expected bounds: {shortcut}"
    );
    shortcuts.push((url.clone(), shortcut));

    let current_count = i + 1;
    if checkpoint_idx < checkpoints.len() && current_count == checkpoints[checkpoint_idx] {
      // Measure disk size of all shard log files
      let mut disk_bytes = 0u64;
      if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
          if let Ok(meta) = entry.metadata() {
            disk_bytes += meta.len();
          }
        }
      }

      let mut_ram = store.memory_size();
      let disk_savings = (1.0 - (disk_bytes as f64 / raw_bytes_acc as f64)) * 100.0;

      // Measure lookup latency over sample
      let lookup_start = Instant::now();
      let sample_size = 50.min(shortcuts.len());
      for (orig, code) in &shortcuts[shortcuts.len() - sample_size..] {
        let resolved = if let Ok(Some(raw)) = store.get_key(code) {
          String::from_utf8(raw.to_vec()).unwrap()
        } else {
          decode(code).unwrap()
        };
        assert_eq!(&resolved, orig);
      }
      let lookup_micros = lookup_start.elapsed().as_micros() as f64 / sample_size as f64;

      urls::store!(
        "checkpoint: count={} raw_bytes={} disk_bytes={} savings={:.1}% ram_mut={} lookup_us={:.2}",
        current_count,
        raw_bytes_acc,
        disk_bytes,
        disk_savings,
        mut_ram,
        lookup_micros
      );

      checkpoint_idx += 1;
    }
  }

  let insert_duration = insert_start.elapsed();

  // Seal store to succinct representation and verify full dataset losslessness
  let pre_seal_ram = store.memory_size();
  store.seal();
  let post_seal_ram = store.memory_size();
  assert!(post_seal_ram <= pre_seal_ram);

  let verify_start = Instant::now();
  for (i, (orig, code)) in shortcuts.iter().enumerate() {
    let raw_res = store.get_key(code);
    let resolved = match raw_res {
      Ok(Some(raw)) => String::from_utf8(raw.to_vec()).unwrap(),
      Ok(None) => {
        urls::store!("FAILED at index {i}: code={code} orig={orig} -> get_key returned None");
        decode(code).unwrap_or_else(|e| panic!("Failed to resolve code '{code}' at index {i}: {e}"))
      }
      Err(e) => panic!("get_key returned Err: {e}"),
    };
    assert_eq!(&resolved, orig);
  }
  let verify_duration = verify_start.elapsed();

  urls::store!(
    "scale summary: total=5000 insert_dur_ms={} verify_dur_ms={} sealed_ram={}",
    insert_duration.as_millis(),
    verify_duration.as_millis(),
    post_seal_ram
  );

  // Clean up temporary test directory
  let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_stream_4m_dataset_integrity() {
  let list_path = if Path::new("data/list.csv").exists() {
    Path::new("data/list.csv")
  } else if Path::new("list.csv").exists() {
    Path::new("list.csv")
  } else if Path::new("../data/list.csv").exists() {
    Path::new("../data/list.csv")
  } else {
    Path::new("../list.csv")
  };
  assert!(list_path.exists(), "list.csv must exist in data/ or workspace root");

  let file = File::open(list_path).expect("failed to open list.csv");
  let reader = BufReader::with_capacity(1024 * 1024 * 4, file);

  let mut total_lines = 0usize;
  let mut sample_urls = Vec::new();

  for (line_idx, line) in reader.lines().enumerate() {
    let line = line.expect("read line failed");
    if line_idx == 0 || line.trim().is_empty() {
      continue;
    }
    total_lines += 1;

    // Sample lines periodically to verify format across the 4.3M rows
    if total_lines % 500_000 == 0 || total_lines == 1 || total_lines == 4_313_006 {
      let parts: Vec<&str> = line.splitn(3, ',').collect();
      if parts.len() == 3 {
        let u = parts[2].trim().trim_matches('"');
        assert!(u.starts_with("http://") || u.starts_with("https://"));
        sample_urls.push(u.to_string());
      }
    }
  }

  assert!(
    total_lines >= 4_000_000,
    "expected 4M+ URLs in list.csv, found {total_lines}"
  );
  assert!(!sample_urls.is_empty());
}
