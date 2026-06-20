use crate::store::{BoulderId, SectorId};

#[derive(Debug)]
pub struct Sector {
  name: Option<String>,
  children: Vec<SectorId>,
  boulders: Vec<BoulderId>,
}

impl Sector {
  pub fn new(name: Option<String>) -> Self {
    Self { name, children: Vec::new(), boulders: Vec::new() }
  }

  pub fn name(&self) -> Option<&str> {
    self.name.as_deref()
  }

  pub fn children(&self) -> &[SectorId] {
    &self.children
  }

  pub fn boulders(&self) -> &[BoulderId] {
    &self.boulders
  }

  pub fn add_child(&mut self, id: SectorId) {
    self.children.push(id)
  }

  pub fn add_boulder(&mut self, id: BoulderId) {
    self.boulders.push(id)
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{boulder::Boulder, store::{SectorStore, BoulderStore}};

    #[test]
    fn sector_nests_and_holds_boulders() {
        let mut sectors  = SectorStore::new();
        let mut boulders = BoulderStore::new();

        let child_id   = sectors.add(Sector::new(Some("Foxey Lady".to_string())));                 // a passthrough child
        let boulder_id = boulders.add(Boulder::new("Egg".to_string()));

        let mut parent = Sector::new(Some("Kingsbury".to_string()));
        parent.add_child(child_id);
        parent.add_boulder(boulder_id);

        assert_eq!(parent.name(), Some("Kingsbury"));
        assert_eq!(parent.children()[0], child_id);
        assert_eq!(parent.boulders()[0], boulder_id);
    }

    #[test]
    fn passthrough_sector_has_no_name() {
        assert_eq!(Sector::new(None).name(), None);
    }
}