use bytes::Bytes;
use crate::hashes::Hash;
use std::collections::{HashMap, VecDeque};

/// Adaptive Replacement Cache (ARC) self-tuning between recency and frequency online.
/// Stores zero-copy `Bytes` slices — no allocation on cache hit.
pub struct Cache {
  capacity: usize,
  target: usize,
  t1: HashMap<Hash, Bytes>,
  t1_order: VecDeque<Hash>,
  t2: HashMap<Hash, Bytes>,
  t2_order: VecDeque<Hash>,
  b1: HashMap<Hash, ()>,
  b1_order: VecDeque<Hash>,
  b2: HashMap<Hash, ()>,
  b2_order: VecDeque<Hash>,
}

impl Cache {
  pub fn new(capacity: usize) -> Self {
    let cap = capacity.max(1);
    Self {
      capacity: cap,
      target: cap / 2,
      t1: HashMap::new(),
      t1_order: VecDeque::new(),
      t2: HashMap::new(),
      t2_order: VecDeque::new(),
      b1: HashMap::new(),
      b1_order: VecDeque::new(),
      b2: HashMap::new(),
      b2_order: VecDeque::new(),
    }
  }

  /// Cache hit — returns cloned `Bytes` handle (zero-copy reference count bump, no allocation).
  /// Promotes T1 hit to MRU of T2.
  pub fn get(&mut self, hash: &Hash) -> Option<Bytes> {
    if self.t1.contains_key(hash) {
      if let Some(pos) = self.t1_order.iter().position(|h| h == hash) {
        self.t1_order.remove(pos);
      }
      let data = self.t1.remove(hash).unwrap();
      self.t2.insert(*hash, data);
      self.t2_order.push_back(*hash);
      return self.t2.get(hash).cloned();
    }

    if self.t2.contains_key(hash) {
      if let Some(pos) = self.t2_order.iter().position(|h| h == hash) {
        self.t2_order.remove(pos);
      }
      self.t2_order.push_back(*hash);
      return self.t2.get(hash).cloned();
    }

    None
  }

  /// Insert zero-copy `Bytes` into the cache with ARC self-tuning adaptation.
  pub fn put(&mut self, hash: Hash, data: Bytes) {
    if self.t1.contains_key(&hash) {
      if let Some(pos) = self.t1_order.iter().position(|h| *h == hash) {
        self.t1_order.remove(pos);
      }
      self.t1.remove(&hash);
      self.t2.insert(hash, data);
      self.t2_order.push_back(hash);
      return;
    }
    if self.t2.contains_key(&hash) {
      if let Some(pos) = self.t2_order.iter().position(|h| *h == hash) {
        self.t2_order.remove(pos);
      }
      self.t2.insert(hash, data);
      self.t2_order.push_back(hash);
      return;
    }

    if self.b1.contains_key(&hash) {
      let delta = if !self.b1.is_empty() && !self.b2.is_empty() {
        (self.b2.len() / self.b1.len()).max(1)
      } else {
        1
      };
      self.target = (self.target + delta).min(self.capacity);
      self.replace(false);

      if let Some(pos) = self.b1_order.iter().position(|h| *h == hash) {
        self.b1_order.remove(pos);
      }
      self.b1.remove(&hash);
      self.t2.insert(hash, data);
      self.t2_order.push_back(hash);
      return;
    }

    if self.b2.contains_key(&hash) {
      let delta = if !self.b1.is_empty() && !self.b2.is_empty() {
        (self.b1.len() / self.b2.len()).max(1)
      } else {
        1
      };
      self.target = self.target.saturating_sub(delta);
      self.replace(true);

      if let Some(pos) = self.b2_order.iter().position(|h| *h == hash) {
        self.b2_order.remove(pos);
      }
      self.b2.remove(&hash);
      self.t2.insert(hash, data);
      self.t2_order.push_back(hash);
      return;
    }

    let l1_len = self.t1.len() + self.b1.len();
    if l1_len == self.capacity {
      if self.t1.len() < self.capacity {
        if let Some(b1_lru) = self.b1_order.pop_front() {
          self.b1.remove(&b1_lru);
        }
        self.replace(false);
      } else if let Some(t1_lru) = self.t1_order.pop_front() {
        self.t1.remove(&t1_lru);
      }
    } else if l1_len < self.capacity {
      let total_len = l1_len + self.t2.len() + self.b2.len();
      if total_len >= self.capacity {
        if total_len == 2 * self.capacity {
          if let Some(b2_lru) = self.b2_order.pop_front() {
            self.b2.remove(&b2_lru);
          }
        }
        self.replace(false);
      }
    }

    self.t1.insert(hash, data);
    self.t1_order.push_back(hash);
  }

  fn replace(&mut self, in_b2: bool) {
    if !self.t1.is_empty()
      && ((self.t1.len() > self.target) || (in_b2 && self.t1.len() == self.target))
    {
      if let Some(t1_lru) = self.t1_order.pop_front() {
        if let Some(_data) = self.t1.remove(&t1_lru) {
          self.b1.insert(t1_lru, ());
          self.b1_order.push_back(t1_lru);
        }
      }
    } else if let Some(t2_lru) = self.t2_order.pop_front() {
      if let Some(_data) = self.t2.remove(&t2_lru) {
        self.b2.insert(t2_lru, ());
        self.b2_order.push_back(t2_lru);
      }
    }
  }

  pub fn len(&self) -> usize {
    self.t1.len() + self.t2.len()
  }

  pub fn is_empty(&self) -> bool {
    self.t1.is_empty() && self.t2.is_empty()
  }
}
