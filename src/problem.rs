use crate::{ grade::VGrade, location::Aspect, feature::Feature };
use std::collections::HashSet;

#[derive(Debug)]
pub struct Problem {
  name: Option<String>,
  grade: Option<VGrade>,
  status: ProblemStatus,
  features: HashSet<Feature>,
  aspect: Option<Aspect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProblemStatus {
  // order matters: lifecycle OpenProject < Established
  OpenProject,
  Established,
}

impl Problem {
  pub fn new(name: Option<String>, grade: Option<VGrade>, status: ProblemStatus) -> Self {
    Problem { name, grade, status, features: HashSet::new(), aspect: None }
  }

  pub fn name(&self) -> Option<&str> {
    self.name.as_deref()
  }

  pub fn grade(&self) -> Option<VGrade> {
    self.grade
  }

  pub fn status(&self) -> ProblemStatus {
    self.status
  }

  pub fn features(&self) -> &HashSet<Feature> {
    &self.features
  }

  pub fn set_aspect(&mut self, aspect: Aspect) {
    self.aspect = Some(aspect)
  }

  pub fn aspect(&self) -> Option<Aspect> {
    self.aspect
  }

  pub fn effective_aspect(&self, boulder_aspect: Option<Aspect>) -> Option<Aspect> {
    self.aspect.or(boulder_aspect)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::boulder::Boulder;

  #[test]
  fn grade_check() {
    let p = Problem {
      name: Some(String::from("Midnight Lightning")),
      grade: VGrade::new(8),
      status: ProblemStatus::Established,
      features: HashSet::from([Feature::Highball, Feature::Technical]),
      aspect: None,
    };

    assert_eq!(p.grade, Some(VGrade::V(8)));
    assert_eq!(p.features.len(), 2);
    assert!(p.features.contains(&Feature::Highball));
  }

  #[test]
  fn aspect_check() {
    let mut p = Problem {
      name: Some(String::from("Midnight Lightning")),
      grade: VGrade::new(8),
      status: ProblemStatus::Established,
      features: HashSet::from([Feature::Highball, Feature::Technical]),
      aspect: None,
    };

    assert_eq!(p.aspect(), None);

    let mut boulder = Boulder::new("The Egg".to_string());

    assert_eq!(p.effective_aspect(boulder.aspect()), None);

    let ba = Aspect::new(0).unwrap();
    boulder.set_aspect(ba);
    
    assert_eq!(p.effective_aspect(boulder.aspect()), boulder.aspect());

    let pa = Aspect::new(90).unwrap();
    p.set_aspect(pa);

    assert_eq!(p.effective_aspect(boulder.aspect()), p.aspect());
    
  }
}