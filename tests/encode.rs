use urls::{Profile, decode, encode};

#[test]
fn test_encode_decode_roundtrip_empty() {
  let code = encode("", None).unwrap();
  assert_eq!(code, "");
  let restored = decode(&code).unwrap();
  assert_eq!(restored, "");
}

#[test]
fn test_encode_decode_roundtrip_atlas_urls() {
  let profile = Profile::generic();
  let urls = &[
    "https://github.com/rust-lang/rust/pull/12345",
    "https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=twitter&utm_medium=social",
    "https://x.com/rustlang/status/1234567890123456789",
    "https://medium.com/engineering/deep-dive-into-asymmetric-numeral-systems",
    "https://api.whatsapp.com/send?phone=254700000000&text=Hello%20World",
    "ipfs://bafybeic56whw5n22n66m6pjh4v3b6n57f4n",
    "bitcoin:1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa?amount=0.5",
    "https://example.com/api/v1/users/a1b2c3d4-e5f6-7890-abcd-ef1234567890/profile?ref=github&gclid=CjwKCAjw",
    "https://wikipedia.org/wiki/Lossless_compression",
  ];

  for &original in urls {
    let code = encode(original, Some(&profile)).expect("encoding failed");
    assert!(!code.is_empty(), "code should not be empty");

    // Verify all code chars are valid unreserved characters
    for ch in code.chars() {
      assert!(
        ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' || ch == '_' || ch == '~',
        "character '{ch}' is outside unreserved set"
      );
    }

    let restored = decode(&code).expect("decoding failed");
    assert_eq!(
      restored, original,
      "mismatch for URL '{original}' != restored '{restored}'"
    );
  }
}
