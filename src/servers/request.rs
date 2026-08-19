use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// Parsed HTTP Request.
#[derive(Debug, Clone)]
pub struct Request {
  pub method: String,
  pub path: String,
  pub query: HashMap<String, String>,
  pub headers: HashMap<String, String>,
  pub body: Vec<u8>,
}

impl Request {
  /// Parse HTTP request from raw stream reader.
  pub fn parse<R: Read>(reader: &mut R) -> Option<Self> {
    let mut buf_reader = BufReader::new(reader);
    let mut request_line = String::new();

    if buf_reader.read_line(&mut request_line).ok()? == 0 {
      return None;
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
      return None;
    }

    let method = parts[0].to_uppercase();
    let full_path = parts[1];

    // Parse path and query parameters
    let mut path_query = full_path.splitn(2, '?');
    let path = path_query.next().unwrap_or("/").to_string();
    let query_str = path_query.next().unwrap_or("");

    let mut query = HashMap::new();
    if !query_str.is_empty() {
      for pair in query_str.split('&') {
        let mut kv = pair.splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        if !k.is_empty() {
          query.insert(k.to_string(), v.to_string());
        }
      }
    }

    // Parse headers
    let mut headers = HashMap::new();
    let mut content_length = 0usize;

    loop {
      let mut line = String::new();
      if buf_reader.read_line(&mut line).ok()? == 0 {
        break;
      }
      let trimmed = line.trim();
      if trimmed.is_empty() {
        break;
      }

      if let Some((k, v)) = trimmed.split_once(':') {
        let key = k.trim().to_lowercase();
        let val = v.trim().to_string();
        if key == "content-length" {
          content_length = val.parse().unwrap_or(0);
        }
        headers.insert(key, val);
      }
    }

    // Parse body if Content-Length > 0
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
      let _ = buf_reader.read_exact(&mut body);
    }

    Some(Self {
      method,
      path,
      query,
      headers,
      body,
    })
  }

  /// Helper to read body as UTF-8 string.
  pub fn body_str(&self) -> &str {
    std::str::from_utf8(&self.body).unwrap_or("")
  }
}
