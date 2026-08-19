use urls::hashes::digest;
use urls::stores::{Elias, Store, Succinct};

#[test]
fn test_elias_fano_monotone_sequence() {
  let seq = vec![0, 10, 25, 100, 150, 500, 1200, 5000, 10000];
  let elias = Elias::build(&seq);

  assert_eq!(elias.len(), seq.len());
  for (i, &expected) in seq.iter().enumerate() {
    let actual = elias.get(i);
    assert_eq!(actual, expected, "mismatch at index {}", i);
  }
}

#[test]
fn test_succinct_mphf_query_and_memory() {
  let mut entries = Vec::new();
  for i in 0..100 {
    let key = digest(format!("test_key_{i}").as_bytes());
    let offset = (i * 64) as u64;
    entries.push((key, offset));
  }

  let succinct = Succinct::build(entries.clone());
  assert_eq!(succinct.len(), 100);

  for (key, expected_offset) in &entries {
    let actual_offset = succinct.query(key).expect("key lookup failed in MPHF");
    assert_eq!(actual_offset, *expected_offset);
  }

  // Verify non-existent key returns None
  let missing_key = digest(b"non_existent_key_12345");
  assert!(succinct.query(&missing_key).is_none());

  // Memory verification: succinct index should be under 12 bytes per key (vs 64 bytes in HashMap)
  let bytes_per_key = succinct.memory_size() as f64 / succinct.len() as f64;
  assert!(
    bytes_per_key < 12.0,
    "expected succinct RAM footprint < 12 bytes/key, got {}",
    bytes_per_key
  );
}

#[test]
fn test_store_seal_and_succinct_lookup() {
  let temp_dir = std::env::temp_dir().join(format!(
    "urls_succinct_test_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let mut store = Store::open(&temp_dir).expect("open store failed");

  // Insert 50 entries
  let mut keys = Vec::new();
  for i in 0..50 {
    let key = format!("s_test_{i}");
    let val = format!("https://example.com/item/{i}?source=test");
    store.put_key(&key, val.as_bytes()).expect("put failed");
    keys.push((key, val));
  }

  let initial_ram = store.memory_size();

  // Seal store into succinct Minimal Perfect Hash indexes
  store.seal();

  let sealed_ram = store.memory_size();
  assert!(
    sealed_ram <= initial_ram,
    "sealed RAM should be smaller than mutable HashMap RAM"
  );

  // Verify all keys can still be retrieved losslessly from sealed succinct index
  for (key, expected_val) in keys {
    let fetched = store
      .get_key(&key)
      .expect("get failed")
      .expect("key missing after seal");
    assert_eq!(String::from_utf8(fetched.to_vec()).unwrap(), expected_val);
  }
}

#[test]
fn test_succinct_mphf_5000_keys() {
  let mut entries = Vec::new();
  for i in 0..5000 {
    let key = digest(format!("large_test_key_{i}").as_bytes());
    let offset = (i * 42) as u64;
    entries.push((key, offset));
  }

  let succinct = Succinct::build(entries.clone());
  assert_eq!(succinct.len(), 5000);

  for (i, (key, expected_offset)) in entries.iter().enumerate() {
    let actual_offset = succinct.query(key).unwrap_or_else(|| panic!("key {i} lookup failed"));
    assert_eq!(actual_offset, *expected_offset);
  }
}
