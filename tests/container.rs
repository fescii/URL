use urls::containers::{MAGIC, Reader, Writer};
use urls::hashes::digest;
use urls::objects::{Blob, Manifest};

#[test]
fn test_magic_constant() {
  assert_eq!(MAGIC, b"URLS");
}

#[test]
fn test_container_pack_and_unpack_roundtrip() {
  let prereq = digest(b"test-profile-v1");
  let blob1 = Blob::new(b"0F6sBGtN-WgG95MqcYU8wAv".to_vec());
  let blob2 = Blob::new(b"0abcdef123456789".to_vec());

  let manifest = Manifest::new(Some(prereq), vec![blob1.hash, blob2.hash]);
  let blobs = vec![blob1.clone(), blob2.clone()];

  let packed = Writer::pack(&prereq, &manifest, &blobs).expect("pack container failed");
  assert!(!packed.is_empty());

  let (unpacked_prereq, unpacked_manifest, unpacked_blobs) =
    Reader::unpack(&packed).expect("unpack container failed");

  assert_eq!(unpacked_prereq, prereq);
  assert_eq!(unpacked_manifest.items.len(), 2);
  assert_eq!(unpacked_blobs.len(), 2);
  assert_eq!(unpacked_blobs[0].data, blob1.data);
  assert_eq!(unpacked_blobs[1].data, blob2.data);
}

#[test]
fn test_container_integrity_tamper_detection() {
  let prereq = digest(b"test-profile-v1");
  let blob = Blob::new(b"some_code".to_vec());
  let manifest = Manifest::new(Some(prereq), vec![blob.hash]);
  let mut packed = Writer::pack(&prereq, &manifest, &[blob]).unwrap();

  // Tamper with a payload byte in the middle
  let mid = packed.len() / 2;
  packed[mid] ^= 0xFF;

  let res = Reader::unpack(&packed);
  assert!(res.is_err(), "expected unpack to reject tampered container");
}
