use urls::grammars::Repair;

#[test]
fn test_repair_roundtrip_repetitive() {
  let repair = Repair::new();
  let text = b"abcde_abcde_abcde_abcde_abcde_12345_12345_12345";
  let (symbols, rules) = repair.compress(text);
  assert!(!rules.is_empty(), "expected grammar rules to be generated");
  assert!(
    symbols.len() < text.len(),
    "expected symbol count reduction"
  );

  let restored = repair.decompress(&symbols, &rules);
  assert_eq!(&restored[..], &text[..]);
}

#[test]
fn test_repair_pack_unpack() {
  let repair = Repair::new();
  let text = b"https://example.com/api/v1/users/123/profile/api/v1/users/456/profile";
  let (symbols, rules) = repair.compress(text);

  let packed = repair.pack(&symbols, &rules);
  let (unpacked_symbols, unpacked_rules) = repair.unpack(&packed).unwrap();

  assert_eq!(symbols, unpacked_symbols);
  assert_eq!(rules, unpacked_rules);

  let restored = repair.decompress(&unpacked_symbols, &unpacked_rules);
  assert_eq!(&restored[..], &text[..]);
}
