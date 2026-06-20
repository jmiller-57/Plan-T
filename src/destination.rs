use crate::store::RegionId;

#[derive(Debug)]
pub struct Destination {
  name: String,
  regions: Vec<RegionId>,
}

impl Destination {
  pub fn new(name: String) -> Self {
    Self { name, regions: Vec::new() }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn regions(&self) -> &[RegionId] {
    &self.regions
  }

  pub fn add_region(&mut self, id: RegionId) {
    self.regions.push(id)
  } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{region::Region, store::RegionStore};

    #[test]
    fn destination_holds_regions() {
        let mut regions = RegionStore::new();
        let r1 = regions.add(Region::new("South Shore".to_string()));

        let mut destination = Destination::new("Lake Tahoe".to_string());
        destination.add_region(r1);

        assert_eq!(destination.name(), "Lake Tahoe");
        assert_eq!(destination.regions()[0], r1);
    }
}