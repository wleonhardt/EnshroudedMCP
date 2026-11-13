//! Bounded voxel observations, called only after scheduling onto the game thread.

use enshwick::voxels::{VoxelBox, VoxelMaterial, VoxelPos};
use enshwick::world::BuildRefusal;
use policy::building_survey::Spec;
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_survey",
        description: "Read at most 4096 half-metre cells in a box at most 8 m per side, with whole-box persistence and optional connected-player build-right observations. May trigger chunk loading; retry an explicit not-loaded error later. No writes, prop/collision checks, crafting costs or complete altar boundary discovery.",
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
