use crate::Server;
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
  /// Port to listen on
  #[arg(short, long, default_value_t = 8080)]
  pub port: u16,

  /// Host IP to bind
  #[arg(long, default_value = "127.0.0.1")]
  pub host: String,
}

pub fn run(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
  let addr = format!("{}:{}", cmd.host, cmd.port);
  let server = Server::bind(&addr)?;
  server.serve()?;
  Ok(())
}
