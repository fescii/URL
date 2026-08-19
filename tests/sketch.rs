use urls::profiles::{Privacy, Sketch};

#[test]
fn test_count_min_sketch_record_and_query() {
  let mut sketch = Sketch::new(4, 256);

  let item_a = b"github.com";
  let item_b = b"crates.io";

  for _ in 0..10 {
    sketch.record(item_a);
  }
  for _ in 0..5 {
    sketch.record(item_b);
  }

  assert!(sketch.query(item_a) >= 10);
  assert!(sketch.query(item_b) >= 5);
  assert_eq!(sketch.query(b"unseen.domain"), 0);
}

#[test]
fn test_count_min_sketch_merge() {
  let mut s1 = Sketch::new(4, 256);
  let mut s2 = Sketch::new(4, 256);

  let item = b"example.org";
  for _ in 0..7 {
    s1.record(item);
  }
  for _ in 0..3 {
    s2.record(item);
  }

  s1.merge(&s2);
  assert!(s1.query(item) >= 10);
}

#[test]
fn test_differential_privacy_sanitizer() {
  let counts = [100u32; 256];
  let sanitized = Privacy::sanitize_counts(&counts, 1.0);

  for i in 0..256 {
    assert!(sanitized[i] > 0, "counts must remain positive");
  }
}
