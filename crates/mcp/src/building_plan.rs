//! Pure geometric intent, deliberately separate from game edits or feasibility.

#[cfg(test)]
use policy::building_plan::cell;
use policy::building_plan::{Shape, cells};
use policy::mcp::Tool;
use serde_json::{Map, Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_plan",
        description: "Preview a bounded floor, wall or framed doorway on a half-metre grid. Pure geometry only; no game reads, writes, material validation or crafting costs.",
        schema: policy::building_plan::schema(),
    }
}

pub(crate) fn call(args: &Map<String, Value>) -> Result<Value, String> {
    let spec = policy::building_plan::check(args)?;
    let cells = cells(&spec);
    let count = cells.len();
    let candidate = spec.size.iter().product::<i32>();
    let kind = match spec.shape {
        Shape::Floor => "floor",
        Shape::Wall => "wall",
        Shape::Doorway => "doorway",
    };
    Ok(json!({
        "status":"geometry_preview_only",
        "executable":false,
        "kind":kind,
        "material_reference":spec.material,
        "material_validated":false,
        "grid":{"cell_size_m":0.5,"coordinates":"integer minimum corners; each cell occupies [n,n+1) in grid units","origin_m":spec.origin.map(|n|f64::from(n)*0.5),"yaw_degrees":spec.yaw,"rotation_convention":"0: local x to +x, local z to +z; 90: local x to +z, local z to -x; about origin vertex, not game prop orientation"},
        "footprint_m":{"min":spec.min.map(|n|f64::from(n)*0.5),"max_exclusive":spec.max_exclusive.map(|n|f64::from(n)*0.5)},
        "local_dimensions_m":spec.size.map(|n|f64::from(n)*0.5),
        "geometry":{"solid_cell_count":count,"candidate_cell_count":candidate,"omitted_opening_cells":candidate as usize-count,"nominal_solid_volume_m3":count as f64*0.125,"material_cell_counts":[{"material_reference":spec.material,"cells":count}],"crafting_cost":null},
        "cell_minima":cells,
        "opening_behavior":"Opening cells are omitted, not erased. Existing terrain or objects in an opening remain.",
        "unchecked":["material reference and voxel kind mapping","loaded world and current occupants","collision and traversal clearance","altar zones and permissions","inventory, crafting cost and unlocks","appearance and neighboring material transitions","persistence and recovery"],
        "next_checks":["Survey the complete footprint and opening in the target world.","Resolve the material on the current build and inspect placement rules.","Confirm clearances, base rights and costs using a disposable fixture.","Use a separate gated operation with current preconditions and read-back for any future write."],
        "provenance":{"geometry_basis":"enshwick v1.0.0-alpha.14 book/src/terrain.md, half-metre voxels","source":"https://github.com/enshwick/enshwick/blob/v1.0.0-alpha.14/book/src/terrain.md","verification":"offline arithmetic tests; not live placement"}
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Map<String, Value> {
        json!({"kind":"wall","x":100,"y":10,"z":100,"width":4,"height":3,"depth":0.5,"material":"stone"}).as_object().unwrap().clone()
    }
    #[test]
    fn rotations_preserve_cell_count_uniqueness_and_footprint() {
        for yaw in [0, 90, 180, 270] {
            let mut a = base();
            a.insert("yaw".into(), json!(yaw));
            let spec = policy::building_plan::check(&a).unwrap();
            let out = cells(&spec);
            assert_eq!(out.len(), 48);
            let set: std::collections::BTreeSet<_> = out.iter().copied().collect();
            assert_eq!(set.len(), out.len());
            for p in out {
                for (i, coordinate) in p.iter().enumerate() {
                    assert!(*coordinate >= spec.min[i] && *coordinate < spec.max_exclusive[i]);
                }
            }
        }
    }
    #[test]
    fn quarter_turn_rotates_cell_extent_not_just_point() {
        let mut a = base();
        a.insert("yaw".into(), json!(90));
        let spec = policy::building_plan::check(&a).unwrap();
        assert_eq!(cell(&spec, 0, 0, 0), [199, 20, 200]);
        assert_eq!(cell(&spec, 7, 5, 0), [199, 25, 207]);
    }
    #[test]
    fn doorway_omits_opening_but_keeps_sides_and_lintel() {
        let mut a = base();
        a.insert("kind".into(), json!("doorway"));
        for (key, n) in [
            ("door_offset", 1.0),
            ("door_width", 1.5),
            ("door_height", 2.0),
        ] {
            a.insert(key.into(), json!(n));
        }
        let spec = policy::building_plan::check(&a).unwrap();
        let out = cells(&spec);
        assert_eq!(out.len(), 36);
        for x in 2..5 {
            for y in 0..4 {
                assert!(!out.contains(&cell(&spec, x, y, 0)));
            }
        }
        assert!(out.contains(&cell(&spec, 2, 4, 0)));
        assert!(out.contains(&cell(&spec, 1, 0, 0)));
        assert!(out.contains(&cell(&spec, 5, 0, 0)));
        let result = call(&a).unwrap();
        assert_eq!(result["executable"], false);
        assert_eq!(result["geometry"]["omitted_opening_cells"], 12);
        assert!(result["geometry"]["crafting_cost"].is_null());
    }
    #[test]
    fn floor_volume_is_geometric_only() {
        let mut a = base();
        a.insert("kind".into(), json!("floor"));
        a.insert("height".into(), json!(0.5));
        a.insert("depth".into(), json!(3));
        let result = call(&a).unwrap();
        assert_eq!(result["geometry"]["solid_cell_count"], 48);
        assert_eq!(result["geometry"]["nominal_solid_volume_m3"], 6.0);
        assert_eq!(result["material_validated"], false);
    }
}
