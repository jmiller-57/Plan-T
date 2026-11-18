use crate::{
  problem::Problem, 
  boulder::Boulder, 
  sector::Sector, 
  area::Area,
  region::Region,
  destination::Destination,
};
use std::marker::PhantomData;

/// A type-safe handle into an `Arena<T>`. `Id<Boulder>` and `Id<Problem>` are
/// different types, so the compiler won't let you index the wrong arena.
pub struct Id<T> {
  index: usize,
  _marker: PhantomData<T>,
}

impl<T> Id<T> {
  fn new(index: usize) -> Self {
    Self { index, _marker: PhantomData }
  }
}

impl<T> Clone for Id<T> {
  fn clone(&self) -> Self { *self }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
  fn eq(&self, other: &Self) -> bool { self.index == other.index }
}
impl<T> Eq for Id<T> {}
impl<T> std::hash::Hash for Id<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.index.hash(state);
  }
}
impl<T> std::fmt::Debug for Id<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Id({})", self.index)
  }
}

#[derive(Debug)]
pub struct Arena<T> {
  items: Vec<T>,
}

impl<T> Default for Arena<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Arena<T> {
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub fn add(&mut self, item: T) -> Id<T> {
    let index = self.items.len();
    self.items.push(item);
    Id::new(index)
  }

  pub fn get(&self, id: Id<T>) -> Option<&T> {
    self.items.get(id.index)
  }

  pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
    self.items.get_mut(id.index)
  }

  pub fn len(&self) -> usize {
    self.items.len()
  }
}

impl Arena<Sector> {
  pub fn all_boulders(&self, root: SectorId) -> Vec<BoulderId> {
    let mut out = Vec::new();
    self.collect_boulders(root, &mut out);
    out
  }

  pub fn collect_boulders(&self, id: SectorId, out: &mut Vec<BoulderId>) {
    if let Some(sector) = self.get(id) {
      out.extend(sector.boulders());
      for &child in sector.children() {
        self.collect_boulders(child, out);
      }
    }
  }
}

pub type DestinationId = Id<Destination>;
pub type RegionId      = Id<Region>;
pub type AreaId        = Id<Area>;
pub type SectorId      = Id<Sector>;
pub type BoulderId     = Id<Boulder>;
pub type ProblemId     = Id<Problem>;

pub type DestinationStore = Arena<Destination>;
pub type RegionStore      = Arena<Region>;
pub type AreaStore        = Arena<Area>;
pub type SectorStore      = Arena<Sector>;
pub type BoulderStore     = Arena<Boulder>;
pub type ProblemStore     = Arena<Problem>;

#[derive(Debug, Default)]
pub struct Catalog {
    pub destinations: DestinationStore,
    pub regions:  RegionStore,
    pub areas:    AreaStore,
    pub sectors:  SectorStore,
    pub boulders: BoulderStore,
    pub problems: ProblemStore,
}

impl Catalog {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn problems_under_area(&self, area_id: AreaId) -> Vec<ProblemId> {
    let mut out: Vec<ProblemId> = Vec::new();

    if let Some(area) = self.areas.get(area_id) {                               // Hop 1: AreaId -> Area
      for &sector_id in area.sectors() {
        for boulder_id in self.sectors.all_boulders(sector_id) {    // Hop 2: recurse over Sub-Sectors
          if let Some(boulder) = self.boulders.get(boulder_id) {            // Hop 3: BoulderId -> Boulder
            out.extend(boulder.problems());                                     // Collect ProblemIds
          }
        }
      }
    }

    out
  }
}

#[cfg(test)]
mod tests {
    use super::*;                                  // brings in Problem, Boulder, etc.
    use crate::grade::VGrade;
    use crate::problem::ProblemStatus;

    #[test]
    fn problem_round_trips() {
        let mut store = ProblemStore::new();
        let id = store.add(Problem::new(Some("Little Wing".to_string()), VGrade::new(10), ProblemStatus::Established));
        assert_eq!(store.get(id).and_then(|p| p.name()), Some("Little Wing"));
    }

    #[test]
    fn boulder_round_trips() {
        let mut store = BoulderStore::new();
        let id = store.add(Boulder::new("Foxy Lady".to_string()));
        assert_eq!(store.get(id).map(|b| b.name()), Some("Foxy Lady"));
    }

    #[test]
    fn missing_id_returns_none() {
        let store = ProblemStore::new();           // empty
        assert!(store.get(ProblemId::new(99)).is_none());
    }
}