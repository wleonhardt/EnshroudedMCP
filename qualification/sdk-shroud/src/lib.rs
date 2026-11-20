//! Inactive server shroud observation; no MCP registration, setter or installation.
use enshwick::{Entity, world::World};
use serde_json::{Value, json};

/// Read only on the server game thread; entity IDs last one server session.
///
/// # Errors
/// Refuse the zero entity sentinel without manufacturing a default observation.
pub fn observe(world: World, entity: Entity) -> Result<Value, &'static str> {
    if entity.0 == 0 {
        return Err("server entity must be nonzero");
    }
    let timer = world.shroud_time(entity);
    let inside = world.in_shroud(entity);
    let state = match (timer.is_some(), inside.is_some()) {
        (true, true) => "observed",
        (false, false) => "unavailable",
        _ => "partial",
    };
    Ok(json!({
        "source":"published_world_shroud", "entity":entity.0, "state":state,
        "in_shroud":inside,"timer_seconds":timer.map(|m|json!({"current":m.current,"max":m.max})),
        "units":"SDK integer seconds; UI correspondence unverified",
        "missing_semantics":"absent, unreadable or unsupported; no safe-time or outside-shroud default",
        "entity_scope":"server entity; session only, never a client or durable selector",
        "world_session":"unavailable","atomic_snapshot":false,
        "world_mutated":false,"write_authorized":false,"current_player_verified":false,
        "rendered_ui_verified":false,"traversal_safety_verified":false,
        "damage_deadline_verified":false,"refill_requested":false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Load, Mod, Off, abi::Status, schedule::Scheduler, world::Meter};
    use serde_json::json;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    static HANDLES: Mutex<Option<(World, Scheduler)>> = Mutex::new(None);
    const CHARACTER: Entity = Entity(17);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *HANDLES.lock().unwrap() = Some((load.world()?, load.scheduler()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.shroud-reader\"\nside = \"server\"");

    // Synthetic SDK attribute sets, using the published fake's table contract.
    // These values are fixture layout, not offsets obtained from a game process.
    fn timer(current: i32, max: i32) -> Vec<u8> {
        let mut bytes = 60u32.to_le_bytes().to_vec();
        bytes.extend(8u16.to_le_bytes());
        bytes.extend([1, 2]);
        bytes.extend(current.to_le_bytes());
        bytes.extend(max.to_le_bytes());
        bytes
    }
    fn host(timer_layout: bool, actor_layout: bool) -> enshwick::test::FakeHost {
        let mut host = enshwick::test::FakeHost::new()
            .world()
            .without_entity_commands()
            .constant("ecs.level_data_offset", 4)
            .constant("ecs.level_flags", 6)
            .constant("ecs.attribute_count", 7);
        for (index, name) in ["health", "stamina", "mana", "level", "experience"]
            .iter()
            .enumerate()
        {
            host = host
                .type_hash(&format!("attribute_set.{name}"), index as u32 + 1)
                .constant(&format!("attribute.{name}"), (index as u64 + 1) * 10);
        }
        for name in ["health", "stamina", "mana"] {
            host = host.constant(&format!("attribute.{name}_max_slot"), 1);
        }
        host = host.constant("attribute.experience_needed_slot", 1);
        if timer_layout {
            host = host
                .type_hash("attribute_set.shroud", 6)
                .constant("attribute.shroud", 60)
                .constant("attribute.shroud_max_slot", 1);
        }
        if actor_layout {
            host = host
                .type_hash("base_actor", 7)
                .type_size("base_actor", 8)
                .constant("base_actor.state", 0)
                .constant("actor.state_in_shroud", 4);
        }
        host
    }
    fn start(host: enshwick::test::FakeHost) -> (enshwick::test::Installed, World, Scheduler) {
        let loader = host.install();
        // SAFETY: these generated exports belong to this fake-only reader.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let (world, scheduler) = HANDLES.lock().unwrap().unwrap();
        (loader, world, scheduler)
    }
    fn limits(value: &Value) {
        for key in [
            "world_mutated",
            "write_authorized",
            "current_player_verified",
            "rendered_ui_verified",
            "traversal_safety_verified",
            "damage_deadline_verified",
            "refill_requested",
        ] {
            assert_eq!(value[key], false);
        }
        assert_eq!(value["world_session"], "unavailable");
    }

    #[test]
    fn sdk_zero_timer_and_false_are_observed_even_without_health() {
        let (_loader, world, _) = start(
            host(true, true)
                .component(CHARACTER, 6, &timer(0, 300))
                .component(CHARACTER, 7, &0u64.to_le_bytes()),
        );
        assert_eq!(
            world.shroud_time(CHARACTER),
            Some(Meter {
                current: 0,
                max: 300
            })
        );
        assert_eq!(world.in_shroud(CHARACTER), Some(false));
        assert!(world.vitals(CHARACTER).is_none());
        let value = observe(world, CHARACTER).unwrap();
        assert_eq!(value["state"], "observed");
        assert_eq!(value["in_shroud"], false);
        assert_eq!(value["timer_seconds"], json!({"current":0,"max":300}));
        limits(&value);
    }
    #[test]
    fn missing_component_and_unavailable_layout_never_default_to_safe_values() {
        for host in [
            host(true, true),
            enshwick::test::FakeHost::new()
                .world()
                .without_entity_commands(),
        ] {
            let (_loader, world, _) = start(host);
            let value = observe(world, CHARACTER).unwrap();
            assert_eq!(value["state"], "unavailable");
            assert!(value["in_shroud"].is_null());
            assert!(value["timer_seconds"].is_null());
            limits(&value);
        }
    }
    #[test]
    fn independent_fields_keep_partial_observations() {
        let (loader, world, _) = start(host(true, false).component(CHARACTER, 6, &timer(12, 300)));
        let value = observe(world, CHARACTER).unwrap();
        assert_eq!(value["state"], "partial");
        assert!(value["in_shroud"].is_null());
        assert_eq!(value["timer_seconds"]["current"], 12);
        limits(&value);
        drop(loader);
        let (_loader, world, _) =
            start(host(false, true).component(CHARACTER, 7, &4u64.to_le_bytes()));
        let value = observe(world, CHARACTER).unwrap();
        assert_eq!(value["state"], "partial");
        assert_eq!(value["in_shroud"], true);
        assert!(value["timer_seconds"].is_null());
        limits(&value);
    }
    #[test]
    fn signed_raw_pool_values_are_never_scaled_or_clamped() {
        for (current, max) in [(137, 111), (-7, 0), (i32::MIN, i32::MAX)] {
            let (_loader, world, _) = start(
                host(true, true)
                    .component(CHARACTER, 6, &timer(current, max))
                    .component(CHARACTER, 7, &4u64.to_le_bytes()),
            );
            let value = observe(world, CHARACTER).unwrap();
            assert_eq!(value["timer_seconds"], json!({"current":current,"max":max}));
            assert_eq!(world.shroud_time(CHARACTER), Some(Meter { current, max }));
            limits(&value);
        }
    }
    #[test]
    fn malformed_sdk_sets_remain_unknown_instead_of_zero() {
        for (offset, replacement) in [(0, 99u8), (6, 0), (7, 1)] {
            let mut bytes = timer(10, 300);
            bytes[offset] = replacement;
            let (_loader, world, _) = start(host(true, false).component(CHARACTER, 6, &bytes));
            assert!(world.shroud_time(CHARACTER).is_none());
            let value = observe(world, CHARACTER).unwrap();
            assert!(value["timer_seconds"].is_null());
            assert_eq!(value["state"], "unavailable");
        }
    }
    #[test]
    fn subsequent_reads_observe_changed_timer_and_actor_state_without_cache() {
        let (loader, world, _) = start(
            host(true, true)
                .component(CHARACTER, 6, &timer(60, 300))
                .component(CHARACTER, 7, &4u64.to_le_bytes()),
        );
        assert_eq!(
            observe(world, CHARACTER).unwrap()["timer_seconds"]["current"],
            60
        );
        assert!(loader.set_component(CHARACTER, 6, &timer(59, 300)));
        assert!(loader.set_component(CHARACTER, 7, &0u64.to_le_bytes()));
        let value = observe(world, CHARACTER).unwrap();
        assert_eq!(value["timer_seconds"]["current"], 59);
        assert_eq!(value["in_shroud"], false);
        assert_eq!(
            world.shroud_time(CHARACTER),
            Some(Meter {
                current: 59,
                max: 300
            })
        );
        limits(&value);
    }
    #[test]
    fn zero_entity_refuses_without_a_default_observation() {
        let (_loader, world, _) = start(host(true, true));
        assert_eq!(
            observe(world, Entity(0)).unwrap_err(),
            "server entity must be nonzero"
        );
    }
    #[test]
    fn scheduler_defers_read_until_server_tick() {
        let (loader, world, scheduler) = start(
            host(true, true)
                .component(CHARACTER, 6, &timer(30, 300))
                .component(CHARACTER, 7, &4u64.to_le_bytes()),
        );
        let result = Arc::new(Mutex::new(None));
        let copied = Arc::clone(&result);
        scheduler.next_tick(move || *copied.lock().unwrap() = Some(observe(world, CHARACTER)));
        assert!(result.lock().unwrap().is_none());
        let _ = loader.tick(Duration::from_millis(1));
        let value = result.lock().unwrap().take().unwrap().unwrap();
        assert_eq!(value["timer_seconds"]["current"], 30);
        limits(&value);
    }
}
