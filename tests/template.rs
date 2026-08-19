use urls::codecs::Template;
use urls::{decode, encode};

#[test]
fn test_template_extract_and_expand_amazon() {
  let template = Template::new();
  let url = "https://www.amazon.com/dp/B08N5WRWNW?ref=nb_sb_ss_custom_track&utm_source=newsletter&utm_medium=email&utm_campaign=summer_sale&utm_content=hero_cta&gclid=EAIaIQobChMI123456789";

  let (id, slots) = template
    .extract(url)
    .expect("failed to extract template slots");
  assert_eq!(id, 1);
  assert_eq!(slots.len(), 7);
  assert_eq!(slots[0], "B08N5WRWNW");
  assert_eq!(slots[1], "nb_sb_ss_custom_track");
  assert_eq!(slots[2], "newsletter");
  assert_eq!(slots[3], "email");
  assert_eq!(slots[4], "summer_sale");
  assert_eq!(slots[5], "hero_cta");
  assert_eq!(slots[6], "EAIaIQobChMI123456789");

  let packed = template.pack(id, &slots);
  let (unpacked_id, unpacked_slots) = template.unpack(&packed).expect("unpack failed");
  assert_eq!(unpacked_id, id);
  assert_eq!(unpacked_slots.len(), slots.len());

  let expanded = template
    .expand(unpacked_id, &unpacked_slots)
    .expect("expand failed");
  assert_eq!(expanded, url);
}

#[test]
fn test_template_extract_and_expand_steam() {
  let template = Template::new();
  let url = "https://store.steampowered.com/app/1086940/Baldurs_Gate_3/?utm_source=steam_store&utm_medium=featured&utm_campaign=winter_sale&fbclid=IwAR2AbCdEfGhIjKlMnOpQrStUvWxYz";

  let (id, slots) = template
    .extract(url)
    .expect("failed to extract template slots");
  assert_eq!(id, 2);
  assert_eq!(slots.len(), 6);
  assert_eq!(slots[0], "1086940");
  assert_eq!(slots[1], "Baldurs_Gate_3");

  let packed = template.pack(id, &slots);
  let (unpacked_id, unpacked_slots) = template.unpack(&packed).expect("unpack failed");
  let expanded = template
    .expand(unpacked_id, &unpacked_slots)
    .expect("expand failed");
  assert_eq!(expanded, url);
}

#[test]
fn test_template_encode_decode_roundtrip_all() {
  let test_urls = vec![
    "https://www.amazon.com/dp/B08N5WRWNW?ref=nb_sb_ss_custom_track&utm_source=newsletter&utm_medium=email&utm_campaign=summer_sale&utm_content=hero_cta&gclid=EAIaIQobChMI123456789",
    "https://store.steampowered.com/app/1086940/Baldurs_Gate_3/?utm_source=steam_store&utm_medium=featured&utm_campaign=winter_sale&fbclid=IwAR2AbCdEfGhIjKlMnOpQrStUvWxYz",
    "https://www.instagram.com/p/C9AbCdEfGhI/?igshid=MzRlODBiNWFlZA==",
    "https://www.reddit.com/r/rust/comments/123456/announcing_urls_v1_compression/",
    "https://www.linkedin.com/posts/rust-foundation_systems-programming-rustlang-activity-7123456789012345678-AbCd",
    "https://api.github.com/repos/rust-lang/rust/issues?state=open&sort=created&direction=desc",
    "https://github.com/rust-lang/rust/blob/master/compiler/rustc_middle/src/ty/context.rs",
    "https://github.com/rust-lang/rust/pull/12345",
    "mailto:core-team@rust-lang.org?subject=Zero-Storage%20URL%20Compression",
    "bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.005&label=Satoshi",
    "magnet:?xt=urn:btih:d2b0018a1a3641b69ad312384a6c4df19910d65b&dn=Rust_Programming_Book_2026",
  ];

  for url in test_urls {
    let code = encode(url, None).expect("encode failed");
    assert!(
      code.starts_with('3') || code.starts_with('0'),
      "expected template or generic tag"
    );
    let decoded = decode(&code).expect("decode failed");
    assert_eq!(decoded, url, "mismatch for url: {}", url);
  }
}
