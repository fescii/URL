use super::request::Request;
use super::response::Response;
use crate::stores::Store;
use crate::{check, decode, encode, shorten, stat};
use std::sync::{Arc, Mutex};

/// Route dispatcher for redirection and REST API.
pub struct Router {
  pub store: Option<Arc<Mutex<Store>>>,
}

impl Router {
  pub fn new() -> Self {
    Self { store: None }
  }

  pub fn with_store(store: Arc<Mutex<Store>>) -> Self {
    Self { store: Some(store) }
  }

  /// Match incoming request and produce response.
  pub fn route(&self, req: &Request) -> Response {
    let method = req.method.as_str();
    let path = req.path.as_str();

    // 1. CORS Preflight
    if method == "OPTIONS" {
      let mut res = Response::new(204, "No Content");
      res
        .headers
        .insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
      res.headers.insert(
        "Access-Control-Allow-Methods".to_string(),
        "GET, POST, OPTIONS".to_string(),
      );
      res.headers.insert(
        "Access-Control-Allow-Headers".to_string(),
        "Content-Type".to_string(),
      );
      return res;
    }

    // 2. Health check
    if method == "GET" && path == "/health" {
      return Response::json("{\"status\":\"ok\"}");
    }

    // 3. API: POST /encode (or /shorten)
    if method == "POST" && (path == "/encode" || path == "/shorten") {
      let body = req.body_str();
      let url = parse_json_field(body, "url").unwrap_or_else(|| body.trim().to_string());

      if url.is_empty() {
        return Response::status(400, "Missing 'url' parameter");
      }

      let result = if path == "/shorten" && self.store.is_some() {
        let mut st_guard = self.store.as_ref().unwrap().lock().unwrap();
        shorten(&url, Some(&mut *st_guard))
      } else {
        encode(&url, None)
      };

      match result {
        Ok(code) => {
          crate::api!("encode endpoint: url={} -> code={}", url, code);
          Response::json(&format!(
            "{{\"url\":\"{}\",\"code\":\"{}\"}}",
            escape_json(&url),
            code
          ))
        }
        Err(e) => Response::status(400, &format!("Encoding error: {e}")),
      }
    }
    // 4. API: POST /decode
    else if method == "POST" && path == "/decode" {
      let body = req.body_str();
      let code = parse_json_field(body, "code").unwrap_or_else(|| body.trim().to_string());

      if code.is_empty() {
        return Response::status(400, "Missing 'code' parameter");
      }

      let result = if code.starts_with("s_") && self.store.is_some() {
        let mut st_guard = self.store.as_ref().unwrap().lock().unwrap();
        match st_guard.get_key(&code) {
          Ok(Some(bytes)) => resolve_payload(bytes),
          Ok(None) => Err(crate::Error::Store("shortcut not found".to_string())),
          Err(e) => Err(e),
        }
      } else {
        decode(&code)
      };

      match result {
        Ok(url) => {
          crate::api!("decode endpoint: code={} -> url={}", code, url);
          Response::json(&format!(
            "{{\"code\":\"{}\",\"url\":\"{}\"}}",
            code,
            escape_json(&url)
          ))
        }
        Err(e) => Response::status(400, &format!("Decoding error: {e}")),
      }
    }
    // 5. API: GET /stat/:code
    else if method == "GET" && path.starts_with("/stat/") {
      let code = &path["/stat/".len()..];
      if code.is_empty() {
        return Response::status(400, "Missing code in /stat/:code");
      }
      let (hits, state) = stat(code);
      Response::json(&format!(
        "{{\"code\":\"{}\",\"hits\":{},\"state\":\"{:?}\"}}",
        code, hits, state
      ))
    }
    // 6. API: GET /check?target=...
    else if method == "GET" && path == "/check" {
      let target = req.query.get("target").or_else(|| req.query.get("url"));
      if let Some(t) = target {
        let state = check(t);
        Response::json(&format!(
          "{{\"target\":\"{}\",\"state\":\"{:?}\"}}",
          escape_json(t),
          state
        ))
      } else {
        Response::status(400, "Missing '?target=' query parameter")
      }
    }
    // 7. Instant Redirection: GET /:code
    else if method == "GET" && path.len() > 1 {
      let code = &path[1..];
      let result = if code.starts_with("s_") && self.store.is_some() {
        let mut st_guard = self.store.as_ref().unwrap().lock().unwrap();
        match st_guard.get_key(code) {
          Ok(Some(bytes)) => resolve_payload(bytes),
          Ok(None) => Err(crate::Error::Store("shortcut not found".to_string())),
          Err(e) => Err(e),
        }
      } else {
        decode(code)
      };

      match result {
        Ok(url) => {
          crate::decode!("redirecting /{} -> {}", code, url);
          Response::redirect(&url)
        }
        Err(_) => Response::status(404, "Invalid or unknown shortcode"),
      }
    }
    // Fallback root or 404
    else if method == "GET" && path == "/" {
      Response::json("{\"name\":\"urls\",\"version\":\"1.0.0\",\"zero_storage\":true}")
    } else {
      Response::status(404, "Not Found")
    }
  }
}

impl Default for Router {
  fn default() -> Self {
    Self::new()
  }
}

fn resolve_payload(bytes: bytes::Bytes) -> crate::Result<String> {
  let s = std::str::from_utf8(&bytes)
    .map_err(|e| crate::Error::Codec(e.to_string()))?;
  Ok(s.to_string())
}

fn parse_json_field(json: &str, field: &str) -> Option<String> {
  let pattern = format!("\"{}\"", field);
  let pos = json.find(&pattern)?;
  let after = &json[pos + pattern.len()..];
  let colon_pos = after.find(':')?;
  let val_part = after[colon_pos + 1..].trim_start();

  if val_part.starts_with('"') {
    let end_quote = val_part[1..].find('"')?;
    Some(val_part[1..1 + end_quote].to_string())
  } else {
    None
  }
}

fn escape_json(s: &str) -> String {
  s.replace('\\', "\\\\").replace('"', "\\\"")
}
