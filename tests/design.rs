use urls::design::{Error, Result, Score, Target};

#[test]
fn test_design_result_and_types() {
  let target = Target::new("https://example.com".to_string());
  assert_eq!(target.as_str(), "https://example.com");

  let score = Score::new(100, 30);
  assert!((score.ratio - 0.3).abs() < f32::EPSILON);

  let res: Result<u32> = Err(Error::Codec("invalid base symbol".to_string()));
  assert!(res.is_err());
}
