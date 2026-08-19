use urls::hashes::digest;
use urls::objects::{Blob, Manifest, Merge};
use urls::{Profile, merge};

#[test]
fn test_blob_digest() {
  let b = Blob::new(b"hello".to_vec());
  assert_eq!(b.hash, digest(b"hello"));
}

#[test]
fn test_merge_manifests_gset_properties() {
  let h1 = digest(b"url1");
  let h2 = digest(b"url2");
  let h3 = digest(b"url3");

  let m1 = Manifest::new(None, vec![h1, h2]);
  let m2 = Manifest::new(None, vec![h2, h3]);

  // Commutativity: M1 u M2 == M2 u M1
  let merged_12 = m1.merge(&m2);
  let merged_21 = m2.merge(&m1);
  assert_eq!(merged_12, merged_21);

  // Idempotency: M1 u M1 == M1
  let merged_11 = m1.merge(&m1);
  assert_eq!(merged_11, m1);

  // Union elements
  assert_eq!(merged_12.items.len(), 3);
}

#[test]
fn test_gcounter_merge() {
  let f1 = vec![10u32; 256];
  let f2 = vec![20u32; 256];
  let merged = Merge::gcounter(&f1, &f2);

  for i in 0..256 {
    assert_eq!(merged[i], 30);
  }
}

#[test]
fn test_profile_merge() {
  let p1 = Profile::generic();
  let p2 = Profile::generic();

  let merged = merge(&[&p1, &p2]).expect("profile merge failed");
  let sum: u32 = merged.table.freqs.iter().sum();
  assert_eq!(sum, 4096);
}
