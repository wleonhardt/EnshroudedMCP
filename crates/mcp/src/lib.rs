//! An MCP endpoint on 127.0.0.1 that lets an agent read the running world.
//! The listener runs on a thread of its own; every game read is handed to the
//! game thread through the scheduler (`tools.rs`).

mod building_plan;
mod building_survey;
mod construction;
mod item_lookup;
mod knowledge;
mod recipe_plan;
mod reposition;
mod server;
mod tools;

use enshwick::{Load, Mod, Off};

/// The mod: the game handles the tools need.
#[derive(Debug)]
pub struct Mcp {
    tools: tools::GameTools,
}

impl Mod for Mcp {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let game = tools::Game {
            world: load.world()?,
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
        Ok(Self {
            tools: tools::GameTools::new(
                game,
                components,
                load.mod_dir().ok(),
                construction,
                reposition,
            ),
        })
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
