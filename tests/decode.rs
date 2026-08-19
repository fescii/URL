use urls::{Profile, batch, decode, encode};

#[test]
fn test_decode_invalid_tag_prefix() {
  // Tag '$' is not in our recognized tag set
  let res = decode("$somebase66content");
  assert!(res.is_err(), "expected error on unrecognized tag prefix");
}

#[test]
fn test_decode_truncated_body() {
  // Single character '0' (valid tag, empty body)
  let res = decode("0");
  assert!(res.is_err(), "expected error on truncated payload");
}

#[test]
fn test_decode_invalid_base66_chars() {
  // Character '!' or ' ' is outside RFC 3986 unreserved alphabet
  let res = decode("0invalid!char");
  assert!(res.is_err(), "expected error on invalid character");
}

#[test]
fn test_decode_batch() {
  let profile = Profile::generic();
  let urls = &[
    "https://github.com/rust-lang/rust",
    "https://crates.io/crates/clap",
    "https://doc.rust-lang.org/book/",
  ];

  let codes: Vec<String> = urls
    .iter()
    .map(|&u| encode(u, Some(&profile)).unwrap())
    .collect();

  let code_refs: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
  let decoded = batch(&code_refs).unwrap();

  assert_eq!(decoded.len(), urls.len());
  for (i, &expected) in urls.iter().enumerate() {
    assert_eq!(decoded[i], expected);
  }
}
