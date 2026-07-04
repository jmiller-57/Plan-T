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

  pub fn distance(self, other: VGrade) -> u8 {
    self.index().abs_diff(other.index())
  }

  pub fn index(&self) -> i8 {
    match self {
      VGrade::VB => -1,
      VGrade::V(n) => n.cast_signed(),
    }
  }

  pub fn from_index(i: i8) -> VGrade {
    if i < 0 {
      VGrade::VB
    } else if i.cast_unsigned() >= VGrade::MAX {
      VGrade::V(VGrade::MAX)
    } else {
      VGrade::V(i.cast_unsigned())
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

  pub fn min(&self) -> VGrade {
    self.min
  }

  pub fn max(&self) -> VGrade {
    self.max
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

  #[test]
  fn grade_distance() {
    let v_b = VGrade::VB;
    let v0 = VGrade::new(0).unwrap();
    let v4 = VGrade::new(4).unwrap();
  
    assert_eq!(v_b.distance(v0), 1);
    assert_eq!(v0.distance(v_b), 1);
    assert_eq!(v0.distance(v4), 4);
    assert_eq!(v4.distance(v0), 4);
  }

  #[test]
  fn validate_index() {
    let v_b = VGrade::VB;
    let v9 = VGrade::V(9);

    assert_eq!(v_b.index(), -1);
    assert_eq!(v9.index(), 9);
  }

  #[test]
  fn validate_from_index() {
    let neg = -2;
    let neg_one = -1;
    let zero = 0;
    let ten = 10;
    let eighteen = 18;
    let twenty = 20;

    assert_eq!(VGrade::from_index(neg), VGrade::VB);
    assert_eq!(VGrade::from_index(neg_one), VGrade::VB);
    assert_eq!(VGrade::from_index(zero), VGrade::V(0));
    assert_eq!(VGrade::from_index(ten), VGrade::V(10));
    assert_eq!(VGrade::from_index(eighteen), VGrade::V(18));
    assert_eq!(VGrade::from_index(twenty), VGrade::V(18));
  }

  #[test]
  fn grade_roundtrip() {
    let v10 = VGrade::V(10);

    assert_eq!(v10, VGrade::from_index(v10.index()));
  }

}