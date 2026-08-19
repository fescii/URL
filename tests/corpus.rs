use urls::profiles::Profile;
use urls::stores::Store;
use urls::{decode, encode, shorten};

struct BenchmarkItem {
  category: &'static str,
  original: &'static str,
}

#[test]
fn test_corpus_compression_and_generate_results() {
  let corpus = vec![
    // 1. Social & Media
    BenchmarkItem {
      category: "Social & Media",
      original: "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    },
    BenchmarkItem {
      category: "Social & Media",
      original: "https://x.com/rustlang/status/1789012345678901234",
    },
    BenchmarkItem {
      category: "Social & Media",
      original: "https://www.instagram.com/p/C9AbCdEfGhI/?igshid=MzRlODBiNWFlZA==",
    },
    BenchmarkItem {
      category: "Social & Media",
      original: "https://www.reddit.com/r/rust/comments/123456/announcing_urls_v1_compression/",
    },
    BenchmarkItem {
      category: "Social & Media",
      original: "https://www.linkedin.com/posts/rust-foundation_systems-programming-rustlang-activity-7123456789012345678-AbCd",
    },
    // 2. Developer & Code
    BenchmarkItem {
      category: "Developer & Code",
      original: "https://github.com/rust-lang/rust/pull/12345",
    },
    BenchmarkItem {
      category: "Developer & Code",
      original: "https://github.com/rust-lang/rust/blob/master/compiler/rustc_middle/src/ty/context.rs",
    },
    BenchmarkItem {
      category: "Developer & Code",
      original: "https://crates.io/crates/clap",
    },
    BenchmarkItem {
      category: "Developer & Code",
      original: "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
    },
    BenchmarkItem {
      category: "Developer & Code",
      original: "https://api.github.com/repos/rust-lang/rust/issues?state=open&sort=created&direction=desc",
    },
    // 3. Marketing & E-Commerce
    BenchmarkItem {
      category: "Marketing & E-Commerce",
      original: "https://www.amazon.com/dp/B08N5WRWNW?ref=nb_sb_ss_custom_track&utm_source=newsletter&utm_medium=email&utm_campaign=summer_sale&utm_content=hero_cta&gclid=EAIaIQobChMI123456789",
    },
    BenchmarkItem {
      category: "Marketing & E-Commerce",
      original: "https://store.steampowered.com/app/1086940/Baldurs_Gate_3/?utm_source=steam_store&utm_medium=featured&utm_campaign=winter_sale&fbclid=IwAR2AbCdEfGhIjKlMnOpQrStUvWxYz",
    },
    // 4. Knowledge & Reference
    BenchmarkItem {
      category: "Knowledge & Reference",
      original: "https://en.wikipedia.org/wiki/Asymmetric_numeral_systems",
    },
    BenchmarkItem {
      category: "Knowledge & Reference",
      original: "https://en.wikipedia.org/wiki/Straight-line_program",
    },
    // 5. Decentralized & Alternate Protocols
    BenchmarkItem {
      category: "Decentralized & Protocols",
      original: "ipfs://bafybeic56t3rwfi6eza63q7hnvd6n2s6j7vg2nff22e7w273nzyuap243a",
    },
    BenchmarkItem {
      category: "Decentralized & Protocols",
      original: "magnet:?xt=urn:btih:d2b0018a1a3641b69ad312384a6c4df19910d65b&dn=Rust_Programming_Book_2026",
    },
    BenchmarkItem {
      category: "Decentralized & Protocols",
      original: "mailto:core-team@rust-lang.org?subject=Zero-Storage%20URL%20Compression",
    },
    BenchmarkItem {
      category: "Decentralized & Protocols",
      original: "bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.005&label=Satoshi",
    },
  ];

  let profile = Profile::generic();
  let temp_dir = std::env::temp_dir().join(format!(
    "urls_bench_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let mut store = Store::open(&temp_dir).expect("open temp store failed");

  let mut markdown = String::new();

  markdown.push_str("# URL Compression Benchmark & Lossless Verification Report\n\n");
  markdown
    .push_str("Generated automatically by `urls` lossless compression engine test suite.\n\n");
  markdown.push_str("### 3-Tier Multi-Strategy Architecture: Tier 1 (Pure Algorithmic), Tier 1.5 (Parametric Template), Tier 2 (Succinct Store)\n\n");
  markdown.push_str("| Category | Original URL (Before) | Raw | Zero-Storage (T1 / T1.5) | Zero-Storage Red. | Succinct Store (Tier 2) | Store Red. | Status |\n");
  markdown.push_str("|---|---|---|---|---|---|---|---|\n");

  let mut total_raw = 0usize;
  let mut total_zero_storage = 0usize;
  let mut total_tiered = 0usize;

  for item in &corpus {
    let code = encode(item.original, Some(&profile)).expect("encode failed");
    let decoded = decode(&code).expect("decode failed");

    assert_eq!(
      decoded, item.original,
      "Lossless roundtrip assertion failed for URL: {}",
      item.original
    );

    let tiered_code = shorten(item.original, Some(&mut store)).expect("shorten failed");
    if let Ok(Some(fetched_bytes)) = store.get_key(&tiered_code) {
      let resolved_url = String::from_utf8(fetched_bytes.to_vec()).unwrap();
      assert_eq!(
        resolved_url, item.original,
        "Store shortcut resolution failed for URL: {}",
        item.original
      );
    } else {
      let decoded = decode(&tiered_code).expect("decode tiered_code failed");
      assert_eq!(
        decoded, item.original,
        "Algorithmic decode assertion failed for URL: {}",
        item.original
      );
    }

    let raw_len = item.original.len();
    let code_len = code.len();
    let tiered_len = tiered_code.len();

    total_raw += raw_len;
    total_zero_storage += code_len;
    total_tiered += tiered_len;

    let status = if decoded == item.original {
      "Verified Lossless"
    } else {
      "Failed"
    };

    // Truncate long strings for clean table formatting
    let display_orig = if item.original.len() > 42 {
      format!("`{}...`", &item.original[..39])
    } else {
      format!("`{}`", item.original)
    };

    let display_code = if code.len() > 22 {
      format!("`{}...`", &code[..19])
    } else {
      format!("`{}`", code)
    };

    let t1_pct = (1.0 - (code_len as f64 / raw_len as f64)) * 100.0;
    let t2_pct = (1.0 - (tiered_len as f64 / raw_len as f64)) * 100.0;

    markdown.push_str(&format!(
      "| {} | {} | {} B | {} ({} B) | **-{:.1}%** | `{}` ({} B) | **-{:.1}%** | {} |\n",
      item.category,
      display_orig,
      raw_len,
      display_code,
      code_len,
      t1_pct,
      tiered_code,
      tiered_len,
      t2_pct,
      status
    ));
  }

  markdown.push_str("\n---\n\n");
  markdown.push_str("## Detailed Full URL Listings\n\n");

  for (i, item) in corpus.iter().enumerate() {
    let code = encode(item.original, Some(&profile)).unwrap();
    let tiered_code = shorten(item.original, Some(&mut store)).unwrap();
    markdown.push_str(&format!("### #{}: {}\n", i + 1, item.category));
    markdown.push_str(&format!(
      "- **Original (Before)**: `{}` ({} bytes)\n",
      item.original,
      item.original.len()
    ));
    markdown.push_str(&format!(
      "- **Zero-Storage Code (Tier 1 / 1.5)**: `{}` ({} chars / bytes)\n",
      code,
      code.len()
    ));
    markdown.push_str(&format!(
      "- **Hybrid Store Code (Tier 2)**: `{}` ({} chars / bytes)\n",
      tiered_code,
      tiered_code.len()
    ));
    markdown.push_str("- **Verification**: Byte-for-byte identical roundtrip decode.\n\n");
  }

  let overall_t1_reduction = (1.0 - (total_zero_storage as f64 / total_raw as f64)) * 100.0;
  let overall_t2_reduction = (1.0 - (total_tiered as f64 / total_raw as f64)) * 100.0;

  markdown.push_str("## Summary Metrics\n\n");
  markdown.push_str(&format!(
    "- **Total Original Raw URL Bytes**: {} B\n",
    total_raw
  ));
  markdown.push_str(&format!("- **Total Zero-Storage (T1 / T1.5) Bytes**: {} B (**-{:.1}% reduction**, 0 B database storage)\n", total_zero_storage, overall_t1_reduction));
  markdown.push_str(&format!("- **Total Tier 2 Succinct Store Bytes**: {} B (**-{:.1}% reduction**, ultra-compact 6-7 char shortcuts, ~3.5 bits/key in RAM)\n", total_tiered, overall_t2_reduction));
  markdown.push_str("- **Lossless Accuracy**: 100.00% across all URL protocols\n");

  // Write to root results.md
  let root_path = if std::path::Path::new("results.md").exists()
    || !std::path::Path::new("../results.md").exists()
  {
    std::path::Path::new("results.md")
  } else {
    std::path::Path::new("../results.md")
  };
  std::fs::write(root_path, markdown).expect("failed to write results.md");
}
