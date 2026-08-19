use tempfile::tempdir;
use urls::{Profile, encode, resolve, train};

#[test]
fn test_profile_train_and_encode_roundtrip() {
  let corpus = vec![
    "https://github.com/rust-lang/rust",
    "https://github.com/rust-lang/cargo",
    "https://github.com/rust-lang/miri",
    "https://github.com/rust-lang/rust-analyzer",
  ];

  let profile = train(&corpus).expect("training failed");
  assert_ne!(profile.table.freqs.len(), 0);

  let test_url = "https://github.com/rust-lang/rust/pull/99999";
  let code = encode(test_url, Some(&profile)).expect("encode with trained profile failed");
  let decoded = resolve(&code, Some(&profile)).expect("resolve failed");

  assert_eq!(decoded, test_url);
}

#[test]
fn test_profile_save_and_load() {
  let dir = tempdir().unwrap();
  let file_path = dir.path().join("test.profile");

  let corpus = vec!["https://example.com/a", "https://example.com/b"];
  let profile = train(&corpus).unwrap();
  profile.save(&file_path).expect("save failed");

  let loaded = Profile::load_file(&file_path).expect("load failed");
  assert_eq!(loaded.table.freqs, profile.table.freqs);
}
