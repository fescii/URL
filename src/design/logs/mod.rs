pub mod fmt;

#[doc(hidden)]
pub use tracing;

/// Initialize the logger with the custom subscriber and structured panic hook
pub fn init() {
  use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

  let layer = tracing_subscriber::fmt::layer()
    .event_format(fmt::Formatter)
    .with_writer(std::io::stdout);

  tracing_subscriber::registry()
    .with(filter)
    .with(layer)
    .init();

  // Install custom panic hook formatting panics using platform logs design
  std::panic::set_hook(Box::new(|info| {
    let location = info
      .location()
      .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
      .unwrap_or_else(|| "unknown".to_string());

    let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
      (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
      s.clone()
    } else {
      "Explicit panic invoked".to_string()
    };

    tracing::error!(target: "panic", "{} (at {})", payload, location);
  }));
}

// General Log Macros
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "info", $($arg)+) };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => { $crate::design::logs::tracing::debug!(target: "debug", $($arg)+) };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => { $crate::design::logs::tracing::trace!(target: "trace", $($arg)+) };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => { $crate::design::logs::tracing::error!(target: "error", $($arg)+) };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => { $crate::design::logs::tracing::warn!(target: "warn", $($arg)+) };
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "success", $($arg)+) };
}

// Encoding & Algorithms
#[macro_export]
macro_rules! encode {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "encode", $($arg)+) };
}

#[macro_export]
macro_rules! decode {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "decode", $($arg)+) };
}

#[macro_export]
macro_rules! grammar {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "grammar", $($arg)+) };
}

#[macro_export]
macro_rules! entropy {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "entropy", $($arg)+) };
}

#[macro_export]
macro_rules! model {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "model", $($arg)+) };
}

#[macro_export]
macro_rules! base {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "base", $($arg)+) };
}

// Objects & CRDTs
#[macro_export]
macro_rules! object {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "object", $($arg)+) };
}

#[macro_export]
macro_rules! blob {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "blob", $($arg)+) };
}

#[macro_export]
macro_rules! manifest {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "manifest", $($arg)+) };
}

#[macro_export]
macro_rules! merge {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "merge", $($arg)+) };
}

// Profiles & Atlas
#[macro_export]
macro_rules! profile {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "profile", $($arg)+) };
}

#[macro_export]
macro_rules! train {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "train", $($arg)+) };
}

#[macro_export]
macro_rules! stat {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "stat", $($arg)+) };
}

#[macro_export]
macro_rules! atlas {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "atlas", $($arg)+) };
}

#[macro_export]
macro_rules! sketch {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "sketch", $($arg)+) };
}

#[macro_export]
macro_rules! privacy {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "privacy", $($arg)+) };
}

// Storage Engine
#[macro_export]
macro_rules! store {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "store", $($arg)+) };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "log", $($arg)+) };
}

#[macro_export]
macro_rules! cache {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "cache", $($arg)+) };
}

#[macro_export]
macro_rules! index {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "index", $($arg)+) };
}

#[macro_export]
macro_rules! shard {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "shard", $($arg)+) };
}

// Containers & Files
#[macro_export]
macro_rules! container {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "container", $($arg)+) };
}

#[macro_export]
macro_rules! export {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "export", $($arg)+) };
}

#[macro_export]
macro_rules! open {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "open", $($arg)+) };
}

// State & Liveness
#[macro_export]
macro_rules! state {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "state", $($arg)+) };
}

#[macro_export]
macro_rules! check {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "check", $($arg)+) };
}

#[macro_export]
macro_rules! probe {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "probe", $($arg)+) };
}

// API & Verification
#[macro_export]
macro_rules! api {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "api", $($arg)+) };
}

#[macro_export]
macro_rules! cli {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "cli", $($arg)+) };
}

#[macro_export]
macro_rules! verify {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "verify", $($arg)+) };
}

#[macro_export]
macro_rules! ingest {
    ($($arg:tt)+) => { $crate::design::logs::tracing::info!(target: "ingest", $($arg)+) };
}
