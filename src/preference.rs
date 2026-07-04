#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preference {
  weight: f64,
  veto: bool,
}

impl Preference {
  pub const MAX_WEIGHT: f64 = 1.0;
  pub const MIN_WEIGHT: f64 = -1.0;

  pub fn new (weight: f64, veto: bool) -> Option<Preference> {
    if weight >= Preference::MIN_WEIGHT && weight <= Preference::MAX_WEIGHT {
      Some(Preference { weight, veto })
    } else {
      None
    }
  }

  pub fn weight(&self) -> f64 {
    self.weight
  }

  pub fn veto(&self) -> bool {
    self.veto
  } 
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn validates_weight_range() {
    assert_eq!(Preference::new(0.0, false), Some(Preference{weight: 0.0, veto: false}));
    assert_eq!(Preference::new(-1.0, false), Some(Preference{weight: -1.0, veto: false}));
    assert_eq!(Preference::new(1.0, false), Some(Preference{weight: 1.0, veto: false}));
    assert_eq!(Preference::new(-1.1, false), None);
    assert_eq!(Preference::new(1.1, false), None);
    assert_eq!(Preference::new(f64::NAN, false), None);
  }
}
