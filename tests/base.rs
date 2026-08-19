use urls::codecs::{Base, Reader, Writer};

#[test]
fn test_base_roundtrip_empty() {
  let base = Base::new();
  let encoded = base.encode(b"");
  assert_eq!(encoded, "");
  let decoded = base.decode(&encoded).unwrap();
  assert_eq!(decoded, b"");
}

#[test]
fn test_base_roundtrip_basic() {
  let base = Base::new();
  let samples: &[&[u8]] = &[
    b"hello",
    b"https://example.com/path?query=1&utm_source=twitter",
    b"\x00\x00\x01\x02\x03\xff\xfe",
    b"1234567890",
    &[0u8; 32],
    &[255u8; 64],
  ];

  for sample in samples {
    let encoded = base.encode(sample);
    // Verify all characters are in RFC 3986 unreserved set
    for ch in encoded.chars() {
      assert!(
        ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' || ch == '_' || ch == '~',
        "character {ch} is not in RFC 3986 unreserved set"
      );
    }
    let decoded = base.decode(&encoded).unwrap();
    assert_eq!(&decoded[..], *sample);
  }
}

#[test]
fn test_bits_roundtrip() {
  let mut writer = Writer::new();
  writer.write(5, 3); // 101 (3 bits)
  writer.write(1, 1); // 1 (1 bit)
  writer.write(0xAB, 8); // 10101011 (8 bits)
  writer.write(0x1234, 16); // 16 bits
  let packed = writer.finish();

  let mut reader = Reader::new(&packed);
  assert_eq!(reader.read(3).unwrap(), 5);
  assert_eq!(reader.read(1).unwrap(), 1);
  assert_eq!(reader.read(8).unwrap(), 0xAB);
  assert_eq!(reader.read(16).unwrap(), 0x1234);
}
