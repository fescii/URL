use urls::states::{Check, Snapshot, State};

#[test]
fn test_state_snapshot() {
  let snap = Snapshot::new(1700000000, State::Alive, 200);
  assert_eq!(snap.state, State::Alive);
  assert_eq!(snap.status, 200);
}

#[test]
fn test_check_probe_non_http() {
  assert_eq!(Check::probe("mailto:test@example.com"), State::Alive);
  assert_eq!(
    Check::probe("ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"),
    State::Alive
  );
}
