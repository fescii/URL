use std::io::Cursor;
use urls::ingests::{Batch, Format, Parser};
use urls::stores::Store;

#[test]
fn test_parser_csv_and_tsv() {
  let csv_data = "id,domain,url\n1,google.com,\"https://google.com/search?q=rust\"\n2,github.com,\"https://github.com/rust-lang/rust\"\n";
  let parser = Parser::new(Format::Csv, None);
  let urls = parser
    .parse_reader(Cursor::new(csv_data))
    .expect("parse failed");
  assert_eq!(urls.len(), 2);
  assert_eq!(urls[0], "https://google.com/search?q=rust");
  assert_eq!(urls[1], "https://github.com/rust-lang/rust");

  let tsv_data = "id\tdomain\tlink\n1\tcrates.io\thttps://crates.io/crates/clap\n";
  let parser_tsv = Parser::new(Format::Tsv, None);
  let urls_tsv = parser_tsv
    .parse_reader(Cursor::new(tsv_data))
    .expect("tsv parse failed");
  assert_eq!(urls_tsv.len(), 1);
  assert_eq!(urls_tsv[0], "https://crates.io/crates/clap");
}

#[test]
fn test_batch_ingest_and_seal() {
  let urls: Vec<String> = (0..200)
    .map(|i| format!("https://github.com/rust-lang/rust/pull/{i}"))
    .collect();

  let temp_dir = std::env::temp_dir().join(format!(
    "urls_ingest_test_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  let mut store = Store::open(&temp_dir).expect("open store failed");

  let stats = Batch::process(&urls, &mut store, 50).expect("batch process failed");
  assert_eq!(stats.count, 200);

  store.seal();
  assert!(store.len() > 0 && store.len() <= 200);

  let _ = std::fs::remove_dir_all(&temp_dir);
}
