use crate::store::AreaId;

#[derive(Debug)]
pub struct Region {
  name: String,
  areas: Vec<AreaId>,
}

impl Region {
  pub fn new(name: String) -> Self {
    Self { name, areas: Vec::new() }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn areas(&self) -> &[AreaId] {
    &self.areas
  }

  pub fn add_area(&mut self, id: AreaId) {
    self.areas.push(id)
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{area::Area, store::AreaStore};

    #[test]
    fn region_holds_areas() {
        let mut areas = AreaStore::new();
        let a1 = areas.add(Area::new("Emerald Bay".to_string()));

        let mut region = Region::new("South Shore".to_string());
        region.add_area(a1);

        assert_eq!(region.name(), "South Shore");
        assert_eq!(region.areas()[0], a1);
    }
}