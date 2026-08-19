use std::collections::HashMap;
use std::io::{self, Write};

/// HTTP Response builder.
#[derive(Debug, Clone)]
pub struct Response {
  pub status: u16,
  pub reason: String,
  pub headers: HashMap<String, String>,
  pub body: Vec<u8>,
}

impl Response {
  pub fn new(status: u16, reason: &str) -> Self {
    let mut headers = HashMap::new();
    headers.insert("Server".to_string(), "urls/1.0".to_string());
    headers.insert("Connection".to_string(), "close".to_string());

    Self {
      status,
      reason: reason.to_string(),
      headers,
      body: Vec::new(),
    }
  }

  /// 302 Found Redirection response with Location header.
  pub fn redirect(location: &str) -> Self {
    let mut res = Self::new(302, "Found");
    res
      .headers
      .insert("Location".to_string(), location.to_string());
    res.headers.insert(
      "Cache-Control".to_string(),
      "public, max-age=31536000, immutable".to_string(),
    );
    res.body = format!("Redirecting to {location}\n").into_bytes();
    res
  }

  /// 200 OK JSON response.
  pub fn json(body: &str) -> Self {
    let mut res = Self::new(200, "OK");
    res
      .headers
      .insert("Content-Type".to_string(), "application/json".to_string());
    res.body = body.as_bytes().to_vec();
    res
  }

  /// 200 OK Plain Text response.
  pub fn text(body: &str) -> Self {
    let mut res = Self::new(200, "OK");
    res.headers.insert(
      "Content-Type".to_string(),
      "text/plain; charset=utf-8".to_string(),
    );
    res.body = body.as_bytes().to_vec();
    res
  }

  /// Generic error or status response.
  pub fn status(code: u16, msg: &str) -> Self {
    let reason = match code {
      400 => "Bad Request",
      404 => "Not Found",
      405 => "Method Not Allowed",
      500 => "Internal Server Error",
      _ => "Status",
    };
    let mut res = Self::new(code, reason);
    res
      .headers
      .insert("Content-Type".to_string(), "application/json".to_string());
    res.body = format!("{{\"error\":\"{msg}\",\"code\":{code}}}\n").into_bytes();
    res
  }

  /// Write full HTTP response stream to writer.
  pub fn send<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let status_line = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
    writer.write_all(status_line.as_bytes())?;

    let mut headers = self.headers.clone();
    headers.insert("Content-Length".to_string(), self.body.len().to_string());

    for (k, v) in &headers {
      let header_line = format!("{k}: {v}\r\n");
      writer.write_all(header_line.as_bytes())?;
    }

    writer.write_all(b"\r\n")?;
    if !self.body.is_empty() {
      writer.write_all(&self.body)?;
    }
    writer.flush()?;
    Ok(())
  }
}
