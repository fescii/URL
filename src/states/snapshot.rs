use super::state::State;

/// Point-in-time health snapshot for a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
  pub timestamp: u64,
  pub state: State,
  pub status: u16,
}

impl Snapshot {
  pub const fn new(timestamp: u64, state: State, status: u16) -> Self {
    Self {
      timestamp,
      state,
      status,
    }
  }
}
