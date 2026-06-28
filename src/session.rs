use crate::{ feature::Feature, preference::Preference };
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct SessionIntent {
  feature_overrides: HashMap<Feature, Preference>,
}

impl SessionIntent {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_feature_override(&mut self, feat: Feature, pref: Preference) {
    self.feature_overrides.insert(feat, pref);
  }

  pub fn effective_preference(&self, feat: Feature, defaults: &HashMap<Feature, Preference>) -> Option<Preference> {
    self.feature_overrides.get(&feat).or(defaults.get(&feat)).copied()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::profile::ClimberProfile;
  use crate::grade::VGrade;

  fn climber_profile() -> ClimberProfile {
    let mut cp = ClimberProfile::new(String::from("Jonny"), Some(71), Some(73), VGrade::V(8), VGrade::V(12));
    let pref = Preference::new(0.5, false).unwrap();
    cp.set_feature_preference(Feature::Highball, pref);
    cp.set_feature_preference(Feature::Technical, pref);
    cp
  }

  fn session_intent() -> SessionIntent {
    let mut si = SessionIntent::default();
    let updated_pref = Preference::new(0.0, true).unwrap();
    si.set_feature_override(Feature::Highball, updated_pref);
    si
  }

  #[test]
  fn override_set() {
    let cp = climber_profile();
    let si = session_intent();
    let updated_pref = Preference::new(0.0, true).unwrap();
    assert_eq!(si.effective_preference(Feature::Highball, cp.get_default_feature_preferences()), Some(updated_pref));    
  } 

  #[test]
  fn no_override_set() {
    let cp = climber_profile();
    let si = session_intent();
    let pref = Preference::new(0.5, false).unwrap();
    assert_eq!(si.effective_preference(Feature::Technical, cp.get_default_feature_preferences()), Some(pref));
  }

  #[test]
  fn unset_returns_none() {
    let cp = climber_profile();
    let si = session_intent();
    assert_eq!(si.effective_preference(Feature::HeightDependent, cp.get_default_feature_preferences()), None);
  }
}
