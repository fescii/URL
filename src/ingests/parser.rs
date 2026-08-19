use super::format::Format;
use crate::design::{Error, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Streaming URL parser for CSV, TSV, and plain text files.
pub struct Parser {
  format: Format,
  col_idx: Option<usize>,
}

impl Parser {
  pub fn new(format: Format, col_idx: Option<usize>) -> Self {
    Self { format, col_idx }
  }

  /// Stream URLs from a file path using a large 1MB read buffer.
  pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
    let file = File::open(&path).map_err(Error::from)?;
    let reader = BufReader::with_capacity(1024 * 1024, file);
    self.parse_reader(reader)
  }

  /// Stream URLs from any buffered reader.
  pub fn parse_reader<R: BufRead>(&self, mut reader: R) -> Result<Vec<String>> {
    let mut line = String::with_capacity(1024);
    let mut urls = Vec::with_capacity(10_000);

    // Read first line to detect delimiter and header column index
    if reader.read_line(&mut line).map_err(Error::from)? == 0 {
      return Ok(urls);
    }

    let delim = self.format.delimiter(&line);
    let delim_char = delim as char;

    let mut url_col = self.col_idx;
    let mut is_header = false;

    if delim != b'\n' {
      let header_cols = Self::split_row(&line, delim_char);

      // 1. High-priority search: explicit URL columns
      for (idx, col) in header_cols.iter().enumerate() {
        let clean = col.trim().trim_matches('"').to_lowercase();
        if clean == "url"
          || clean == "urls"
          || clean == "link"
          || clean == "uri"
          || clean == "website"
        {
          url_col = Some(idx);
          is_header = true;
          break;
        }
      }

      // 2. Fallback search: domain column
      if url_col.is_none() {
        for (idx, col) in header_cols.iter().enumerate() {
          let clean = col.trim().trim_matches('"').to_lowercase();
          if clean == "domain" || clean == "domains" || clean == "host" {
            url_col = Some(idx);
            is_header = true;
            break;
          }
        }
      }

      if url_col.is_none() {
        // Default to last column
        url_col = Some(header_cols.len().saturating_sub(1));
      }
    }

    if !is_header {
      self.extract_url(&line, delim_char, url_col, &mut urls);
    }

    line.clear();
    while reader.read_line(&mut line).map_err(Error::from)? > 0 {
      self.extract_url(&line, delim_char, url_col, &mut urls);
      line.clear();
    }

    Ok(urls)
  }

  fn extract_url(&self, line: &str, delim: char, col_idx: Option<usize>, urls: &mut Vec<String>) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      return;
    }

    if delim == '\n' || col_idx.is_none() {
      urls.push(trimmed.trim_matches('"').to_string());
      return;
    }

    let idx = col_idx.unwrap();
    let cols = Self::split_row(trimmed, delim);
    if idx < cols.len() {
      let val = cols[idx].trim().trim_matches('"');
      if !val.is_empty() {
        let formatted = if !val.starts_with("http://")
          && !val.starts_with("https://")
          && !val.starts_with("ipfs://")
          && !val.starts_with("magnet:")
          && !val.starts_with("mailto:")
          && !val.starts_with("bitcoin:")
        {
          format!("https://{val}")
        } else {
          val.to_string()
        };
        urls.push(formatted);
      }
    }
  }

  /// Split a CSV/TSV row respecting quoted fields.
  fn split_row(line: &str, delim: char) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    let bytes = line.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
      if b == b'"' {
        in_quotes = !in_quotes;
      } else if b == delim as u8 && !in_quotes {
        fields.push(&line[start..i]);
        start = i + 1;
      }
    }

    if start <= line.len() {
      fields.push(&line[start..]);
    }

    fields
  }
}
