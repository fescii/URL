use chrono::Local;
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
  fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
  registry::LookupSpan,
};

pub struct Formatter;

impl<S, N> FormatEvent<S, N> for Formatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    _ctx: &FmtContext<'_, S, N>,
    mut writer: Writer<'_>,
    event: &Event<'_>,
  ) -> fmt::Result {
    let meta = event.metadata();
    let target = meta.target();

    let color = get_color(target, meta.level());
    let reset = "\x1b[0m";
    let gray = "\x1b[90m";

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    // Write timestamp
    write!(writer, "{}[{}]{} ", gray, timestamp, reset)?;

    // Determine label to print
    let label = match target {
      t if t.starts_with("urls::design::logs::") || t.starts_with("logs::") => t
        .replace("urls::design::logs::", "")
        .replace("logs::", "")
        .to_uppercase(),
      t if is_known_category(t) => t.to_uppercase(),
      _ => meta.level().as_str().to_uppercase(),
    };

    write!(writer, "{}{}:{} ", color, label, reset)?;

    // Use visitor to print event fields nicely
    let mut visitor = Visitor {
      writer: &mut writer,
      fields: false,
    };
    event.record(&mut visitor);
    drop(visitor);

    writeln!(writer)
  }
}

fn is_known_category(t: &str) -> bool {
  matches!(
    t,
    "info"
      | "debug"
      | "trace"
      | "error"
      | "warn"
      | "success"
      | "panic"
      | "encode"
      | "decode"
      | "grammar"
      | "entropy"
      | "model"
      | "base"
      | "object"
      | "blob"
      | "manifest"
      | "merge"
      | "profile"
      | "train"
      | "stat"
      | "atlas"
      | "sketch"
      | "privacy"
      | "store"
      | "index"
      | "log"
      | "cache"
      | "shard"
      | "container"
      | "export"
      | "open"
      | "state"
      | "check"
      | "probe"
      | "api"
      | "cli"
      | "verify"
  )
}

fn get_color(target: &str, level: &tracing::Level) -> &'static str {
  match target {
    "info" => "\x1b[34m",                 // blue
    "debug" => "\x1b[90m",                // gray
    "trace" => "\x1b[90m",                // gray
    "success" => "\x1b[1m\x1b[32m",       // bold green
    "panic" => "\x1b[1m\x1b[41m\x1b[37m", // white on red background
    "error" => "\x1b[1m\x1b[31m",         // bold red
    "warn" => "\x1b[1m\x1b[33m",          // bold yellow

    // Encoding & Algorithms
    "encode" | "decode" => "\x1b[36m",   // cyan
    "grammar" | "entropy" => "\x1b[96m", // bright cyan
    "model" | "base" => "\x1b[94m",      // bright blue

    // Objects & CRDTs
    "object" | "blob" | "manifest" => "\x1b[35m", // magenta
    "merge" => "\x1b[95m",                        // bright magenta

    // Profiles & Atlas
    "profile" | "train" | "atlas" => "\x1b[33m", // yellow
    "sketch" | "privacy" => "\x1b[93m",          // bright yellow

    // Storage Engine
    "store" | "log" => "\x1b[32m",             // green
    "cache" | "index" | "shard" => "\x1b[92m", // bright green

    // Container & CLI
    "container" | "export" | "open" => "\x1b[34m", // blue
    "state" | "stat" | "check" | "probe" => "\x1b[91m", // bright red
    "api" | "cli" | "verify" => "\x1b[97m",        // bright white

    _ => match *level {
      tracing::Level::ERROR => "\x1b[1m\x1b[31m",
      tracing::Level::WARN => "\x1b[1m\x1b[33m",
      tracing::Level::INFO => "\x1b[34m",
      tracing::Level::DEBUG => "\x1b[90m",
      tracing::Level::TRACE => "\x1b[90m",
    },
  }
}

struct Visitor<'a, 'w> {
  writer: &'a mut Writer<'w>,
  fields: bool,
}

impl<'a, 'w> tracing::field::Visit for Visitor<'a, 'w> {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
    if field.name() == "message" {
      let s = format!("{:?}", value);
      if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let _ = write!(self.writer, "{}", &s[1..s.len() - 1]);
      } else {
        let _ = write!(self.writer, "{}", s);
      }
    } else {
      if !self.fields {
        let _ = write!(self.writer, " \x1b[90m");
        self.fields = true;
      } else {
        let _ = write!(self.writer, " ");
      }
      let _ = write!(self.writer, "{}={:?}", field.name(), value);
    }
  }
}

impl<'a, 'w> Drop for Visitor<'a, 'w> {
  fn drop(&mut self) {
    if self.fields {
      let _ = write!(self.writer, "\x1b[0m");
    }
  }
}
