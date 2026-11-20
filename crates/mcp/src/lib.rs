//! An MCP endpoint on 127.0.0.1 that lets an agent read the running world.
//! The listener runs on a thread of its own; every game read is handed to the
//! game thread through the scheduler (`tools.rs`).

mod buff_read;
mod building_access;
mod building_plan;
mod building_survey;
mod clock_control;
mod construction;
mod factory_station;
mod give_item;
mod health_recovery;
mod interaction_target;
#[cfg(test)]
mod inventory_lifecycle;
mod item_lookup;
mod journal;
mod knowledge;
mod knowledge_read;
mod pacification;
mod player_buffs;
mod player_shroud;
mod player_vitals;
mod player_waypoint;
mod prop_observation;
mod recipe_plan;
mod reposition;
mod server;
mod terrain_image;
mod tools;
mod weather_read;
mod world_clock;

use enshwick::{Load, Mod, Off};

/// The mod: the game handles the tools need.
#[derive(Debug)]
pub struct Mcp {
    tools: tools::GameTools,
}

impl Mod for Mcp {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let game = tools::Game {
            factory_station:
                enshwick::typed::ComponentType::<enshwick::typed::ecs::FactoryStation>::resolve(
                    load,
                )
                .map_err(|e| enshwick::warn!("typed FactoryStation unavailable: {e}"))
                .ok(),
            interaction_targets: load
                .interaction_targets()
                .map_err(|e| enshwick::warn!("interaction target reader unavailable: {e}"))
                .ok(),
            world: load.world()?,
            native_knowledge: load.knowledge(),
            clock: load.clock(),
            weather: load.weather(),
            buffs: load.buffs(),
            voxels: load.voxels(),
            scheduler: load.scheduler()?,
            used_item: enshwick::ecs::ComponentType::resolve(load, "keen::ecs::UsedItem").ok(),
            static_used_item: enshwick::ecs::ComponentType::resolve(
                load,
                "keen::ecs::StaticUsedItem",
            )
            .ok(),
            inventories: load.inventories().ok(),
            item_layout: enshwick::content::items::ItemLayout::resolve(load).ok(),
            game_settings: load.game_settings().ok(),
            network_health:
                enshwick::typed::ComponentType::<enshwick::typed::ecs::NetworkHealth>::resolve(load)
                    .map_err(|e| enshwick::warn!("typed NetworkHealth unavailable: {e}"))
                    .ok(),
        };
        let components = tools::resolve_components(load);
        let construction = construction::Construction::new(load)
            .map_err(|error| enshwick::warn!("construction unavailable: {error}"))
            .ok();
        let reposition = construction.clone().and_then(|construction| {
            reposition::Reposition::new(load, construction)
                .map_err(|error| enshwick::warn!("reposition unavailable: {error}"))
                .ok()
        });
        let recovery = construction.clone().and_then(|construction| {
            health_recovery::Recovery::new(load, construction)
                .map_err(|error| enshwick::warn!("health recovery unavailable: {error}"))
                .ok()
        });
        let mut tools = tools::GameTools::new(
            game,
            components,
            load.mod_dir().ok(),
            construction,
            reposition,
            recovery,
        );
        tools.provider_header = Some(load.host().api().header);
        tools.waypoint_map = load
            .map()
            .map_err(|error| enshwick::warn!("native map service unavailable: {error}"))
            .ok();
        tools.register_shutdown_events(load);
        Ok(Self { tools })
    }

    fn ready(&'static self, _host: enshwick::Host) {
        let config = server::config();
        let spawned = std::thread::Builder::new()
            .name("enshrouded-mcp".into())
            .spawn(move || server::run(&config, &self.tools));
        if let Err(e) = spawned {
            enshwick::error!("could not start the listener thread: {e}");
        }
    }
}

enshwick::declare_mod!(Mcp);

mod server_shutdown;

mod server_save;

#[cfg(test)]
mod provider_tests {
    use super::*;
    use policy::mcp::Tools;

    struct Recorder;
    impl Mod for Recorder {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let mcp = Mcp::init(load)?;
            let header = load.host().api().header;
            let value = mcp
                .tools
                .call("capabilities", &serde_json::Map::new())
                .unwrap();
            assert_eq!(value["runtime_framework"]["state"], "observed");
            assert_eq!(value["runtime_framework"]["abi_major"], header.abi_major);
            assert_eq!(value["runtime_framework"]["abi_minor"], header.abi_minor);
            assert_eq!(value["runtime_framework"]["api_table_bytes"], header.size);
            assert_eq!(value["compiled_framework_abi"], "12.1");
            assert_eq!(value["new_inspection_tools_live_verified"], false);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Recorder,
        r#"id = "test.provider-header"
side = "server""#
    );

    #[test]
    fn initialization_copies_published_header_without_starting_listener_or_world_reads() {
        let loader = enshwick::test::FakeHost::new()
            .side(enshwick::abi::Side::SERVER)
            .world()
            .install();
        // SAFETY: exports belong to the test recorder; its ready starts no listener.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
    }
}

#[cfg(test)]
mod factory_station_lifecycle;
