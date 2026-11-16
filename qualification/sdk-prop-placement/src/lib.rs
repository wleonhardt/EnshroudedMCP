//! Inactive source-only consumer probe; no MCP endpoint or installed provider.
#![cfg(test)]
use enshwick::{
    Entity, ItemId, Load, Mod, Off, WorldError, abi::Status, content::Guid, schedule::Scheduler,
    world::World,
};
use serde_json::{Value, json};
use std::sync::{Mutex, mpsc};

const ITEM: ItemId = ItemId::new(30);
const TEMPLATE: Guid = Guid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
static CONTEXT: Mutex<Option<(World, Scheduler)>> = Mutex::new(None);

fn copy_metadata(world: World, item: ItemId) -> Result<Value, WorldError> {
    let info = world.prop_placement_info(item)?;
    let local_box = info
        .local_box
        .map(|corners| json!({"min":corners.min,"max":corners.max}));
    let template = info
        .template
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(
        json!({"source":"published_world_prop_placement_info","item_id":info.item.0,
        "template_guid_file_order_hex":template,"local_box_state":if local_box.is_some(){"declared"}else{"unknown"},
        "declared_local_box_m":local_box,"world_mutated":false,"physical_footprint_verified":false,
        "collision_verified":false,"build_permission_verified":false,"persistence_verified":false,
        "label_ownership_verified":false,"safe_recovery_verified":false}),
    )
}

struct Consumer;
impl Mod for Consumer {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some((load.world()?, load.scheduler()?));
        Ok(Self)
    }
}
enshwick::test::declare_mod!(
    Consumer,
    "id = \"test.prop-placement-consumer\"\nside = \"server\""
);

fn start(fake: enshwick::test::FakeHost) -> (enshwick::test::Installed, World, Scheduler) {
    let host = fake
        .world()
        .item(ITEM, 1, 0)
        .item_places(ITEM, TEMPLATE)
        .install();
    *CONTEXT.lock().unwrap() = None;
    // SAFETY: these exports are generated for this test-only mod.
    assert_eq!(
        unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    let (world, scheduler) = CONTEXT.lock().unwrap().unwrap();
    (host, world, scheduler)
}

#[test]
fn exact_merged_abi_preserves_append_and_metadata_layout() {
    use enshwick::abi::{ABI_MAJOR, ABI_MINOR, HostApi, PropPlacementInfo};
    use std::mem::{offset_of, size_of};
    assert_eq!((ABI_MAJOR, ABI_MINOR), (12, 8));
    assert_eq!(size_of::<usize>(), 8);
    assert_eq!(offset_of!(HostApi, prop_placement_info), 24 + 212 * 8);
    assert_eq!(size_of::<HostApi>(), 1728);
    assert_eq!(size_of::<PropPlacementInfo>(), 52);
}

#[test]
fn scheduled_copy_preserves_file_order_identity_and_raw_corners_without_mutation() {
    let (host, world, scheduler) = start(
        enshwick::test::FakeHost::new()
            .without_entity_commands()
            .placement_box(ITEM, [-2., 0., -0.5], [3., 4., 0.5])
            .prop(Entity(40), ITEM, [100.; 3]),
    );
    let before = host.props();
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        scheduler.next_tick(move || tx.send(copy_metadata(world, ITEM)).unwrap())
    })
    .join()
    .unwrap();
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    let _ = host.tick(std::time::Duration::from_millis(100));
    let value = rx.try_recv().unwrap().unwrap();
    assert_eq!(value["item_id"], 30);
    assert_eq!(
        value["template_guid_file_order_hex"],
        "000102030405060708090a0b0c0d0e0f"
    );
    assert_eq!(value["local_box_state"], "declared");
    assert_eq!(
        value["declared_local_box_m"],
        json!({"min":[-2.,0.,-0.5],"max":[3.,4.,0.5]})
    );
    for field in [
        "world_mutated",
        "physical_footprint_verified",
        "collision_verified",
        "build_permission_verified",
        "persistence_verified",
        "label_ownership_verified",
        "safe_recovery_verified",
    ] {
        assert_eq!(value[field], false);
    }
    assert_eq!(copy_metadata(world, ITEM).unwrap(), value);
    assert_eq!(host.props(), before);
    assert!(host.transactions().is_empty());
}

#[test]
fn unknown_boxes_are_not_empty_geometry_and_sdk_corner_order_is_not_normalized() {
    for corners in [
        None,
        Some([[0.; 3]; 2]),
        Some([[f32::NAN; 3], [1.; 3]]),
        Some([[f32::INFINITY; 3], [1.; 3]]),
        Some([[1.; 3], [-1.; 3]]),
    ] {
        let fake = enshwick::test::FakeHost::new();
        let fake = if let Some([min, max]) = corners {
            fake.placement_box(ITEM, min, max)
        } else {
            fake
        };
        let (host, world, _) = start(fake);
        let value = copy_metadata(world, ITEM).unwrap();
        assert_eq!(value["local_box_state"], "unknown");
        assert!(value["declared_local_box_m"].is_null());
        assert_eq!(value["item_id"], 30);
        assert!(host.transactions().is_empty());
    }
    let (host, world, _) =
        start(enshwick::test::FakeHost::new().placement_box(ITEM, [3., -2., 1.], [-4., 5., 1.]));
    let value = copy_metadata(world, ITEM).unwrap();
    assert_eq!(value["local_box_state"], "declared");
    assert_eq!(
        value["declared_local_box_m"],
        json!({"min":[3.,-2.,1.],"max":[-4.,5.,1.]})
    );
    assert!(host.props().is_empty() && host.transactions().is_empty());
}

#[test]
fn native_refusal_nonprop_and_inactive_world_remain_errors() {
    let (host, world, _) = start(
        enshwick::test::FakeHost::new()
            .item(ItemId::new(31), 1, 0)
            .item(ItemId::new(32), 1, 0)
            .item_places(ItemId::new(32), TEMPLATE)
            .item_not_a_prop(ItemId::new(32)),
    );
    assert_eq!(
        copy_metadata(world, ItemId::new(31)),
        Err(WorldError::Refused)
    );
    assert_eq!(
        copy_metadata(world, ItemId::new(32)),
        Err(WorldError::NotAProp)
    );
    assert_eq!(
        copy_metadata(world, ItemId::new(99)),
        Err(WorldError::Refused)
    );
    host.stop_world();
    assert_eq!(copy_metadata(world, ITEM), Err(WorldError::Unavailable));
    assert!(host.props().is_empty() && host.transactions().is_empty());
}
