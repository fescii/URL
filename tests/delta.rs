use urls::stores::{Cluster, Delta, Symbol};

#[test]
fn test_positional_delta_amora_omera() {
  let delta = Delta::new();
  let anchor = b"amora";
  let target = b"omera";

  let record = delta.diff(0, anchor, target);
  assert_eq!(record.anchor, 0);
  assert_eq!(record.target_len, 5);
  // In "omera" vs "amora", indices 1 ('m'), 3 ('r'), 4 ('a') match -> bits 1, 3, 4 = 0b11010 = 26
  assert_eq!(record.mask[0], 26);
  // Diffs only contain the 2 differing characters: 'o' and 'e'
  assert_eq!(record.diffs, vec![b'o', b'e']);

  // Lossless reconstruction
  let restored = delta.apply(anchor, &record).expect("apply failed");
  assert_eq!(restored, target);
}

#[test]
fn test_positional_delta_identical() {
  let delta = Delta::new();
  let anchor = b"https://github.com/rust-lang/rust/pull/12345";
  let target = b"https://github.com/rust-lang/rust/pull/12345";

  let record = delta.diff(1, anchor, target);
  assert!(record.diffs.is_empty());
  let restored = delta.apply(anchor, &record).expect("apply failed");
  assert_eq!(restored, target);
}

#[test]
fn test_positional_delta_random_substitutions() {
  let delta = Delta::new();
  let anchor = b"0ODifZ9l45xAuU78~FepkFeLDwh.Zbwjkdv4hSGYRifF_VW7r";
  let target = b"0ODifZ9l45xAuU78~FepkFeLDwh.Zbwjkdv4hSGYRifF_ABCD";

  let record = delta.diff(2, anchor, target);
  let restored = delta.apply(anchor, &record).expect("apply failed");
  assert_eq!(restored, target);
}

#[test]
fn test_cluster_anchor_registration_and_locate() {
  let mut cluster = Cluster::new();
  let a1 = b"https://github.com/rust-lang/rust/pull/100";
  let a2 = b"https://github.com/rust-lang/rust/pull/200";
  let a3 = b"https://amazon.com/dp/B08N5WRWNW";

  assert!(cluster.locate(a1).is_none()); // First entry registered as Anchor 0
  let located = cluster.locate(a2);
  assert!(located.is_some());
  assert_eq!(located.unwrap().0, 0); // Matched Anchor 0

  let located_amazon = cluster.locate(a3);
  assert!(located_amazon.is_none()); // New domain registered as Anchor 1
}

#[test]
fn test_symbol_fsst_compression_roundtrip() {
  let symbol = Symbol::new();
  let sample = b"https://www.github.com/rust-lang/rust/pull/12345?utm_source=newsletter&gclid=123";
  let compressed = symbol.compress(sample);
  assert!(compressed.len() < sample.len());
  let decompressed = symbol.decompress(&compressed);
  assert_eq!(decompressed, sample);
}
