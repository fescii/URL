use crate::hashes::digest;

/// Differential privacy noise generator for sanitizing shared frequency sketches.
pub struct Privacy;

impl Privacy {
  /// Add pseudo-random Laplace-like noise to counts based on privacy budget epsilon.
  pub fn sanitize_counts(counts: &[u32; 256], epsilon: f64) -> [u32; 256] {
    let eps = epsilon.max(0.01);
    let scale = (1.0 / eps).round() as i32;

    let mut sanitized = [1u32; 256];
    for i in 0..256 {
      // Generate deterministic bounded noise derived from seed
      let mut seed_buf = Vec::new();
      seed_buf.push(i as u8);
      seed_buf.extend_from_slice(&(counts[i]).to_le_bytes());
      let seed_hash = digest(&seed_buf);
      let raw_noise = seed_hash.bytes()[0] as i32 % (scale * 2 + 1) - scale;

      let noisy_val = (counts[i] as i32 + raw_noise).max(1) as u32;
      sanitized[i] = noisy_val;
    }

    crate::privacy!("sanitized 256 frequency counts with epsilon={}", eps);
    sanitized
  }
}
