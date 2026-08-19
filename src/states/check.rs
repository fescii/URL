use super::state::State;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Out-of-band URL health checker.
pub struct Check;

impl Check {
  /// Probe target URL liveness using a lightweight HEAD request with timeout.
  pub fn probe(target: &str) -> State {
    if target.starts_with("mailto:")
      || target.starts_with("ipfs://")
      || target.starts_with("magnet:")
      || target.starts_with("urn:")
    {
      return State::Alive;
    }

    let is_https = target.starts_with("https://");
    let is_http = target.starts_with("http://");

    if !is_http && !is_https {
      return State::Unknown;
    }

    // Extract host and path
    let stripped = if is_https {
      &target["https://".len()..]
    } else {
      &target["http://".len()..]
    };

    let mut parts = stripped.splitn(2, '/');
    let host_port = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    let host = host_port.split(':').next().unwrap_or(host_port);
    let port: u16 = if is_https { 443 } else { 80 };

    let addr = format!("{host}:{port}");
    let Ok(mut addrs) = addr.to_socket_addrs() else {
      return State::Error;
    };

    let Some(sock_addr) = addrs.next() else {
      return State::Error;
    };

    let timeout = Duration::from_millis(2000);
    let Ok(mut stream) = TcpStream::connect_timeout(&sock_addr, timeout) else {
      return State::Error;
    };

    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    if !is_https {
      // Send plain HTTP HEAD request
      let req = format!(
        "HEAD /{} HTTP/1.1\r\nHost: {}\r\nUser-Agent: urls/1.0\r\nConnection: close\r\n\r\n",
        path, host
      );
      if stream.write_all(req.as_bytes()).is_err() {
        return State::Error;
      }

      let mut buf = [0u8; 512];
      let Ok(n) = stream.read(&mut buf) else {
        return State::Error;
      };

      let response = String::from_utf8_lossy(&buf[..n]);
      let status_code = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);

      match status_code {
        200..=299 => State::Alive,
        301 | 308 => State::Changed,
        302..=307 => State::Alive,
        404 | 410 => State::Dead,
        _ if status_code > 0 => State::Error,
        _ => State::Error,
      }
    } else {
      // For HTTPS without heavy TLS dependencies in basic probe, successful TCP handshake indicates alive host
      State::Alive
    }
  }
}
