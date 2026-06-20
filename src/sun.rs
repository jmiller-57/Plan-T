use chrono::{ Timelike, Datelike, DateTime, Utc };
use crate::location::{ Aspect, Coordinates };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SunCover {
  Shade,
  Glancing,
  Sun,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SunPosition {
  /// Compass direction of the sun, in degrees `[0, 360)` (wraps at 360).
  /// 0 = North, 90 = East, 180 = South, 270 = West.
  /// N. hemisphere: rises ~east (90), peaks ~south (180) at solar noon, sets ~west (270).
  azimuth: f64,

  /// Height above the horizon, in degrees `[-90, 90]`.
  /// +90 = straight overhead (zenith); 0 = on the horizon (sunrise/sunset);
  /// negative = below the horizon (night); -90 = directly underfoot (nadir).
  /// elevation <= 0 means the sun is down — no direct light.
  elevation: f64,
}

impl SunPosition {
  pub fn azimuth(&self) -> f64 {
    self.azimuth
  }

  pub fn elevation(&self) -> f64 {
    self.elevation
  }

  pub fn calculate(coordinates: Coordinates, instant: DateTime<Utc>) -> SunPosition {
    let phi = coordinates.lat().to_radians();
    let h = Self::hour_angle(coordinates, instant).to_radians();
    let sigma = Self::declination(instant).to_radians();

    let elevation = ((phi.sin() * sigma.sin()) + (phi.cos() * sigma.cos() * h.cos())).asin().to_degrees();
    let elevation_rad = elevation.to_radians();
    let ratio = (sigma.sin() - (elevation_rad.sin() * phi.sin())) / (elevation_rad.cos() * phi.cos());
    let mut azimuth = ratio.clamp(-1.0, 1.0).acos().to_degrees();

    if h > 0.0 {
      azimuth = 360.0 - azimuth;
    }

    SunPosition { azimuth: azimuth, elevation: elevation }
  }

  fn declination(instant: DateTime<Utc>) -> f64 {
    let day = f64::from(instant.ordinal());
    // Cooper's formula
    23.45 * ((360.0 / 365.0) * (day + 284.0)).to_radians().sin()
  }

  fn equation_of_time(instant: DateTime<Utc>) -> f64 {
    let day = f64::from(instant.ordinal());
    let b = (360.0 / 365.0) * (day - 81.0); // degrees: need to convert to radians
    9.87 * ((2.0 * b).to_radians().sin()) - (7.53 * (b.to_radians().cos())) - (1.5 * (b.to_radians().sin()))
  }

  fn hour_angle(coordinates: Coordinates, instant: DateTime<Utc>) -> f64 {
    let utc_hours = f64::from(instant.hour()) + (f64::from(instant.minute()) / 60.0) + (f64::from(instant.second()) / 3600.0);
    let true_solar_time = utc_hours + (coordinates.long() / 15.0) + (Self::equation_of_time(instant) / 60.0);
    15.0 * (true_solar_time - 12.0)
  }

  pub fn cover_for(&self, aspect: Aspect) -> SunCover {
    if self.elevation() <= 0.0 { // sun is down
      return SunCover::Shade;
    } 

    let mut dir = (self.azimuth() - f64::from(aspect.heading())).abs();
    if dir > 180.0 {
      dir = 360.0 - dir;
    }
    
    let coverage = match dir {
      0.0..45.0 => SunCover::Sun,
      45.0..90.0 => SunCover::Glancing,
      _ => SunCover::Shade,
    };
    coverage
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::TimeZone;

  #[test]
  fn solstice_equinox() {
    let summer_solstice = Utc.with_ymd_and_hms(2026, 6, 21, 12, 0, 0).unwrap();
    let spring_equinox = Utc.with_ymd_and_hms(2026, 3, 20, 12, 0, 0).unwrap();
    let winter_solstice = Utc.with_ymd_and_hms(2026, 12, 21, 12, 0, 0).unwrap();

    assert!((SunPosition::declination(summer_solstice) - 23.4).abs() < 1.0);
    assert!((SunPosition::declination(spring_equinox) - 0.0).abs() < 1.0);
    assert!((SunPosition::declination(winter_solstice) + 23.4).abs() < 1.0);
  }

  #[test]
  fn validate_hour_angle() {
    let summer_solstice = Utc.with_ymd_and_hms(2026, 6, 21, 20, 0, 0).unwrap();
    let coords = Coordinates::new(38.9900167, -120.18235).unwrap();

    assert!((SunPosition::hour_angle(coords, summer_solstice)).abs() < 1.0);
  }

  #[test]
  fn validate_solar_noon_summer() {
    let summer_solstice = Utc.with_ymd_and_hms(2026, 6, 21, 20, 0, 0).unwrap();
    let coords = Coordinates::new(38.9900167, -120.18235).unwrap();
    let sp = SunPosition::calculate(coords, summer_solstice);
    assert!((sp.azimuth() - 177.75).abs() < 1.0);
    assert!((sp.elevation() - 74.44).abs() < 1.0);
  }

  #[test]
  fn validate_afternoon_calculation() {
    let afternoon = Utc.with_ymd_and_hms(2026, 6, 21, 23, 0, 0).unwrap();
    let coords = Coordinates::new(38.9900167, -120.18235).unwrap();
    let sp = SunPosition::calculate(coords, afternoon);
    assert!(sp.azimuth() > 180.0);
    assert!((sp.azimuth() - 260.81).abs() < 1.0);
    assert!((sp.elevation() - 49.51).abs() < 1.0);
  }

  #[test]
  fn validate_night_calculation() {
    let night = Utc.with_ymd_and_hms(2026, 6, 21, 6, 0, 0).unwrap();
    let coords = Coordinates::new(38.9900167, -120.18235).unwrap();
    let sp = SunPosition::calculate(coords, night);
    assert!(sp.elevation() < 0.0);
  }

  #[test]
  fn validate_sundown_shade() {
    let a = Aspect::new(180).unwrap();
    let sp = SunPosition { azimuth: 180.0, elevation: -45.0 };
    assert_eq!(sp.cover_for(a), SunCover::Shade);
  }

  #[test]
  fn validate_sun() {
    let a = Aspect::new(180).unwrap();
    let sp = SunPosition { azimuth: 180.0, elevation: 45.0 };
    assert_eq!(sp.cover_for(a), SunCover::Sun);
  }

  #[test]
  fn validate_glancing() {
    let a = Aspect::new(120).unwrap();
    let sp = SunPosition { azimuth: 180.0, elevation: 45.0 };
    assert_eq!(sp.cover_for(a), SunCover::Glancing);
  }

  #[test]
  fn validate_sun_up_shade() {
    let a = Aspect::new(0).unwrap();
    let sp = SunPosition { azimuth: 180.0, elevation: -45.0 };
    assert_eq!(sp.cover_for(a), SunCover::Shade);
  }

  #[test]
fn validate_wraparound_sun() {
    // sun just east of north (10), face points NNW (350): |10 - 350| = 340 -> folds to 20 -> Sun
    let sp = SunPosition { azimuth: 10.0, elevation: 45.0 };
    let a = Aspect::new(350).unwrap();
    assert_eq!(sp.cover_for(a), SunCover::Sun);
}

#[test]
fn validate_wraparound_shade() {
    // sun at 10, face points 280: |10 - 280| = 270 -> folds to 90 -> Shade (>= 90)
    let sp = SunPosition { azimuth: 10.0, elevation: 45.0 };
    let a = Aspect::new(280).unwrap();
    assert_eq!(sp.cover_for(a), SunCover::Shade);
}

#[test]
fn validate_dead_on_sun() {
    // sun and face both at 180: diff 0 -> Sun
    let sp = SunPosition { azimuth: 180.0, elevation: 45.0 };
    let a = Aspect::new(180).unwrap();
    assert_eq!(sp.cover_for(a), SunCover::Sun);
}

#[test]
fn validate_boundary_glancing() {
    // diff exactly 45 -> Glancing (Sun is < 45, Glancing is >= 45)
    let sp = SunPosition { azimuth: 180.0, elevation: 45.0 };
    let a = Aspect::new(135).unwrap();
    assert_eq!(sp.cover_for(a), SunCover::Glancing);
}

#[test]
fn validate_boundary_shade() {
    // diff exactly 90 -> Shade (Glancing is < 90)
    let sp = SunPosition { azimuth: 180.0, elevation: 45.0 };
    let a = Aspect::new(90).unwrap();
    assert_eq!(sp.cover_for(a), SunCover::Shade);
}
}