/// Calculate the percentage of a number
pub fn calculate_percentage(current: u64, total: u64) -> u64 {
  ((current as f64 / total as f64) * 100_f64).round() as u64
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_percentage() {
    assert_eq!(calculate_percentage(1, 10), 10);
    assert_eq!(calculate_percentage(5, 10), 50);
    assert_eq!(calculate_percentage(10, 10), 100);
  }
}
