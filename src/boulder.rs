use crate::{
  store::ProblemId, 
  location::{
    Coordinates, Aspect
  },
};

#[derive(Debug)]
pub struct Boulder {
  name: String,
  problems: Vec<ProblemId>,
  coordinates: Option<Coordinates>,
  aspect: Option<Aspect>,
}

impl Boulder {
  pub fn new(name: String) -> Self {
    Self { name, problems: Vec::new(), coordinates: None, aspect: None }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn problems(&self) -> &[ProblemId] {
    &self.problems
  }

  pub fn add_problem(&mut self, id: ProblemId) {
    self.problems.push(id);
  }

  pub fn set_aspect(&mut self, aspect: Aspect) {
    self.aspect = Some(aspect);
  }

  pub fn aspect(&self) -> Option<Aspect> {
    self.aspect
  }

  pub fn set_coordinates(&mut self, coords: Coordinates) {
    self.coordinates = Some(coords);
  }

  pub fn coordinates(&self) -> Option<Coordinates> {
    self.coordinates
  }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::grade::VGrade;
    use crate::problem::{Problem, ProblemStatus};
    use crate::store::ProblemStore;

    #[test]
    fn boulder_references_problems_in_store() {
        let mut problems = ProblemStore::new();
        let p1 = problems.add(Problem::new(Some("Slab".to_string()),  VGrade::new(2), ProblemStatus::Established));
        let p2 = problems.add(Problem::new(Some("Arete".to_string()), VGrade::new(5), ProblemStatus::Established));

        let mut boulder = Boulder::new("The Egg".to_string());
        boulder.add_problem(p1);
        boulder.add_problem(p2);

        for id in &boulder.problems {              // iterating &Vec yields &ProblemId
            assert!(problems.get(*id).is_some());  // *id derefs the &ProblemId back to a (Copy) ProblemId
        }
    }

    #[test]
    fn boulder_valid_location() {
      let mut problems = ProblemStore::new();
      let p1 = problems.add(Problem::new(Some("Slab".to_string()),  VGrade::new(2), ProblemStatus::Established));
      let p2 = problems.add(Problem::new(Some("Arete".to_string()), VGrade::new(5), ProblemStatus::Established));

      let mut boulder = Boulder::new("The Egg".to_string());
      boulder.add_problem(p1);
      boulder.add_problem(p2);

      assert_eq!(boulder.coordinates, None);
      assert_eq!(boulder.aspect, None);

      let a = Aspect::new(0).unwrap();
      let c = Coordinates::new(-90.0, 90.0).unwrap();

      boulder.set_aspect(a);
      boulder.set_coordinates(c);

      assert_eq!(boulder.coordinates(), Some(c));
      assert_eq!(boulder.aspect(), Some(a));
    }
}