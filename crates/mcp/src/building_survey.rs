//! Bounded voxel observations, called only after scheduling onto the game thread.

use enshwick::voxels::{VoxelBox, VoxelMaterial, VoxelPos};
use enshwick::world::BuildRefusal;
use policy::building_survey::Spec;
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_survey",
        description: "Read at most 4096 half-metre cells in a box at most 8 m per side, with whole-box persistence and optional connected-player build rights. Optional foundation previews one bounded Air-only pier before occupied terrain; no rendered surface-height claim. May trigger chunk loading. No writes, prop/collision checks, crafting costs or complete altar boundary discovery.",
        schema: policy::building_survey::schema(),
    }
}

fn encode(block: &VoxelBox) -> Value {
    let mut counts = [0usize; 4];
    let cells: Vec<_> = block.iter().map(|(position, voxel)| {
        let (kind, id, count) = match voxel.material() {
            VoxelMaterial::Air => ("air", None, 0),
            VoxelMaterial::Terrain(id) => ("terrain", Some(id), 1),
            VoxelMaterial::Building(id) => ("building", Some(id), 2),
            VoxelMaterial::Unknown(id) => ("unknown", Some(id), 3),
            _ => ("unknown", None, 3),
        };
        counts[count] += 1;
        json!({"min_cells":position.to_array(),"material":{"kind":kind,"id":id},"fill":voxel.fill()})
    }).collect();
    json!({
        "cell_count":cells.len(),
        "material_counts":{"air":counts[0],"terrain":counts[1],"building":counts[2],"unknown":counts[3]},
        "cells":cells
    })
}

fn foundation(block: &VoxelBox, request: &policy::building_survey::Foundation) -> Value {
    use policy::building_survey::{ColumnCell, foundation_span};
    let [x, upper, z] = request.upper;
    let mut column: Vec<_> = block
        .iter()
        .filter_map(|(position, voxel)| {
            let p = position.to_array();
            (p[0] == x && p[2] == z && p[1] < upper).then(|| {
                let cell = match voxel.material() {
                    VoxelMaterial::Air => ColumnCell::Air,
                    VoxelMaterial::Terrain(_) => ColumnCell::Terrain(voxel.fill()),
                    _ => ColumnCell::Obstruction,
                };
                (p[1], cell)
            })
        })
        .collect();
    column.sort_unstable_by_key(|(y, _)| std::cmp::Reverse(*y));
    let cells: Vec<_> = column.iter().map(|(_, cell)| *cell).collect();
    match foundation_span(&cells, request.max_cells) {
        Err(reason) => json!({"state":"refused","plan":null,"reason":reason}),
        Ok(count) => {
            let lower = upper - count as u32;
            json!({"state":"preview_only","upper_boundary_cells":request.upper,
                "terrain_stop_cell":[x,lower-1,z],"air_cell_count":count,
                "plan":{"kind":"wall","x":f64::from(x)*0.5,"y":f64::from(lower)*0.5,"z":f64::from(z)*0.5,"width":0.5,"height":count as f64*0.5,"depth":0.5},
                "terrain_overwrite":false,"rendered_contact_verified":false,
                "limitations":["occupied terrain is a cell stop, not the rendered surface height","no collision, structural support or traversal proof","survey and proposed plan are not authorization; placement requires fresh gated validation"]})
        }
    }
}

fn rights(game: crate::tools::Game, spec: &Spec, min: [f64; 3], max: [f64; 3]) -> Value {
    let Some(selector) = &spec.player else {
        return json!({"state":"not_requested","allowed":null});
    };
    let players: Vec<_> = game
        .world
        .players()
        .into_iter()
        .filter(|p| p.key.0.to_string() == *selector || p.is_named(selector))
        .collect();
    let player = match players.as_slice() {
        [] => return json!({"state":"player_not_found","allowed":null}),
        [player] => player,
        _ => return json!({"state":"ambiguous_player","allowed":null}),
    };
    match game.world.can_build(player, min, max) {
        Ok(()) => json!({"state":"allowed","allowed":true}),
        Err(BuildRefusal::Unchecked(error)) => {
            json!({"state":"unavailable","allowed":null,"error":error.to_string()})
        }
        Err(reason) => json!({"state":"refused","allowed":false,"reason":reason.to_string()}),
    }
}

/// The caller must queue this read with `Scheduler::next_tick`.
pub(crate) fn read(game: crate::tools::Game, spec: Spec) -> Result<Value, String> {
    if !enshwick::host().is_some_and(|host| host.hosting()) {
        return Err("building survey requires a hosted world".into());
    }
    let block = game
        .voxels
        .read(VoxelPos::from(spec.min), spec.size)
        .map_err(|error| format!("voxel survey failed: {error}"))?;
    let min = spec.min.map(|n| f64::from(n) * 0.5);
    let max = spec.max_exclusive.map(|n| f64::from(n) * 0.5);
    let persistence = match game.world.persists(min, max) {
        Ok(value) => json!({"state":"observed","whole_box_persists":value}),
        Err(error) => {
            json!({"state":"unavailable","whole_box_persists":null,"error":error.to_string()})
        }
    };
    Ok(json!({
        "status":"read_only_observation",
        "region":{"min_cells":spec.min,"size_cells":spec.size,"max_exclusive_cells":spec.max_exclusive,"min_m":min,"max_exclusive_m":max},
        "grid":{"cell_size_m":0.5,"bounds":"half-open complete cells","cell_order":"x fastest, then y, then z","index_formula":"x + size_x * (y + size_y * z), using local offsets"},
        "voxels":encode(&block),
        "foundation":spec.foundation.as_ref().map(|request|foundation(&block,request)),
        "persistence":persistence,
        "build_rights":rights(game, &spec, min, max),
        "material_ids":"runtime voxel IDs; not inventory item IDs or a validated building palette",
        "live_semantics_verified":false,
        "unchecked":["props, collision and traversal clearance","material/item mapping and shape extents","crafting costs, stock and unlocks","complete altar boundaries","save/reload and recovery"],
        "limitations":["persistence is the published whole-box zone check, not a save/reload proof","build rights are an observation, not authorization for a later write","a voxel read can load chunks; an error is not empty geometry"]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::voxels::Voxel;

    #[test]
    fn foundation_preview_preserves_stop_and_emits_existing_primitive() {
        let request = policy::building_survey::Foundation {
            upper: [10, 24, 30],
            max_cells: 2,
        };
        let block = VoxelBox::new(
            VoxelPos::new(10, 20, 30),
            [1, 4, 1],
            vec![
                Voxel::new(VoxelMaterial::Building(157), 7),
                Voxel::new(VoxelMaterial::Terrain(4), 67),
                Voxel::new(VoxelMaterial::Air, 255),
                Voxel::new(VoxelMaterial::Air, 0),
            ],
        )
        .unwrap();
        let out = foundation(&block, &request);
        assert_eq!(out["terrain_stop_cell"], json!([10, 21, 30]));
        assert_eq!(out["air_cell_count"], 2);
        assert_eq!(out["rendered_contact_verified"], false);
        let args = json!({"operation":"foundation_fixture","source_x":0,"source_y":0,"source_z":0,"player":"fixture","plan":out["plan"]});
        let compiled = policy::construction::check_plan(args.as_object().unwrap()).unwrap();
        assert_eq!(compiled.targets, vec![[10, 22, 30], [10, 23, 30]]);
        let mut plan = out["plan"].as_object().unwrap().clone();
        plan.insert("material".into(), json!("unvalidated"));
        let spec = policy::building_plan::check(&plan).unwrap();
        assert_eq!(
            policy::building_plan::cells(&spec),
            vec![[10, 22, 30], [10, 23, 30]]
        );
    }

    #[test]
    fn foundation_obstruction_and_missing_stop_never_return_a_plan() {
        let request = policy::building_survey::Foundation {
            upper: [10, 22, 30],
            max_cells: 1,
        };
        for bottom in [
            VoxelMaterial::Building(157),
            VoxelMaterial::Unknown(254),
            VoxelMaterial::Air,
        ] {
            let block = VoxelBox::new(
                VoxelPos::new(10, 20, 30),
                [1, 2, 1],
                vec![Voxel::new(bottom, 7), Voxel::new(VoxelMaterial::Air, 0)],
            )
            .unwrap();
            let out = foundation(&block, &request);
            assert_eq!(out["state"], "refused");
            assert!(out["plan"].is_null());
        }
    }

    #[test]
    fn exact_cells_preserve_x_y_z_order_material_ids_and_fill() {
        let block = VoxelBox::new(
            VoxelPos::new(10, 20, 30),
            [2, 2, 2],
            vec![
                Voxel::new(VoxelMaterial::Air, 255),
                Voxel::new(VoxelMaterial::Terrain(7), 128),
                Voxel::new(VoxelMaterial::Building(157), 0),
                Voxel::new(VoxelMaterial::Unknown(254), 9),
                Voxel::new(VoxelMaterial::Air, 0),
                Voxel::new(VoxelMaterial::Building(128), 0),
                Voxel::new(VoxelMaterial::Terrain(127), 1),
                Voxel::new(VoxelMaterial::Air, 0),
            ],
        )
        .unwrap();
        let out = encode(&block);
        assert_eq!(out["cell_count"], 8);
        assert_eq!(
            out["material_counts"],
            json!({"air":3,"terrain":2,"building":2,"unknown":1})
        );
        let positions: Vec<_> = out["cells"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["min_cells"].clone())
            .collect();
        assert_eq!(
            positions,
            vec![
                json!([10, 20, 30]),
                json!([11, 20, 30]),
                json!([10, 21, 30]),
                json!([11, 21, 30]),
                json!([10, 20, 31]),
                json!([11, 20, 31]),
                json!([10, 21, 31]),
                json!([11, 21, 31])
            ]
        );
        assert_eq!(
            out["cells"][1]["material"],
            json!({"kind":"terrain","id":7})
        );
        assert_eq!(out["cells"][1]["fill"], 128);
        assert_eq!(
            out["cells"][2]["material"],
            json!({"kind":"building","id":157})
        );
        assert_eq!(out["cells"][2]["fill"], 0);
        assert_eq!(
            out["cells"][3]["material"],
            json!({"kind":"unknown","id":254})
        );
        assert_eq!(out["cells"][0]["material"], json!({"kind":"air","id":null}));
    }
}
