use urls::entropies::{Rans, Table};

#[test]
fn test_rans_roundtrip_uniform() {
  let rans = Rans::new();
  let table = Table::uniform();
  let sample = b"https://example.com/hello-world?query=123";

  let encoded = rans.encode(sample, &table);
  let decoded = rans.decode(&encoded, sample.len(), &table).unwrap();

  assert_eq!(&decoded[..], &sample[..]);
}

#[test]
fn test_rans_roundtrip_skewed() {
  let rans = Rans::new();
  let sample = b"aaaaaaaaaabbbbbbbbccccdddd";

  let mut counts = [1u32; 256];
  for &b in sample {
    counts[b as usize] += 10;
  }
  let table = Table::from_counts(&counts);

  let encoded = rans.encode(sample, &table);
  let decoded = rans.decode(&encoded, sample.len(), &table).unwrap();

  assert_eq!(&decoded[..], &sample[..]);
}
