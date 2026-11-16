//! Inactive consumer contract prototype; no MCP endpoint or native installation.
#![cfg(test)]
use enshwick::{
    Entity, ItemId, Load, Mod, Off,
    abi::{Side, Status},
    inventory::{InventoryCategory, ItemInstance, SlotRef},
    ui::{Frame, HudScreenType},
};
use serde_json::{Value, json};
use std::sync::Mutex;

fn copy_hover(frame: &Frame<'_>) -> Value {
    let slot = frame.hovered_slot();
    json!({"source":"published_inventory_hover_callback",
        "state":if slot.is_some(){"observed"}else{"not_observed"},
        "read_availability":"not_separately_exposed","entity_world":"client",
        "identity_scope":"this client UI callback; reacquire after screen/session changes",
        "hovered_slot":slot.map(|slot|json!({"opaque_slot_ref":format!("{:016x}",slot.at.0),
            "inventory_entity":slot.at.inventory().0,"slot_index":slot.at.index(),
            "item_id":slot.item.0,"count":slot.count,"item_instance_entity":slot.pide.map(|pide|pide.0),
            "category_raw":slot.category.0,"category_state":if slot.category==InventoryCategory::INVALID{"unknown"}else{"observed"}})),
        "current_selection_verified":false,"server_selectors_valid":false,
        "commands_submitted":false,"rendered_verified":false})
}

fn copy_tooltip(item: ItemInstance) -> Value {
    json!({"source":"published_item_tooltip_callback","item_id":item.item.0,
        "item_instance_entity":item.pide.0,"count":null,"entity_world":"client",
        "identity_scope":"this client item callback; never a server-world selector",
        "current_visibility_verified":false,"current_selection_verified":false,
        "server_selectors_valid":false,"ui_lines_added":false})
}

static HOVERS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static TOOLTIPS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
struct Observer;
impl Mod for Observer {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        load.extend_item_tooltip(|item, _lines| TOOLTIPS.lock().unwrap().push(copy_tooltip(item)))?;
        load.on_frame(|frame| HOVERS.lock().unwrap().push(copy_hover(frame)))?;
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Observer, "id = \"test.ui-feedback\"\nside = \"both\"");

fn started(side: Side) -> (enshwick::test::Installed, Status) {
    let loader = enshwick::test::FakeHost::new().side(side).install();
    HOVERS.lock().unwrap().clear();
    TOOLTIPS.lock().unwrap().clear();
    // SAFETY: these are this module's framework-generated exports.
    let status = unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) };
    (loader, status)
}

#[test]
fn published_api_and_append_only_abi_are_exactly_the_qualified_source_shape() {
    use enshwick::abi::{ABI_MAJOR, ABI_MINOR, HostApi};
    use std::mem::{offset_of, size_of};
    assert_eq!((ABI_MAJOR, ABI_MINOR), (12, 4));
    assert_eq!(offset_of!(HostApi, client_command_status), 24 + 206 * 8);
    assert_eq!(offset_of!(HostApi, terrain_material_add), 24 + 207 * 8);
    assert_eq!(offset_of!(HostApi, interaction_prompt), 24 + 208 * 8);
    assert_eq!(offset_of!(HostApi, ui_item_tooltip), 24 + 209 * 8);
    assert_eq!(size_of::<HostApi>(), 24 + 210 * 8);
}

#[test]
fn frame_copies_preserve_hovered_empty_and_extreme_slots_without_selection_or_dispatch() {
    let (loader, status) = started(Side::CLIENT);
    assert_eq!(status, Status::OK);
    loader.open_screen(Some(HudScreenType::Backpack));
    for (at, item, count, pide) in [
        (SlotRef::new(Entity(41), 3), ItemId(7), 12, Entity(0)),
        (SlotRef::new(Entity(42), 0), ItemId(8), 1, Entity(900)),
        (SlotRef::new(Entity(41), 7), ItemId(0), 0, Entity(0)),
        (
            SlotRef(u64::MAX),
            ItemId(u32::MAX),
            u32::MAX,
            Entity(u32::MAX),
        ),
    ] {
        loader.hover_slot(Some((at.0, item, count, pide)));
        assert!(loader.draw_frame().is_empty());
        let value = HOVERS.lock().unwrap().last().unwrap().clone();
        assert_eq!(value["state"], "observed");
        let slot = &value["hovered_slot"];
        assert_eq!(slot["opaque_slot_ref"], format!("{:016x}", at.0));
        assert_eq!(slot["inventory_entity"], at.inventory().0);
        assert_eq!(slot["slot_index"], at.index());
        assert_eq!(slot["item_id"], item.0);
        assert_eq!(slot["count"], count);
        assert_eq!(
            slot["item_instance_entity"],
            json!((pide != Entity(0)).then_some(pide.0))
        );
        assert_eq!(slot["category_raw"], InventoryCategory::INVALID.0);
        assert_eq!(slot["category_state"], "unknown");
        assert_eq!(value["entity_world"], "client");
        for field in [
            "current_selection_verified",
            "server_selectors_valid",
            "commands_submitted",
            "rendered_verified",
        ] {
            assert_eq!(value[field], false);
        }
    }
    loader.hover_slot(None);
    assert!(loader.draw_frame().is_empty());
    let none = HOVERS.lock().unwrap().last().unwrap().clone();
    assert_eq!(none["state"], "not_observed");
    assert!(none["hovered_slot"].is_null());
    assert_eq!(none["read_availability"], "not_separately_exposed");
    loader.hover_slot(Some((0, ItemId(7), 12, Entity(900))));
    assert!(loader.draw_frame().is_empty());
    assert!(HOVERS.lock().unwrap().last().unwrap()["hovered_slot"].is_null());
    loader.hover_slot(Some((
        SlotRef::new(Entity(41), 3).0,
        ItemId(7),
        12,
        Entity(0),
    )));
    loader.open_screen(None);
    assert!(loader.draw_frame().is_empty());
    assert!(HOVERS.lock().unwrap().last().unwrap()["hovered_slot"].is_null());
    assert!(loader.client_commands().is_empty());
}

#[test]
fn tooltip_callbacks_copy_item_identity_without_lines_counts_or_current_visibility_claims() {
    let (loader, status) = started(Side::CLIENT);
    assert_eq!(status, Status::OK);
    assert!(loader.open_tooltip(ItemId(7), Entity(0)).is_empty());
    assert!(TOOLTIPS.lock().unwrap().is_empty());
    for (item, pide) in [(ItemId(7), Entity(900)), (ItemId(8), Entity(901))] {
        assert!(loader.open_tooltip(item, pide).is_empty());
        let value = TOOLTIPS.lock().unwrap().last().unwrap().clone();
        assert_eq!(value["item_id"], item.0);
        assert_eq!(value["item_instance_entity"], pide.0);
        assert!(value["count"].is_null());
        assert_eq!(value["entity_world"], "client");
        for field in [
            "current_visibility_verified",
            "current_selection_verified",
            "server_selectors_valid",
            "ui_lines_added",
        ] {
            assert_eq!(value[field], false);
        }
    }
    assert!(loader.client_commands().is_empty());
}

#[test]
fn server_role_cannot_register_the_tooltip_observer() {
    let (loader, status) = started(Side::SERVER);
    assert_ne!(status, Status::OK);
    assert!(loader.open_tooltip(ItemId(7), Entity(900)).is_empty());
    assert!(HOVERS.lock().unwrap().is_empty());
    assert!(TOOLTIPS.lock().unwrap().is_empty());
}
