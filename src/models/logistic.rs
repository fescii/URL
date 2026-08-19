/// Logistic mixing model combining order-N context predictors.
#[derive(Debug, Clone)]
pub struct Logistic {
  pub weights: Vec<f32>,
}

impl Logistic {
  pub const fn new(weights: Vec<f32>) -> Self {
    Self { weights }
  }

  /// Default balanced logistic mixer.
  pub fn default_mixer() -> Self {
    Self {
      weights: vec![0.5, 0.3, 0.2],
    }
  }

  /// Predict next symbol probability by weighted logistic mixing.
  pub fn mix(&self, probs: &[f32]) -> f32 {
    if probs.is_empty() {
      return 1.0 / 256.0;
    }
    let mut score = 0.0;
    let mut total_w = 0.0;
    for (i, &p) in probs.iter().enumerate() {
      let w = self.weights.get(i).copied().unwrap_or(1.0);
      score += p * w;
      total_w += w;
    }
    if total_w > 0.0 {
      score / total_w
    } else {
      probs[0]
    }
  }
}
