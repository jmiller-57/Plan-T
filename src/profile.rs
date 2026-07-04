use crate::{ 
  grade::VGrade,
  feature::Feature,
  preference::Preference
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ClimberProfile {
  name: String,

  /// measured as an integer in inches
  height: Option<u8>,

  /// measured as an integer in inches
  wingspan: Option<u8>,
  flash_grade: VGrade,
  max_grade: VGrade,
  default_feature_preferences: HashMap<Feature, Preference>,
}

impl ClimberProfile {
  pub fn new(
      name: String
      ,height: Option<u8>
      ,wingspan: Option<u8>
      ,flash_grade: VGrade
      ,max_grade: VGrade) -> Self {
    Self { name, height, wingspan, max_grade, flash_grade, default_feature_preferences: HashMap::new() }
  } 

  pub fn set_feature_preference(&mut self, feat: Feature, pref: Preference) {
    self.default_feature_preferences.insert(feat, pref);
  }

  pub fn feature_preference(&self, feat: Feature) -> Option<&Preference> {
    self.default_feature_preferences.get(&feat)
  }

  pub fn default_feature_preferences(&self) -> &HashMap<Feature, Preference> {
    &self.default_feature_preferences
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn height(&self) -> Option<u8> {
    self.height
  }

  pub fn wingspan(&self) -> Option<u8> {
    self.wingspan
  }

  pub fn flash_grade(&self) -> VGrade {
    self.flash_grade
  }

  pub fn max_grade(&self) -> VGrade {
    self.max_grade
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validate_feature_preferences() {
    let p = Preference::new(1.0, false).unwrap();
    let mut cp = ClimberProfile::new(String::from("Jonny"), Some(71), Some(73), VGrade::V(8), VGrade::V(12));
    cp.set_feature_preference(Feature::Highball, p);
    assert_eq!(cp.feature_preference(Feature::Highball), Some(&p));
    assert_eq!(cp.feature_preference(Feature::HeightDependent), None);
  }
}