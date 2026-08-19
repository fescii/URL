use super::request::Request;
use super::router::Router;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// High-performance embedded HTTP server for URL redirection and REST API.
pub struct Server {
  listener: TcpListener,
  router: Arc<Router>,
}

impl Server {
  /// Bind server to specified address (e.g. "127.0.0.1:8080" or "0.0.0.0:3000").
  pub fn bind(addr: &str) -> io::Result<Self> {
    let listener = TcpListener::bind(addr)?;
    let router = Arc::new(Router::new());
    Ok(Self { listener, router })
  }

  /// Retrieve local bound socket address (useful for dynamic port allocation in tests).
  pub fn local_addr(&self) -> io::Result<SocketAddr> {
    self.listener.local_addr()
  }

  /// Run the server loop accepting incoming connections.
  pub fn serve(&self) -> io::Result<()> {
    let local_addr = self.listener.local_addr()?;
    crate::api!("HTTP server listening on http://{}", local_addr);

    for stream in self.listener.incoming() {
      match stream {
        Ok(stream) => {
          let router = Arc::clone(&self.router);
          thread::spawn(move || {
            let _ = Self::handle_connection(stream, router);
          });
        }
        Err(e) => {
          crate::error!("error accepting connection: {}", e);
        }
      }
    }
    Ok(())
  }

  fn handle_connection(mut stream: TcpStream, router: Arc<Router>) -> io::Result<()> {
    let timeout = Duration::from_secs(5);
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    if let Some(req) = Request::parse(&mut stream) {
      let res = router.route(&req);
      res.send(&mut stream)?;
    }

    Ok(())
  }
}
