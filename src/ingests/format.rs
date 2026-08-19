use std::path::Path;

/// Supported file formats for high-speed ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
  Auto,
  Csv,
  Tsv,
  Txt,
}

impl Format {
  /// Detect format from file extension or content snippet.
  pub fn detect<P: AsRef<Path>>(path: P) -> Self {
    let p = path.as_ref();
    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
      match ext.to_lowercase().as_str() {
        "csv" => Self::Csv,
        "tsv" => Self::Tsv,
        "txt" | "urls" | "list" => Self::Txt,
        _ => Self::Auto,
      }
    } else {
      Self::Auto
    }
  }

  /// Return field delimiter byte.
  pub fn delimiter(&self, sample: &str) -> u8 {
    match self {
      Self::Csv => b',',
      Self::Tsv => b'\t',
      Self::Txt => b'\n',
      Self::Auto => {
        if sample.contains('\t') {
          b'\t'
        } else if sample.contains(',') {
          b','
        } else {
          b'\n'
        }
      }
    }
  }
}
