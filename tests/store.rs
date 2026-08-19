use tempfile::tempdir;
use urls::hashes::Hash;
use urls::stores::{Cache, Shard, Store};

#[test]
fn test_shard_route() {
  let hash = Hash::new([10u8; 32]);
  assert_eq!(Shard::route(&hash, 4), 2);
}

#[test]
fn test_arc_cache_basic_and_capacity() {
  let mut cache = Cache::new(2);

  let k1 = Hash::new([1u8; 32]);
  let k2 = Hash::new([2u8; 32]);
  let k3 = Hash::new([3u8; 32]);

  cache.put(k1, bytes::Bytes::from_static(b"val1"));
  cache.put(k2, bytes::Bytes::from_static(b"val2"));
  assert_eq!(cache.len(), 2);

  // Hit on k1
  assert_eq!(cache.get(&k1), Some(bytes::Bytes::from_static(b"val1")));

  // Insert k3
  cache.put(k3, bytes::Bytes::from_static(b"val3"));
  assert_eq!(cache.len(), 2);
  assert!(cache.get(&k3).is_some());
}

#[test]
fn test_store_put_get_and_dedup() {
  let dir = tempdir().unwrap();
  let mut store = Store::open(dir.path()).expect("open store failed");

  let data1 = b"profile-structural-dictionary-blob";
  let data2 = b"compressed-url-shortcode-blob";

  let hash1 = store.put(data1).expect("put data1 failed");
  let hash2 = store.put(data2).expect("put data2 failed");

  // Exact point lookup
  let retrieved1 = store.get(&hash1).unwrap().expect("data1 missing");
  let retrieved2 = store.get(&hash2).unwrap().expect("data2 missing");

  assert_eq!(&retrieved1[..], data1);
  assert_eq!(&retrieved2[..], data2);

  // Dedup test
  let hash1_again = store.put(data1).unwrap();
  assert_eq!(hash1, hash1_again);
  assert_eq!(store.len(), 2);
}

#[test]
fn test_store_startup_recovery() {
  let dir = tempdir().unwrap();
  let hash;
  let payload = b"persistent-link-blob-12345";

  // 1. Write to store and close
  {
    let mut store = Store::open(dir.path()).expect("open initial store failed");
    hash = store.put(payload).expect("put failed");
    assert_eq!(store.len(), 1);
  }

  // 2. Re-open store from same directory and verify recovery
  {
    let mut store = Store::open(dir.path()).expect("reopen store failed");
    assert_eq!(store.len(), 1, "expected 1 entry recovered from logs");

    let retrieved = store.get(&hash).unwrap().expect("recovered entry missing");
    assert_eq!(&retrieved[..], payload);
  }
}
