use std::fs;
use urls::objects::{Blob, Manifest, Merge};
use urls::{Profile, batch, encode, export, open, verify};

#[test]
fn test_sharable_urls_container_lifecycle() {
  let profile = Profile::generic();

  // 1. Author compiles a collection of 18 links to share
  let raw_links = vec![
    "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    "https://x.com/rustlang/status/1789012345678901234",
    "https://www.instagram.com/p/C9AbCdEfGhI/?igshid=MzRlODBiNWFlZA==",
    "https://www.reddit.com/r/rust/comments/123456/announcing_urls_v1_compression/",
    "https://www.linkedin.com/posts/rust-foundation_systems-programming-rustlang-activity-7123456789012345678-AbCd",
    "https://github.com/rust-lang/rust/pull/12345",
    "https://github.com/rust-lang/rust/blob/master/compiler/rustc_middle/src/ty/context.rs",
    "https://crates.io/crates/clap",
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html",
    "https://api.github.com/repos/rust-lang/rust/issues?state=open&sort=created&direction=desc",
    "https://www.amazon.com/dp/B08N5WRWNW?ref=nb_sb_ss_custom_track&utm_source=newsletter&utm_medium=email&utm_campaign=summer_sale&utm_content=hero_cta&gclid=EAIaIQobChMI123456789",
    "https://store.steampowered.com/app/1086940/Baldurs_Gate_3/?utm_source=steam_store&utm_medium=featured&utm_campaign=winter_sale&fbclid=IwAR2AbCdEfGhIjKlMnOpQrStUvWxYz",
    "https://en.wikipedia.org/wiki/Asymmetric_numeral_systems",
    "https://en.wikipedia.org/wiki/Straight-line_program",
    "ipfs://bafybeic56t3rwfi6eza63q7hnvd6n2s6j7vg2nff22e7w273nzyuap243a",
    "magnet:?xt=urn:btih:d2b0018a1a3641b69ad312384a6c4df19910d65b&dn=Rust_Programming_Book_2026",
    "mailto:core-team@rust-lang.org?subject=Zero-Storage%20URL%20Compression",
    "bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.005&label=Satoshi",
  ];

  let mut blobs = Vec::new();
  let mut blob_hashes = Vec::new();

  for &url in &raw_links {
    let code = encode(url, Some(&profile)).expect("encode failed");
    let blob = Blob::new(code.into_bytes());
    blob_hashes.push(blob.hash);
    blobs.push(blob);
  }

  // 2. Build content-addressed Manifest pinning the prerequisite profile
  let manifest = Manifest::new(Some(profile.hash), blob_hashes.clone());
  assert_eq!(manifest.items.len(), raw_links.len());

  // 3. Export to portable .urls binary container
  let container_bytes = export(&profile.hash, &manifest, &blobs).expect("export container failed");
  assert!(!container_bytes.is_empty());

  // 4. Verify integrity seal (tamper detection)
  assert!(verify(&container_bytes), "container verification failed");

  // Write persistent sharable .urls file on disk
  let file_path = "/home/femar/Downloads/URL/collection.urls";
  fs::write(file_path, &container_bytes).expect("write .urls file failed");

  // 5. Simulate recipient peer loading the .urls file
  let loaded_bytes = fs::read(file_path).expect("read .urls file failed");
  assert!(verify(&loaded_bytes), "recipient verification failed");

  let (prereq, unpacked_manifest, unpacked_blobs) =
    open(&loaded_bytes).expect("open container failed");

  // Verify pinned prerequisite profile version
  assert_eq!(prereq, profile.hash);
  assert_eq!(unpacked_manifest.items.len(), raw_links.len());
  assert_eq!(unpacked_blobs.len(), raw_links.len());

  // 6. Recipient decodes all links in batch
  let codes: Vec<String> = unpacked_blobs
    .iter()
    .map(|b| String::from_utf8(b.data.clone()).unwrap())
    .collect();

  let code_refs: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
  let decoded_urls = batch(&code_refs).expect("batch decode failed");

  assert_eq!(decoded_urls.len(), raw_links.len());
  for (i, decoded) in decoded_urls.iter().enumerate() {
    assert_eq!(decoded, raw_links[i]);
  }

  // 7. CRDT G-Set merge test with a second peer's link collection
  let peer_raw_links = vec![
    "https://crates.io/crates/clap", // overlapping link
    "https://news.ycombinator.com/", // new link
  ];

  let mut peer_blobs = Vec::new();
  let mut peer_hashes = Vec::new();

  for &url in &peer_raw_links {
    let code = encode(url, Some(&profile)).expect("peer encode failed");
    let blob = Blob::new(code.into_bytes());
    peer_hashes.push(blob.hash);
    peer_blobs.push(blob);
  }

  let peer_manifest = Manifest::new(Some(profile.hash), peer_hashes);
  let merged_manifest = Merge::manifests(&manifest, &peer_manifest);

  // 18 initial + 2 peer (1 duplicate) = 19 unique content-addressed entries
  assert_eq!(merged_manifest.items.len(), 19);
}
