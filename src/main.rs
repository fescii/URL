#![allow(dead_code, unused_variables)]

use clap::{Parser, Subcommand};
use urls::commands::{
  check, decode, encode, export, ingest, open, report, serve, stat, train, verify,
};

#[derive(Parser, Debug)]
#[command(name = "urls")]
#[command(
  about = "Zero-storage URL compressor and portable link container",
  long_about = None
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Compress a URL into a zero-storage shortcode
  Encode(encode::Command),
  /// Reconstruct original URL from a shortcode
  Decode(decode::Command),
  /// Train profile artifacts from a URL corpus
  Train(train::Command),
  /// Ingest large CSV, TSV, or URL list files into store
  Ingest(ingest::Command),
  /// Run multi-scale benchmark and generate full CSV and Markdown reports
  Report(report::Command),
  /// Package link entries into a .urls container file
  Export(export::Command),
  /// Open and decode a .urls container file
  Open(open::Command),
  /// Inspect frequency and liveness state of a shortcode
  Stat(stat::Command),
  /// Probe target link reachability out-of-band
  Check(check::Command),
  /// Verify file or object hash integrity
  Verify(verify::Command),
  /// Start high-performance redirection and REST API HTTP server
  Serve(serve::Command),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  urls::design::logs::init();

  let cli = Cli::parse();
  match cli.command {
    Commands::Encode(cmd) => encode::run(cmd),
    Commands::Decode(cmd) => decode::run(cmd),
    Commands::Train(cmd) => train::run(cmd),
    Commands::Ingest(cmd) => ingest::run(cmd),
    Commands::Report(cmd) => report::run(cmd),
    Commands::Export(cmd) => export::run(cmd),
    Commands::Open(cmd) => open::run(cmd),
    Commands::Stat(cmd) => stat::run(cmd),
    Commands::Check(cmd) => check::run(cmd),
    Commands::Verify(cmd) => verify::run(cmd),
    Commands::Serve(cmd) => serve::run(cmd),
  }
}
