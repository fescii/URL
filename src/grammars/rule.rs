/// Single grammar rule replacing an adjacent symbol pair with a new non-terminal symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rule {
  pub id: u16,
  pub left: u16,
  pub right: u16,
}

impl Rule {
  pub const fn new(id: u16, left: u16, right: u16) -> Self {
    Self { id, left, right }
  }
}
