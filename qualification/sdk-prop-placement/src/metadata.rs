//! Shared copied-value contract for model tests and the opt-in native probe.
use enshwick::{ItemId, WorldError, world::World};
use serde_json::{Value, json};

pub(crate) fn template_hex(template: enshwick::content::Guid) -> String {
    template
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn copy_metadata(world: World, item: ItemId) -> Result<Value, WorldError> {
    let info = world.prop_placement_info(item)?;
    let local_box = info
        .local_box
        .map(|corners| json!({"min":corners.min,"max":corners.max}));
    let template = template_hex(info.template);
    Ok(
        json!({"source":"published_world_prop_placement_info","item_id":info.item.0,
        "template_guid_file_order_hex":template,"local_box_state":if local_box.is_some(){"declared"}else{"unknown"},
        "declared_local_box_m":local_box,"world_mutated":false,"physical_footprint_verified":false,
        "collision_verified":false,"build_permission_verified":false,"persistence_verified":false,
        "label_ownership_verified":false,"safe_recovery_verified":false}),
    )
}
