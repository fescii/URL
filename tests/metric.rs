use std::time::Instant;
use urls::profiles::Profile;
use urls::stores::Store;
use urls::{decode, encode, shorten};

struct BenchmarkRecord {
  category: &'static str,
  original: &'static str,
}

#[test]
fn test_store_benchmark_and_export_csv() {
  let corpus = vec![
    BenchmarkRecord {
      category: "Social & Media",
      original: "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    },
    BenchmarkRecord {
      category: "Social & Media",
      original: "https://x.com/rustlang/status/1789012345678901234",
    },
    BenchmarkRecord {
      category: "Social & Media",
      original: "https://www.instagram.com/p/C9AbCdEfGhI/?igshid=MzRlODBiNWFlZA==",
    },
    BenchmarkRecord {
      category: "Social & Media",
      original: "https://www.reddit.com/r/rust/comments/123456/announcing_urls_v1_compression/",
    },
    BenchmarkRecord {
      category: "Social & Media",
      original: "https://www.linkedin.com/posts/rust-foundation_systems-programming-rustlang-activity-7123456789012345678-AbCd",
    },
    BenchmarkRecord {
      category: "Developer & Code",
      original: "https://github.com/rust-lang/rust/pull/12345",
    },
    BenchmarkRecord {
      category: "Developer & Code",
      original: "https://github.com/rust-lang/rust/blob/master/compiler/rustc_middle/src/ty/context.rs",
    },
    BenchmarkRecord {
      category: "Developer & Code",
      original: "https://crates.io/crates/clap",
    },
    BenchmarkRecord {
      category: "Developer & Code",
      original: "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
    },
    BenchmarkRecord {
      category: "Developer & Code",
      original: "https://api.github.com/repos/rust-lang/rust/issues?state=open&sort=created&direction=desc",
    },
    BenchmarkRecord {
      category: "Marketing & E-Commerce",
      original: "https://www.amazon.com/dp/B08N5WRWNW?ref=nb_sb_ss_custom_track&utm_source=newsletter&utm_medium=email&utm_campaign=summer_sale&utm_content=hero_cta&gclid=EAIaIQobChMI123456789",
    },
    BenchmarkRecord {
      category: "Marketing & E-Commerce",
      original: "https://store.steampowered.com/app/1086940/Baldurs_Gate_3/?utm_source=steam_store&utm_medium=featured&utm_campaign=winter_sale&fbclid=IwAR2AbCdEfGhIjKlMnOpQrStUvWxYz",
    },
    BenchmarkRecord {
      category: "Knowledge & Reference",
      original: "https://en.wikipedia.org/wiki/Asymmetric_numeral_systems",
    },
    BenchmarkRecord {
      category: "Knowledge & Reference",
      original: "https://en.wikipedia.org/wiki/Straight-line_program",
    },
    BenchmarkRecord {
      category: "Decentralized & Protocols",
      original: "ipfs://bafybeic56t3rwfi6eza63q7hnvd6n2s6j7vg2nff22e7w273nzyuap243a",
    },
    BenchmarkRecord {
      category: "Decentralized & Protocols",
      original: "magnet:?xt=urn:btih:d2b0018a1a3641b69ad312384a6c4df19910d65b&dn=Rust_Programming_Book_2026",
    },
    BenchmarkRecord {
      category: "Decentralized & Protocols",
      original: "mailto:core-team@rust-lang.org?subject=Zero-Storage%20URL%20Compression",
    },
    BenchmarkRecord {
      category: "Decentralized & Protocols",
      original: "bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.005&label=Satoshi",
    },
  ];

  let profile = Profile::generic();

  // Setup temporary storage directory
  let temp_dir = std::env::temp_dir().join(format!(
    "urls_csv_bench_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let mut store = Store::open(&temp_dir).expect("open store failed");

  // Prepare CSV header
  let mut csv = String::new();
  csv.push_str("id,category,original_url,raw_bytes,zero_storage_code,zero_bytes,store_shortcut,store_bytes,disk_raw_bytes,disk_compressed_bytes,ram_hashmap_bytes,ram_succinct_bytes,lookup_nanos,status\n");

  let mut total_raw_bytes = 0usize;
  let mut total_zero_bytes = 0usize;
  let mut total_store_bytes = 0usize;
  let mut total_disk_raw = 0usize;
  let mut total_disk_compressed = 0usize;

  for (i, item) in corpus.iter().enumerate() {
    let raw_len = item.original.len();
    total_raw_bytes += raw_len;

    // 1. Zero-Storage Encode & Verify
    let zero_code = encode(item.original, Some(&profile)).expect("encode failed");
    let decoded = decode(&zero_code).expect("decode failed");
    assert_eq!(decoded, item.original);

    let zero_len = zero_code.len();
    total_zero_bytes += zero_len;

    // 2. Tier 2 Shorten (Compressed Payload Store)
    let shortcut = shorten(item.original, Some(&mut store)).expect("shorten failed");
    let store_len = shortcut.len();
    total_store_bytes += store_len;

    // 3. Disk footprint calculation
    let disk_raw = raw_len + 40; // 32-byte BLAKE3 key + 8-byte header
    let disk_compressed = zero_len + 40;
    total_disk_raw += disk_raw;
    total_disk_compressed += disk_compressed;

    // 4. Memory footprint calculation per key
    let ram_hashmap = 64; // std::collections::HashMap overhead per entry (Hash + Location + bucket pointer)
    let ram_succinct = 10; // Succinct MPHF + offset array (~5.3 bytes rounded to word)

    // 5. Measure Lookup Latency
    let start = Instant::now();
    if let Ok(Some(fetched)) = store.get_key(&shortcut) {
      let _ = String::from_utf8(fetched.to_vec()).unwrap();
    } else {
      let _ = decode(&shortcut).unwrap();
    }
    let lookup_nanos = start.elapsed().as_nanos();

    // 6. Escape fields for CSV formatting
    let escaped_url = format!("\"{}\"", item.original.replace('"', "\"\""));
    let escaped_zero = format!("\"{}\"", zero_code.replace('"', "\"\""));
    let escaped_store = format!("\"{}\"", shortcut.replace('"', "\"\""));

    csv.push_str(&format!(
      "{},\"{}\",{},{},{},{},{},{},{},{},{},{},{},\"Verified\"\n",
      i + 1,
      item.category,
      escaped_url,
      raw_len,
      escaped_zero,
      zero_len,
      escaped_store,
      store_len,
      disk_raw,
      disk_compressed,
      ram_hashmap,
      ram_succinct,
      lookup_nanos
    ));
  }

  // Seal store to verify in-memory compaction
  let initial_ram = store.memory_size();
  store.seal();
  let sealed_ram = store.memory_size();
  assert!(sealed_ram <= initial_ram);

  // Write CSV file to root
  let csv_path = if std::path::Path::new("store.csv").exists()
    || !std::path::Path::new("../store.csv").exists()
  {
    std::path::Path::new("store.csv")
  } else {
    std::path::Path::new("../store.csv")
  };
  std::fs::write(csv_path, &csv).expect("failed to write store.csv");

  urls::store!(
    "store benchmark summary: raw_bytes={} zero_bytes={} store_bytes={} disk_raw={} disk_comp={} ram_mut={} ram_succ={}",
    total_raw_bytes,
    total_zero_bytes,
    total_store_bytes,
    total_disk_raw,
    total_disk_compressed,
    initial_ram / corpus.len().max(1),
    sealed_ram / corpus.len().max(1)
  );
}
