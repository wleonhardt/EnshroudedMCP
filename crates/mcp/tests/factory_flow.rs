//! Real factory adapter and SDK component decoding; synthetic state is not live crafting proof.
#[path = "../src/factory_station.rs"]
mod adapter;

mod tools {
    #[derive(Clone, Copy)]
    pub(crate) struct Game {
        pub(crate) world: enshwick::world::World,
        pub(crate) factory_station:
            Option<enshwick::typed::ComponentType<enshwick::typed::ecs::FactoryStation>>,
    }
}
use enshwick::abi::Status;
use enshwick::resources::fnv1a32;
use enshwick::{Entity, Load, Mod, Off};
use serde_json::{Value, json};
use std::{sync::Mutex, time::Duration};

#[path = "support/factory.rs"]
mod fixture;
use fixture::{FACTORY, types};
static GAME: Mutex<Option<tools::Game>> = Mutex::new(None);
static OBSERVED: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static SCHEDULER: Mutex<Option<enshwick::schedule::Scheduler>> = Mutex::new(None);
struct Probe;
impl Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let game = tools::Game {
            world: load.world()?,
            factory_station: Some(enshwick::typed::ComponentType::resolve(load)?),
        };
        *GAME.lock().unwrap() = Some(game);
        *SCHEDULER.lock().unwrap() = Some(load.scheduler()?);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Probe, "id = \"test.factory-flow\"\nside = \"server\"");

#[test]
fn sdk_reads_populated_factory_states_without_mutating_or_promoting_unknowns() {
    assert_eq!(adapter::tool().name, "factory_station_read");
    let hash = fnv1a32(FACTORY.as_bytes());
    let mut bytes = [0u8; 128];
    bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    let running = [(0u32, 255u8), (u32::MAX, 0), (9, 1), (9, 2)];
    let next = [(7u32, 3u8), (0, 255), (0, 0), (u32::MAX, 4)];
    for (base, choices) in [(16, running), (56, next)] {
        for (index, (id, selected)) in choices.into_iter().enumerate() {
            let at = base + index * 8;
            bytes[at..at + 4].copy_from_slice(&id.to_le_bytes());
            bytes[at + 4] = selected;
        }
    }
    bytes[96..104].copy_from_slice(&i64::MIN.to_le_bytes());
    bytes[104..112].copy_from_slice(&i64::MAX.to_le_bytes());
    bytes[116] = 1;
    bytes[118] = (1 << 3) | (1 << 4);
    let loader = enshwick::test::FakeHost::new()
        .world()
        .types(types())
        .type_hash("ecs::FactoryStation", hash)
        .type_size("ecs::FactoryStation", 128)
        .component(Entity(77), hash, &bytes)
        .install();
    // SAFETY: these exports are generated for this test's mod.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK,
        "{:?}",
        loader.logs()
    );
    let game = GAME.lock().unwrap().unwrap();
    let scheduler = SCHEDULER.lock().unwrap().unwrap();
    for (state, name) in [
        "NoRecipe",
        "MissingMaterials",
        "Paused",
        "Stopped",
        "Running",
        "MissingOutputSpace",
    ]
    .into_iter()
    .enumerate()
    {
        bytes[0] = state as u8;
        bytes[117] = u8::from(state == 3);
        assert!(loader.set_component(Entity(77), hash, &bytes));
        scheduler.next_tick(move || {
            OBSERVED.lock().unwrap().extend([
                adapter::read(game, 77).unwrap(),
                adapter::read(game, 88).unwrap(),
                adapter::read(
                    tools::Game {
                        factory_station: None,
                        ..game
                    },
                    77,
                )
                .unwrap(),
            ]);
        });
        assert!(OBSERVED.lock().unwrap().is_empty());
        let _ = loader.tick(Duration::from_millis(100));
        let reports = OBSERVED.lock().unwrap().split_off(0);
        assert_eq!(reports.len(), 3);
        let observed = &reports[0];
        let data = &observed["factory"];
        assert_eq!(observed["state"], "observed");
        assert_eq!(data["station_state"], name);
        assert_eq!(data["running_recipe_id"], u32::MAX);
        assert_eq!(data["next_recipe_id"], 0);
        for (key, choices) in [
            ("running_category_choices", running),
            ("next_category_choices", next),
        ] {
            assert_eq!(
                data[key],
                json!(choices.map(|(id, index)| json!({"category_id":id,"selected_index":index})))
            );
        }
        assert_eq!(data["recipe_start_raw"], i64::MIN);
        assert_eq!(data["recipe_pause_duration_raw"], i64::MAX);
        assert_eq!(data["use_all_available_category_ingredients"], true);
        assert_eq!(data["is_stopped_by_player"], state == 3);
        assert_eq!(
            data["sdk_missing_requirements"],
            json!(["InputMaterials", "OutputSpace"])
        );
        assert_eq!(reports[1]["state"], "not_observed");
        assert_eq!(reports[2]["state"], "unknown");
        for report in reports {
            for key in [
                "world_mutated",
                "complete_queue_verified",
                "crafting_eligibility_verified",
                "safe_removal_verified",
                "current_client_selection_verified",
            ] {
                assert_eq!(report[key], false);
            }
        }
        assert_eq!(loader.component_bytes(Entity(77), hash).unwrap(), bytes);
    }
    bytes[0] = 200;
    bytes[118] = 1 << 7;
    assert!(loader.set_component(Entity(77), hash, &bytes));
    scheduler.next_tick(move || {
        OBSERVED
            .lock()
            .unwrap()
            .push(adapter::read(game, 77).unwrap())
    });
    assert!(OBSERVED.lock().unwrap().is_empty());
    let _ = loader.tick(Duration::from_millis(100));
    let unknown = OBSERVED.lock().unwrap().split_off(0);
    assert!(
        unknown[0]["factory"]["station_state"]
            .as_str()
            .unwrap()
            .contains("200")
    );
    let flags = unknown[0]["factory"]["sdk_missing_requirements"]
        .as_array()
        .unwrap();
    assert_eq!(flags.len(), 1);
    assert!(flags[0].as_str().unwrap().contains('7'));
    assert_eq!(loader.component_bytes(Entity(77), hash).unwrap(), bytes);
}
