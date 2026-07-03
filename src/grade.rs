#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VGrade {
  VB,     //easier than V0
  V(u8),  // V0 - V18
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GradeRange {
  min: VGrade,
  max: VGrade,
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

impl GradeRange {
  pub fn new(min: VGrade, max: VGrade) -> Option<GradeRange> {
    if min <= max {
      Some(GradeRange { min, max })
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

  #[test]
  fn reject_invalid_grade_range() {
    let v4 = VGrade::new(4).unwrap();
    let v8 = VGrade::new(8).unwrap();
    assert_eq!(GradeRange::new(v4, v8), Some(GradeRange{min: v4, max: v8}));
    assert_eq!(GradeRange::new(v8, v4), None);
    assert_eq!(GradeRange::new(v4, v4), Some(GradeRange{min: v4, max: v4}));
  }
}