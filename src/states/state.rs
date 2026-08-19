/// Liveness state for target link health tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
  Unknown,
  Alive,
  Dead,
  Changed,
  Error,
}
