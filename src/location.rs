#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
  lat: f64,
  long: f64,
}

/// The compass direction a rock face points, in whole degrees `[0, 359]`.
/// Same convention as solar azimuth, which is what makes them directly comparable:
/// 0 = North, 90 = East, 180 = South, 270 = West.
/// A south-facing wall (180) bakes in sun much of the day (N. hemisphere);
/// a north-facing wall (0) stays shady.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Aspect {
  heading: u16,
}

impl Coordinates {
  pub const LAT_MAX: f64 = 90.0;
  pub const LAT_MIN: f64 = -90.0;
  pub const LONG_MAX: f64 = 180.0;
  pub const LONG_MIN: f64 = -180.0;

  pub fn new(lat: f64, long: f64) -> Option<Coordinates> {
    if lat <= Self::LAT_MAX 
        && lat >= Self::LAT_MIN 
        && long <= Self::LONG_MAX 
        && long >= Self::LONG_MIN {
      Some(Coordinates { lat, long })
    } else {
      None
    }
  }

  pub fn lat(&self) -> f64 {
    self.lat
  }

  pub fn long(&self) -> f64 {
    self.long
  }
}

impl Aspect {
  pub const MAX: u16 = 359;

  pub fn new(heading: u16) -> Option<Aspect> {
    if heading <= Self::MAX {
      Some(Aspect { heading })
    } else {
      None
    }
  }

  pub fn heading(&self) -> u16 {
    self.heading
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_out_of_range() {
    assert!(Coordinates::new(-90.0, -180.0).is_some());
    assert!(Coordinates::new(90.0, 180.0).is_some());
    assert!(Coordinates::new(-90.1, -180.1).is_none());
    assert!(Coordinates::new(90.1, 180.1).is_none());
    assert!(Coordinates::new(-90.0, -180.1).is_none());
    assert!(Coordinates::new(90.0, 180.1).is_none());
    assert!(Coordinates::new(-90.1, -180.0).is_none());
    assert!(Coordinates::new(90.1, 180.0).is_none());
    assert!(Aspect::new(0).is_some());
    assert!(Aspect::new(359).is_some());
    assert!(Aspect::new(360).is_none());
  }
}