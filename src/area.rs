use crate::store::SectorId;

#[derive(Debug)]
pub struct Area {
  name: String,
  sectors: Vec<SectorId>,
}

impl Area {
  pub fn new(name: String) -> Self {
    Self { name, sectors: Vec::new() }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn sectors(&self) -> &[SectorId] {
    &self.sectors
  }

  pub fn add_sector(&mut self, id: SectorId) {
    self.sectors.push(id);
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sector::Sector, store::SectorStore};

    #[test]
    fn area_holds_sectors() {
        let mut sectors = SectorStore::new();
        let s1 = sectors.add(Sector::new(Some("Kings".to_string())));

        let mut area = Area::new("Emerald Bay".to_string());
        area.add_sector(s1);

        assert_eq!(area.name(), "Emerald Bay");
        assert_eq!(area.sectors()[0], s1);          // SectorId == SectorId — value object, legit
    }
}