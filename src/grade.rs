#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VGrade {
  VB,     //easier than V0
  V(u8),  // V0 - V18
}

impl VGrade {
  pub const MAX: u8 = 18;

  pub fn new(n: u8) -> Option<VGrade> {
    if n <= VGrade::MAX {
      Some(VGrade::V(n))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_out_of_range() {
    assert_eq!(VGrade::new(4), Some(VGrade::V(4)));
    assert_eq!(VGrade::new(50), None);
  }
}