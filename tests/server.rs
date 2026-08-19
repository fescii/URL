use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use urls::encode;
use urls::servers::Server;

#[test]
fn test_http_server_end_to_end() {
  let server = Server::bind("127.0.0.1:0").expect("bind server failed");
  let addr = server.local_addr().expect("get local addr failed");

  // Spawn server thread
  thread::spawn(move || {
    let _ = server.serve();
  });

  thread::sleep(Duration::from_millis(100));

  // 1. Test GET /health
  {
    let mut stream = TcpStream::connect(addr).expect("connect to server failed");
    let req = "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(req.as_bytes()).unwrap();

    let mut res = String::new();
    stream.read_to_string(&mut res).unwrap();
    assert!(res.starts_with("HTTP/1.1 200 OK"), "got: {}", res);
    assert!(res.contains("{\"status\":\"ok\"}"), "got: {}", res);
  }

  // 2. Test Zero-Storage Redirection: GET /:code -> 302 Found
  let target_url = "https://github.com/rust-lang/rust/pull/42";
  let code = encode(target_url, None).expect("encode failed");

  {
    let mut stream = TcpStream::connect(addr).expect("connect to server failed");
    let req = format!("GET /{} HTTP/1.1\r\nHost: localhost\r\n\r\n", code);
    stream.write_all(req.as_bytes()).unwrap();

    let mut res = String::new();
    stream.read_to_string(&mut res).unwrap();

    assert!(res.starts_with("HTTP/1.1 302 Found"), "got: {}", res);
    assert!(
      res.contains(&format!("Location: {}", target_url)),
      "expected Location header matching target, got: {}",
      res
    );
  }

  // 3. Test POST /encode
  {
    let mut stream = TcpStream::connect(addr).expect("connect to server failed");
    let body = format!("{{\"url\":\"{}\"}}", target_url);
    let req = format!(
      "POST /encode HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
      body.len(),
      body
    );
    stream.write_all(req.as_bytes()).unwrap();

    let mut res = String::new();
    stream.read_to_string(&mut res).unwrap();

    assert!(res.starts_with("HTTP/1.1 200 OK"), "got: {}", res);
    assert!(
      res.contains(&format!("\"code\":\"{}\"", code)),
      "code was: {}, got res: {}",
      code,
      res
    );
  }

  // 4. Test POST /decode
  {
    let mut stream = TcpStream::connect(addr).expect("connect to server failed");
    let body = format!("{{\"code\":\"{}\"}}", code);
    let req = format!(
      "POST /decode HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
      body.len(),
      body
    );
    stream.write_all(req.as_bytes()).unwrap();

    let mut res = String::new();
    stream.read_to_string(&mut res).unwrap();

    assert!(res.starts_with("HTTP/1.1 200 OK"), "got: {}", res);
    assert!(res.contains(target_url), "got: {}", res);
  }
}
