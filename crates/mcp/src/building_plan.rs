//! Pure geometric intent, deliberately separate from game edits or feasibility.

#[cfg(test)]
use policy::building_plan::cell;
use policy::building_plan::{Shape, cells};
use policy::mcp::Tool;
use serde_json::{Map, Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_plan",
        description: "Preview a bounded floor, wall, framed doorway, solid staircase or solid gable roof or union of up to 8 named primitives on a half-metre grid. Pure geometry only; no game reads, writes, material validation or crafting costs.",
        schema: policy::building_plan::preview_schema(),
    }
}

pub(crate) fn call(args: &Map<String, Value>) -> Result<Value, String> {
    if args.get("kind").and_then(Value::as_str) == Some("composite") {
        let (spec, material) = policy::building_plan::check_composite(args)?;
        let count = spec.elements.len();
        let cells: Vec<_> = spec.elements.iter().map(|(_, cell)| cell).collect();
        return Ok(json!({
            "status":"geometry_preview_only","executable":false,"kind":"composite",
            "material_reference":material,"material_validated":false,"canonical_plan":spec.canonical_plan,
            "grid":{"cell_size_m":0.5,"coordinates":"integer minimum corners; each cell occupies [n,n+1) in grid units","rotation_convention":"each part uses absolute origin-vertex quarter turns"},
            "footprint_m":{"min":spec.min.map(|n|f64::from(n)*0.5),"max_exclusive":spec.max_exclusive.map(|n|f64::from(n)*0.5)},
            "geometry":{"solid_cell_count":count,"candidate_cell_count":spec.candidates,"contributed_solid_cell_count":spec.contributed_solids,"omitted_cells":spec.candidates as usize-spec.contributed_solids,"overlap_cells":spec.contributed_solids-count,"nominal_solid_volume_m3":count as f64*0.125,"material_cell_counts":[{"material_reference":material,"cells":count}],"crafting_cost":null},
            "cell_minima":cells,
            "opening_behavior":"Primitive voids omit cells, never subtract other parts or erase occupants. Overlapping solid contributions are counted once; lexical part IDs own stable cells.",
            "unchecked":["source voxel and live material mapping","current world/session and occupants","rights and complete footprint persistence","collision, support, shelter and traversal","inventory, costs and unlocks","appearance, functional props, workshop usability and recovery"],
            "next_checks":["Survey the full footprint and resolve a source voxel on the current build.","Use the separate construction dry path and fresh actor/session/write gates before any mutation.","Independently verify full captures, rendered geometry, usability and guarded undo/reload."],
            "provenance":{"geometry_basis":"shared bounded construction compiler","verification":"offline geometry only; not live placement or write authorization"}
        }));
    }
    let spec = policy::building_plan::check(args)?;
    let cells = cells(&spec);
    let count = cells.len();
    let candidate = spec.size.iter().product::<i32>();
    let kind = match spec.shape {
        Shape::Floor => "floor",
        Shape::Wall => "wall",
        Shape::Doorway => "doorway",
        Shape::Staircase => "staircase",
        Shape::GableRoof => "gable_roof",
    };
    let mut result = json!({
        "status":"geometry_preview_only",
        "executable":false,
        "kind":kind,
        "material_reference":spec.material,
        "material_validated":false,
        "grid":{"cell_size_m":0.5,"coordinates":"integer minimum corners; each cell occupies [n,n+1) in grid units","origin_m":spec.origin.map(|n|f64::from(n)*0.5),"yaw_degrees":spec.yaw,"rotation_convention":"0: local x to +x, local z to +z; 90: local x to +z, local z to -x; about origin vertex, not game prop orientation"},
        "footprint_m":{"min":spec.min.map(|n|f64::from(n)*0.5),"max_exclusive":spec.max_exclusive.map(|n|f64::from(n)*0.5)},
        "local_dimensions_m":spec.size.map(|n|f64::from(n)*0.5),
        "geometry":{"solid_cell_count":count,"candidate_cell_count":candidate,"omitted_opening_cells":if matches!(spec.shape,Shape::Staircase|Shape::GableRoof) {0}else{candidate as usize-count},"nominal_solid_volume_m3":count as f64*0.125,"material_cell_counts":[{"material_reference":spec.material,"cells":count}],"crafting_cost":null},
        "cell_minima":cells,
        "opening_behavior":"Opening cells are omitted, not erased. Existing terrain or objects in an opening remain.",
        "unchecked":["material reference and voxel kind mapping","loaded world and current occupants","collision and traversal clearance","altar zones and permissions","inventory, crafting cost and unlocks","appearance and neighboring material transitions","persistence and recovery"],
        "next_checks":["Survey the complete footprint and opening in the target world.","Resolve the material on the current build and inspect placement rules.","Confirm clearances, base rights and costs using a disposable fixture.","Use a separate gated operation with current preconditions and read-back for any future write."],
        "provenance":{"geometry_basis":"enshwick v1.0.0-alpha.14 book/src/terrain.md, half-metre voxels","source":"https://github.com/enshwick/enshwick/blob/v1.0.0-alpha.14/book/src/terrain.md","verification":"offline arithmetic tests; not live placement"}
    });
    if spec.shape == Shape::Staircase {
        result["staircase"] = json!({"steps":spec.size[1],"rise_m":0.5,"tread_depth_m":f64::from(spec.size[2]/spec.size[1])*0.5,"ascends_local":"+z","walkability_verified":false});
        result["geometry"]["omitted_above_steps_cells"] = json!(candidate as usize - count);
        result["opening_behavior"] = json!(
            "Cells above treads are omitted, not erased. Existing occupants remain; headroom and climbability are unverified."
        );
    }
    if spec.shape == Shape::GableRoof {
        result["gable_roof"] = json!({"rise_m":0.5,"slope_run_m":f64::from(spec.size[0]/(2*spec.size[1]))*0.5,"ridge_local_axis":"z","attic_filled":true,"shelter_verified":false});
        result["geometry"]["omitted_outside_gable_cells"] = json!(candidate as usize - count);
        result["opening_behavior"] = json!(
            "Cells outside the solid gable are omitted, not erased. The gable volume is filled; this is not a hollow attic or verified shelter."
        );
    }
    Ok(result)
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
    #[test]
    fn staircase_preview_distinguishes_step_voids_from_doorway_openings() {
        let args = json!({"kind":"staircase","x":4,"y":4,"z":4,"width":1,"height":1.5,"depth":3,"material":"stone"});
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(result["geometry"]["solid_cell_count"], 24);
        assert_eq!(result["geometry"]["candidate_cell_count"], 36);
        assert_eq!(result["geometry"]["omitted_opening_cells"], 0);
        assert_eq!(result["geometry"]["omitted_above_steps_cells"], 12);
        assert_eq!(
            result["staircase"],
            json!({"steps":3,"rise_m":0.5,"tread_depth_m":1.0,"ascends_local":"+z","walkability_verified":false})
        );
        assert_eq!(result["executable"], false);
        assert_eq!(result["material_validated"], false);
    }
    #[test]
    fn gable_roof_preview_reports_solid_volume_and_unverified_shelter() {
        let args = json!({"kind":"gable_roof","x":4,"y":4,"z":4,"width":3,"height":1.5,"depth":1,"material":"stone"});
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(result["geometry"]["solid_cell_count"], 24);
        assert_eq!(result["geometry"]["candidate_cell_count"], 36);
        assert_eq!(result["geometry"]["omitted_opening_cells"], 0);
        assert_eq!(result["geometry"]["omitted_outside_gable_cells"], 12);
        assert_eq!(
            result["gable_roof"],
            json!({"rise_m":0.5,"slope_run_m":0.5,"ridge_local_axis":"z","attic_filled":true,"shelter_verified":false})
        );
        assert_eq!(result["executable"], false);
        assert_eq!(result["material_validated"], false);
    }
    #[test]
    fn compound_preview_matches_independent_union_without_live_context() {
        let args = json!({"kind":"composite","material":"stone","parts":[
            {"id":"floor","kind":"floor","x":4,"y":4,"z":4,"width":2,"height":0.5,"depth":2},
            {"id":"stairs","kind":"staircase","x":4,"y":4,"z":4,"width":1,"height":1,"depth":2},
            {"id":"roof","kind":"gable_roof","x":4,"y":5,"z":4,"width":2,"height":1,"depth":1}
        ]});
        let out = call(args.as_object().unwrap()).unwrap();
        let mut expected = std::collections::BTreeSet::new();
        for x in 8..12 {
            for z in 8..12 {
                expected.insert([x, 8, z]);
            }
        }
        for x in 8..10 {
            for z in 10..12 {
                expected.insert([x, 9, z]);
            }
        }
        for (x, height) in [(8, 1), (9, 2), (10, 2), (11, 1)] {
            for y in 10..10 + height {
                for z in 8..10 {
                    expected.insert([x, y, z]);
                }
            }
        }
        let actual: Vec<[i32; 3]> = serde_json::from_value(out["cell_minima"].clone()).unwrap();
        assert_eq!(
            actual
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        assert_eq!(actual.len(), 32);
        assert_eq!(out["geometry"]["candidate_cell_count"], 48);
        assert_eq!(out["geometry"]["contributed_solid_cell_count"], 40);
        assert_eq!(out["geometry"]["omitted_cells"], 8);
        assert_eq!(out["geometry"]["overlap_cells"], 8);
        assert_eq!(
            out["footprint_m"],
            json!({"min":[4.0,4.0,4.0],"max_exclusive":[6.0,6.0,6.0]})
        );
        assert_eq!(out["executable"], false);
        assert_eq!(out["material_validated"], false);
        let mut reordered = args.clone();
        reordered["parts"].as_array_mut().unwrap().reverse();
        assert_eq!(call(reordered.as_object().unwrap()).unwrap(), out);
        assert!(policy::args::check(&tool().schema, args.as_object().unwrap()).is_ok());
    }
}
