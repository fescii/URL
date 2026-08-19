use urls::entropies::{Markov, Mtf};

#[test]
fn test_mtf_roundtrip_all() {
  let mtf = Mtf::new();
  let sample = b"https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share";
  let encoded = mtf.encode(sample);
  let decoded = mtf.decode(&encoded);
  assert_eq!(decoded, sample);
}

#[test]
fn test_mtf_empty() {
  let mtf = Mtf::new();
  let sample = b"";
  let encoded = mtf.encode(sample);
  let decoded = mtf.decode(&encoded);
  assert_eq!(decoded, sample);
}

#[test]
fn test_markov_empty() {
  let markov = Markov::new();
  let sample = b"";
  let encoded = markov.encode(sample).expect("encode failed");
  let decoded = markov.decode(&encoded, 0).expect("decode failed");
  assert_eq!(decoded, sample);
}
