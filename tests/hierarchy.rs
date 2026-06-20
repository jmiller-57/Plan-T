use plan_t::area::Area;
use plan_t::boulder::Boulder;
use plan_t::grade::VGrade;
use plan_t::problem::{Problem, ProblemStatus};
use plan_t::region::Region;
use plan_t::sector::Sector;
use plan_t::store::{
    AreaStore, 
    BoulderStore, 
    ProblemStore, 
    RegionStore, 
    SectorStore
};

#[test]
fn full_hierarchy_traverses() {
    let mut problems = ProblemStore::new();
    let mut boulders = BoulderStore::new();
    let mut sectors  = SectorStore::new();
    let mut areas    = AreaStore::new();
    let mut regions  = RegionStore::new();

    // problem -> boulder
    let pid = problems.add(Problem::new(
        Some("Midnight Lightning".to_string()), VGrade::new(8), ProblemStatus::Established,
    ));
    let mut egg = Boulder::new("The Egg".to_string());
    egg.add_problem(pid);
    let egg_id = boulders.add(egg);

    // nested sectors: parent -> child, boulder lives in the CHILD
    let mut child = Sector::new(Some("Lower Gauntlet".to_string()));
    child.add_boulder(egg_id);
    let child_id = sectors.add(child);

    let mut parent = Sector::new(Some("The Gauntlet".to_string()));
    parent.add_child(child_id);
    let parent_id = sectors.add(parent);

    // area -> sector, region -> area
    let mut area = Area::new("Emerald Bay".to_string());
    area.add_sector(parent_id);
    let area_id = areas.add(area);

    let mut region = Region::new("South Shore".to_string());
    region.add_area(area_id);
    let region_id = regions.add(region);

    // walk the WHOLE chain top-to-bottom: region -> areas -> sectors -> (recurse) -> boulders
    let region = regions.get(region_id).unwrap();
    let mut found = Vec::new();
    for &aid in region.areas() {
        let area = areas.get(aid).unwrap();
        for &sid in area.sectors() {
            found.extend(sectors.all_boulders(sid));
        }
    }

    assert_eq!(found, vec![egg_id]);   // nested several levels down → proves full descent

    // and it still resolves to its problem through the stores
    let boulder = boulders.get(found[0]).unwrap();
    let problem = problems.get(boulder.problems()[0]).unwrap();
    assert_eq!(problem.name(), Some("Midnight Lightning"));
}
