use std::collections::{HashMap, HashSet};
use crate::{
  feature::Feature, grade::*, preference::Preference, problem::Problem, profile::ClimberProfile, session::{GradeDesire, SessionIntent, SessionMode}
};

pub fn problem_score(problem: &Problem, profile: &ClimberProfile, session: &SessionIntent) -> Option<f64> {
  let grade = problem.grade()?;
  let range = effective_range(session.grade_desire(), profile);
  let g = grade_fit(grade, range);
  let mult = feature_fit(problem.features(), session, profile.default_feature_preferences())?;

  Some(g * mult)
}

pub fn grade_fit(grade: VGrade, range: GradeRange) -> f64 {
  if in_range(grade, range) {
    1.0
  } else {
    calculate_falloff(grade, range)
  }
}

pub fn feature_fit(
    features: &HashSet<Feature>
    ,session: &SessionIntent
    ,defaults: &HashMap<Feature, Preference>
) -> Option<f64> {
  const K: f64 = 0.5;  // feature sensitivity, 0.5 == symmetric
  let mut count = 0;
  let mut weight_sum = 0.0;
  for feature in features {
    if let Some(p) = session.effective_preference(*feature, defaults) {
      if p.veto() {
        return None
      }
      weight_sum += p.weight();
      count += 1;
    }
  }
  if count == 0 {
    Some(1.0)
  } else {
    let w = weight_sum / f64::from(count);
    Some(1.0 + w * K)
  }
}

fn in_range(grade: VGrade, range: GradeRange) -> bool {
  grade <= range.max() && grade >= range.min()
}

fn above(grade: VGrade, range: GradeRange) -> bool {
  grade > range.max()
}

fn calculate_falloff(grade: VGrade, range: GradeRange) -> f64 {
  const HARD_FALLOFF: f64 = 3.0;
  const EASY_FALLOFF: f64 = 4.0;
  let d: u8;
  let k: f64;
  if above(grade, range) { // fell outside range on harder end
    d = grade.distance(range.max());
    k = HARD_FALLOFF;
  } else {  // fell outside range on easier end
    d = grade.distance(range.min());
    k = EASY_FALLOFF;
  }
  (1.0 - (f64::from(d) / k)).max(0.0)
}

pub fn effective_range(grade_desire: GradeDesire, profile: &ClimberProfile) -> GradeRange {
  let flash = profile.flash_grade().index();
  let max = profile.max_grade().index();
  match grade_desire {
    GradeDesire::Explicit(range) => range,
    GradeDesire::FromMode(mode) => match mode {  
      SessionMode::Cruise => {
        GradeRange::new(
          VGrade::VB, 
          VGrade::from_index(flash - 2)
        ).expect("min <= max by construction")
      },
      SessionMode::SendVolume => {
        GradeRange::new(
          VGrade::from_index(flash - 2),
          VGrade::from_index(flash + 1),
        ).expect("min <= max by construction")
      },
      SessionMode::Project => {
        GradeRange::new(
          VGrade::from_index(max - 2),
          VGrade::from_index(max + 1),
        ).expect("min <= max by construction")
      },
      SessionMode::Flexible => {
        GradeRange::new(
          VGrade::VB,
          VGrade::from_index(max),
        ).expect("min <= max by construction")
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{problem::ProblemStatus, session::{GradeDesire::{Explicit, FromMode}, SessionMode::*}};
  use super::*;

  #[test]
  fn within_range() {
    let v1 = VGrade::new(1).unwrap();
    let v3 = VGrade::new(3).unwrap();
    let v6 = VGrade::new(6).unwrap();
    let v5 = VGrade::new(5).unwrap();
    let v15 = VGrade::new(15).unwrap();

    let range = GradeRange::new(v3, v6).unwrap();
  
    assert_eq!(in_range(v3, range), true);
    assert_eq!(in_range(v6, range), true);
    assert_eq!(in_range(v5, range), true);
    assert_eq!(in_range(v15, range), false);
    assert_eq!(above(v15, range), true);
    assert_eq!(above(v1, range), false);
  }

  #[test]
  fn expected_grade_score() {
    let v1 = VGrade::new(1).unwrap();
    let v3 = VGrade::new(3).unwrap();
    
    let v5 = VGrade::new(5).unwrap();
    let v6 = VGrade::new(6).unwrap();
    let v8 = VGrade::new(8).unwrap();
    let v15 = VGrade::new(15).unwrap();

    let range = GradeRange::new(v3, v6).unwrap();

    assert_eq!(grade_fit(v3, range), 1.0);
    assert_eq!(grade_fit(v5, range), 1.0);
    assert_eq!(grade_fit(v6, range), 1.0);
    assert_eq!(grade_fit(v15, range), 0.0);
    assert!(grade_fit(v1, range) < 0.75);
    assert!(grade_fit(v1, range) > grade_fit(v8, range));
  }

  #[test]
  fn expected_feature_score() {
    let loved = Preference::new(1.0, false).unwrap();
    let hated = Preference::new(-1.0, false).unwrap();
    let vetoed = Preference::new(0.0, true).unwrap();

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    cp.set_feature_preference(Feature::Technical, loved);
    cp.set_feature_preference(Feature::Highball, hated);

    let mut features = HashSet::from([Feature::Highball]);
    let mut session = SessionIntent::new();

    // a hated feature
    assert!(feature_fit(&features, &session, cp.default_feature_preferences()).unwrap() < 1.0);

    // a vetoed the feature
    features = HashSet::from([Feature::HeightDependent]);
    session.set_feature_override(Feature::HeightDependent, vetoed);
    assert_eq!(feature_fit(&features, &session, cp.default_feature_preferences()), None);

    // a loved feature
    features = HashSet::from([Feature::Technical]);
    assert!(feature_fit(&features, &session, cp.default_feature_preferences()).unwrap() > 1.0);

    // a neutral feature
    features = HashSet::from([Feature::BadLanding]);
    assert_eq!(feature_fit(&features, &session, cp.default_feature_preferences()), Some(1.0));

    // a loved and hated set of features
    features = HashSet::from([Feature::Technical, Feature::Highball]);
    assert!((feature_fit(&features, &session, cp.default_feature_preferences()).unwrap() - 1.0).abs() < 0.1);

  }

  #[test]
  fn validate_effective_range() {
    let cruise = FromMode(Cruise);
    let flex = FromMode(Flexible);
    let project = FromMode(Project);
    let send_volume = FromMode(SendVolume);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());

    let cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    assert_eq!(effective_range(cruise, &cp), GradeRange::new(VGrade::VB, VGrade::V(6)).unwrap());
    assert_eq!(effective_range(flex, &cp), GradeRange::new(VGrade::VB, VGrade::V(12)).unwrap());
    assert_eq!(effective_range(project, &cp), GradeRange::new(VGrade::V(10), VGrade::V(13)).unwrap());
    assert_eq!(effective_range(send_volume, &cp), GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());

    let beginner = ClimberProfile::new(
      String::from("Beginner")
      , Some(71), Some(73)
      , VGrade::VB
      , VGrade::V(1)
    );

    assert_eq!(effective_range(cruise, &beginner), GradeRange::new(VGrade::VB, VGrade::VB).unwrap());

    let pro = ClimberProfile::new(
      String::from("Pro")
      , Some(71), Some(73)
      , VGrade::V(15)
      , VGrade::V(18)
    );

    assert_eq!(effective_range(project, &pro), GradeRange::new(VGrade::V(16), VGrade::V(18)).unwrap());
    assert_eq!(effective_range(explicit, &pro), GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
  }

  #[test]
  fn validate_problem_score_in_grade_liked() {
    let mut in_grade_liked = Problem::new(Some(String::from("In Liked")), Some(VGrade::V(8)), ProblemStatus::Established);
    in_grade_liked.add_feature(Feature::HeightDependent);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );
    cp.set_feature_preference(Feature::HeightDependent, Preference::new(1.0, false).unwrap());

    assert!(problem_score(&in_grade_liked, &cp, &session).unwrap() > 1.0);
  }

  #[test]
  fn validate_problem_score_in_grade_disliked() {
    let mut in_grade_disliked = Problem::new(Some(String::from("In Disliked")), Some(VGrade::V(8)), ProblemStatus::Established);
    in_grade_disliked.add_feature(Feature::Highball);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );
    cp.set_feature_preference(Feature::Highball, Preference::new(-1.0, false).unwrap());

    assert!(problem_score(&in_grade_disliked, &cp, &session).unwrap() < 1.0);
  }

  #[test]
  fn validate_problem_score_unkown_grade() {
    let uknown_grade = Problem::new(Some(String::from("Unknown Grade")), None, ProblemStatus::Established);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    assert_eq!(problem_score(&uknown_grade, &cp, &session), None);
  }

  #[test]
  fn validate_problem_score_vetoed_feature() {
    let mut vetoed = Problem::new(Some(String::from("Vetoed")), Some(VGrade::V(8)), ProblemStatus::Established);
    vetoed.add_feature(Feature::BadLanding);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    cp.set_feature_preference(Feature::BadLanding, Preference::new(0.0, true).unwrap());

    assert_eq!(problem_score(&vetoed, &cp, &session), None);
  }

  #[test]
  fn validate_problem_score_way_off_grade_liked() {
    let mut off_grade_liked = Problem::new(Some(String::from("Off Liked")), Some(VGrade::V(16)), ProblemStatus::Established);
    off_grade_liked.add_feature(Feature::HeightDependent);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    cp.set_feature_preference(Feature::HeightDependent, Preference::new(1.0, false).unwrap());

    assert!(problem_score(&off_grade_liked, &cp, &session).unwrap() < 1.0);
  }

  #[test]
  fn validate_problem_score_near_off_grade_liked() {
    let mut off_grade_liked = Problem::new(Some(String::from("Off Liked")), Some(VGrade::V(11)), ProblemStatus::Established);
    off_grade_liked.add_feature(Feature::HeightDependent);

    let explicit = Explicit(GradeRange::new(VGrade::V(6), VGrade::V(9)).unwrap());
    let mut session = SessionIntent::new();
    session.set_grade_desire(explicit);

    let mut cp = ClimberProfile::new(
      String::from("Jonny")
      , Some(71), Some(73)
      , VGrade::V(8)
      , VGrade::V(12)
    );

    cp.set_feature_preference(Feature::HeightDependent, Preference::new(1.0, false).unwrap());

    assert!(problem_score(&off_grade_liked, &cp, &session).unwrap() < 1.0);
  }
}