use serde::Deserialize;
use std::{
  error::Error,
  path::Path,
  collections::HashMap
};
use crate::{
  grade::VGrade,
  problem::{ProblemStatus, Problem},
  store::{Catalog, RegionId, AreaId, SectorId, BoulderId},
  boulder::Boulder,
  sector::Sector,
  area::Area,
  region::Region,
  destination::Destination,
};

#[derive(Debug, Deserialize)]
pub struct Row {
    pub region: String,
    pub area: String,
    pub sector_path: String,          // may be "" → handled later as a passthrough
    pub boulder: String,
    pub problem: Option<String>,      // blank cell → None (unnamed project)
    pub number: Option<String>,
    pub grade: Option<String>,
    pub stars: Option<String>,
    pub status: Option<String>,
    pub sun_notes: Option<String>,
    pub features: Option<String>,
    pub page: Option<String>,
}

pub fn read_rows(path: &Path) -> Result<Vec<Row>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut rows = Vec::new();
    for result in reader.deserialize() {
        let row: Row = result?;
        rows.push(row);
    }
    Ok(rows)
}

fn parse_grade(raw: Option<&str>) -> Result<Option<VGrade>, String> {
  let text = raw.unwrap_or("").trim();

  // Missing, blank, or and explicit "project" marker -> legitimately no grade
  if text.is_empty() {
    return Ok(None);
  }

  let lower = text.to_ascii_lowercase();
  if lower == "project" {
    return Ok(None);
  }

  if lower == "vb" {
    return Ok(Some(VGrade::VB));
  }

  // Otherwise, we expect "V<number>"
  let rest = match lower.strip_prefix('v') {
    Some(r) => r,
    None => return Err(format!("unrecognized grade: {text:?}")),
  };

  let n: u8 = match rest.parse() {
    Ok(n) => n,
    Err(_) => return Err(format!("unrecognized grade: {text:?}")),
  };

  match VGrade::new(n) {
    Some(vg) => Ok(Some(vg)),
    None => Err(format!("grade out of range (max V{}): {text:?}", VGrade::MAX)),
  }
}

pub fn build_catalog(rows: &[Row], destination: &str) -> Result<Catalog, String> {
    let mut catalog = Catalog::new();
    let dest_id = catalog.destinations.add(Destination::new(destination.to_string()));

    let mut region_ids: HashMap<String, RegionId> = HashMap::new();
    let mut area_ids:   HashMap<(RegionId, String), AreaId> = HashMap::new();
    let mut sector_ids: HashMap<(AreaId, String), SectorId> = HashMap::new();
    let mut passthrough_ids: HashMap<(AreaId, String), SectorId> = HashMap::new();
    let mut boulder_ids: HashMap<(SectorId, String), BoulderId> = HashMap::new();


    for row in rows {
        let region_id = *region_ids
            .entry(row.region.clone())
            .or_insert_with(|| {
                let id = catalog.regions.add(Region::new(row.region.clone()));
                catalog.destinations.get_mut(dest_id).unwrap().add_region(id);
                id
            });

        let area_id = *area_ids
            .entry((region_id, row.area.clone()))
            .or_insert_with(|| {
                let id = catalog.areas.add(Area::new(row.area.clone()));
                catalog.regions.get_mut(region_id).unwrap().add_area(id);
                id
            });
        
        // sectors: resolve the deepest sector this boulder lives in
        let segments: Vec<&str> = row.sector_path
            .split('>')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let sector_id = if segments.is_empty() {
          // lone boulder -> a passthrough sector (name None), one per boulder
          *passthrough_ids
            .entry((area_id, row.boulder.clone()))
            .or_insert_with(|| {
              let id = catalog.sectors.add(Sector::new(None));
              catalog.areas.get_mut(area_id).unwrap().add_sector(id);
              id
            })
        } else {
          // walk the path, get-or-creating each level and nested under the last
          let mut cumulative = String::new();
          let mut parent: Option<SectorId> = None; // None => parent is the Area
          for seg in &segments {
            if cumulative.is_empty() {
              cumulative.push_str(seg);
            } else {
              cumulative.push_str(" > ");
              cumulative.push_str(seg);
            }

            let seg_name = seg.to_string();
            let id = *sector_ids
                .entry((area_id, cumulative.clone()))
                .or_insert_with(|| {
                  let id = catalog.sectors.add(Sector::new(Some(seg_name)));
                  match parent {
                    Some(pid) => catalog.sectors.get_mut(pid).unwrap().add_child(id),
                    None => catalog.areas.get_mut(area_id).unwrap().add_sector(id),
                  }
                  id
                });
            parent = Some(id);
          }
          parent.unwrap()
        };

        // boulder: get-or-create under the deepest sector
        let boulder_id = *boulder_ids
            .entry((sector_id, row.boulder.clone()))
            .or_insert_with(|| {
              let id = catalog.boulders.add(Boulder::new(row.boulder.clone()));
              catalog.sectors.get_mut(sector_id).unwrap().add_boulder(id);
              id
            });
        
        // problem: one per row (no dedup), grade parsed (may fail)
        let grade = parse_grade(row.grade.as_deref())?;
        let status = parse_status(row.status.as_deref());
        let pid = catalog.problems.add(Problem::new(row.problem.clone(), grade, status));
        catalog.boulders.get_mut(boulder_id).unwrap().add_problem(pid);
    }
    Ok(catalog)
}

pub fn parse_status(raw: Option<&str>) -> ProblemStatus {
  match raw.unwrap_or("").trim().to_ascii_lowercase().as_str() {
    "project" | "open" | "openproject" => ProblemStatus::OpenProject,
    _ => ProblemStatus::Established,
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_kingsbury_rows() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/data/KingsburyGrade.csv"));
        let rows = read_rows(path).expect("KingsburyGrade.csv should parse");
        assert!(!rows.is_empty());
    }

    #[test]
    fn parses_valid_grades() {
        assert_eq!(parse_grade(Some("V4")), Ok(Some(VGrade::V(4))));
        assert_eq!(parse_grade(Some("v4")), Ok(Some(VGrade::V(4))));
        assert_eq!(parse_grade(Some("VB")), Ok(Some(VGrade::VB)));
    }

    #[test]
    fn absent_grade_is_ok_none() {
        assert_eq!(parse_grade(Some("project")), Ok(None));
        assert_eq!(parse_grade(Some("")), Ok(None));
        assert_eq!(parse_grade(None), Ok(None));
    }

    #[test]
    fn malformed_grade_is_err() {
        assert!(parse_grade(Some("banana")).is_err());
        assert!(parse_grade(Some("V99")).is_err());
    }

    #[test]
    fn build_dedups_hierarchy() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/data/KingsburyGrade.csv"));
        let rows = read_rows(path).unwrap();
        let catalog = build_catalog(&rows, "Lake Tahoe").unwrap();

        assert_eq!(catalog.destinations.len(), 1);
        assert_eq!(catalog.regions.len(), 1);
        assert_eq!(catalog.areas.len(), 1);
        assert!(catalog.sectors.len() >= 1);
        assert!(catalog.boulders.len() >= 1);
        assert_eq!(catalog.problems.len(), rows.len());
    }
}