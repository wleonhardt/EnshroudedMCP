//! The read-only tools. Each game read runs on the game thread: the listener
//! queues it with `Scheduler::next_tick` and waits for the answer.

use std::sync::{Arc, mpsc};
use std::time::Duration;

use enshwick::ecs::ComponentType;
use enshwick::schedule::Scheduler;
use enshwick::voxels::{VoxelPos, Voxels};
use enshwick::world::{ContainerKinds, Placed, World};
use policy::mcp::{Tool, Tools};
use policy::{args, bounds, inspection, write_gate};
use serde_json::{Map, Value, json};

/// How long a request waits for the game thread.
const GAME_WAIT: Duration = Duration::from_secs(5);
/// The longest side of a `props_in` box, in metres; enshwick calls the read slow.
const PROPS_BOX: f64 = 64.0;
/// The largest `entities_near` radius, in metres.
const NEAR_RADIUS: f64 = 32.0;
/// The most entities one `entities_near` answer lists.
const NEAR_MAX: usize = 200;
/// The most entities one `entity_components` answer describes.
const DESCRIBE_MAX: usize = 20;

pub(crate) fn give_gate(
    dry: bool,
    dir: Option<&std::path::Path>,
    confirm: Option<&str>,
) -> Result<(), String> {
    let age = dir.and_then(|d| write_gate::flag_age(&d.join(write_gate::FLAG)));
    write_gate::check(dry, true, age, confirm)
}

pub(crate) fn give_dispatch<T>(
    dir: Option<&std::path::Path>,
    confirm: Option<&str>,
    dispatch: impl FnOnce() -> T,
) -> Result<T, String> {
    give_gate(false, dir, confirm)?;
    Ok(dispatch())
}

/// The component types this build reflects, by the game's name, read from
/// `components.txt` in the mod's folder (`scripts/component_names.py`).
pub(crate) type Components = Arc<Vec<(String, ComponentType)>>;

/// Resolves every name in `components.txt`; a missing file or a name this
/// build does not reflect is logged and skipped.
pub(crate) fn resolve_components(load: &enshwick::Load<'_>) -> Components {
    let path = match load.mod_dir() {
        Ok(dir) => dir.join("components.txt"),
        Err(e) => {
            enshwick::warn!("no mod folder, so no component names: {e}");
            return Arc::default();
        }
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        enshwick::warn!(
            "{} not found: entity_components lists nothing",
            path.display()
        );
        return Arc::default();
    };
    let (mut found, mut missing) = (Vec::new(), 0);
    for name in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        match ComponentType::resolve(load, name) {
            Ok(ty) => found.push((name.to_owned(), ty)),
            Err(_) => missing += 1, // allow: counted and logged below
        }
    }
    enshwick::info!(
        "{} component types resolved, {missing} not on this build",
        found.len()
    );
    Arc::new(found)
}

/// The game handles; all are plain copies, safe to move between threads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Game {
    pub(crate) interaction_targets: Option<enshwick::interactions::Targets>,
    pub(crate) world: World,
    pub(crate) native_knowledge: enshwick::knowledge::Knowledge,
    pub(crate) clock: enshwick::daytime::Clock,
    pub(crate) weather: enshwick::weather::Weather,
    pub(crate) buffs: enshwick::buffs::Buffs,
    pub(crate) game_settings: Option<enshwick::game_settings::GameSettings>,
    pub(crate) voxels: Voxels,
    pub(crate) scheduler: Scheduler,
    /// `keen::ecs::UsedItem` and `keen::ecs::StaticUsedItem`; `None` on a
    /// build that does not reflect them.
    pub(crate) used_item: Option<ComponentType>,
    pub(crate) static_used_item: Option<ComponentType>,
    /// `None` on a build without the inventory transactions.
    pub(crate) inventories: Option<enshwick::inventory::Inventories>,
    pub(crate) item_layout: Option<enshwick::content::items::ItemLayout>,
    /// Resolved and shape-checked against the running build during init.
    pub(crate) network_health:
        Option<enshwick::typed::ComponentType<enshwick::typed::ecs::NetworkHealth>>,
    pub(crate) factory_station:
        Option<enshwick::typed::ComponentType<enshwick::typed::ecs::FactoryStation>>,
}

#[derive(Debug)]
pub(crate) struct GameTools {
    game: Game,
    components: Components,
    /// The mod's folder: the write flag and the journal live there.
    mod_dir: Option<std::path::PathBuf>,
    construction: Option<crate::construction::Construction>,
    reposition: Option<crate::reposition::Reposition>,
    recovery: Option<crate::health_recovery::Recovery>,
    clock_control: Option<crate::clock_control::ClockControl>,
    shutdown: Option<crate::server_shutdown::Shutdown>,
    save: Option<crate::server_save::SaveControl>,
    grants: Option<crate::give_item::Grants>,
    list: Vec<Tool>,
}

impl GameTools {
    pub(crate) fn register_shutdown_events(&self, load: &mut enshwick::Load<'_>) {
        if let Some(shutdown) = &self.shutdown {
            shutdown.register_events(load);
        }
    }

    pub(crate) fn new(
        game: Game,
        components: Components,
        mod_dir: Option<std::path::PathBuf>,
        construction: Option<crate::construction::Construction>,
        reposition: Option<crate::reposition::Reposition>,
        recovery: Option<crate::health_recovery::Recovery>,
    ) -> Self {
        let point = |extra: &str| {
            json!({"type": "object", "properties": {
                format!("x{extra}"): {"type": "number"}, format!("y{extra}"): {"type": "number"}, format!("z{extra}"): {"type": "number"},
            }})
        };
        let mut box_schema = point("1");
        let second = point("2");
        for (k, v) in second["properties"].as_object().into_iter().flatten() {
            box_schema["properties"][k] = v.clone();
        }
        box_schema["required"] = json!(["x1", "y1", "z1", "x2", "y2", "z2"]);
        let mut sample = json!({"type": "object", "properties": {"x": {"type": "number"}, "y": {"type": "number"}, "z": {"type": "number"}}});
        sample["required"] = json!(["x", "y", "z"]);
        let mut describe = sample.clone();
        describe["properties"]["radius"] = json!({"type": "number", "maximum": 16});
        describe["properties"]["limit"] = json!({"type": "integer", "maximum": DESCRIBE_MAX});
        let mut near = sample.clone();
        near["properties"]["radius"] = json!({"type": "number", "maximum": NEAR_RADIUS});
        near["properties"]["kind"] = json!({"type": "string", "enum": ["placed", "used_item", "static_used_item", "containers"]});
        let mut health = sample.clone();
        health["properties"]["radius"] =
            json!({"type":"number", "maximum":inspection::HEALTH_RADIUS});
        health["properties"]["limit"] =
            json!({"type":"integer", "minimum":1, "maximum":inspection::HEALTH_LIMIT});
        let mut list = vec![
            crate::weather_read::tool(),
            crate::buff_read::tool(),
            crate::factory_station::tool(),
            crate::journal::tool(),
            crate::knowledge_read::tool(),
            crate::interaction_target::tool(),
            crate::building_access::tool(),
            Tool {
                name: "player_restore_health",
                description: "ADMIN WRITE, dry by default. Restore the sole connected living player's health to its observed maximum. Exact actor/session/generation/health preview, fresh write flag and confirmation required. Queued work cancels on timeout before execution. Synced intent and read-back outcome; unknown outcomes require observation, not automatic retry. No damage, revive, item use, invulnerability, undo or replay protection.",
                schema: policy::health_recovery::schema(),
            },
            crate::building_survey::tool(),
            Tool {
                name: "enemy_pacification",
                description: "Read or explicitly set only server enemy pacification. Dry by default; real changes require expected current value, active session, fresh write flag and confirmation. Not invulnerability or combat safety. Configuration saves may persist changes; restore explicitly.",
                schema: policy::pacification::schema(),
            },
            crate::item_lookup::tool(),
            Tool {
                name: "capabilities",
                description: "Resolved inspection handles and limits in this mod process. Availability is initialization evidence, not proof of successful live reads. Underlying nearby-entity discovery and inventory-link traversal are not capped by output limits.",
                schema: json!({"type":"object"}),
            },
            Tool {
                name: "inventory_read",
                description: "Read a page of slots from exactly one connected player (name/account key) or session-only entity ID. Empty slots included. Optional item_ids (1..8 unique nonzero IDs) adds physical-slot-deduplicated counts over the same SDK slot read, independent of page; completeness and crafting eligibility remain unknown. Counts refuse more than 65536 returned records or conflicting aliases. Paging caps output and per-slot metadata, not upstream traversal. Offsets are not stable across requests. Empty output cannot distinguish absent/uninitialized inventory. Default 64, maximum 128 slots.",
                schema: json!({"type":"object","properties": {
                    "player":{"type":"string"},
                    "entity":{"type":"integer","minimum":1,"maximum":u32::MAX},
                    "offset":{"type":"integer","minimum":0,"maximum":inspection::SLOT_OFFSET_MAX},
                    "limit":{"type":"integer","minimum":1,"maximum":inspection::SLOT_LIMIT},
                    "item_ids":policy::item_lookup::schema()["properties"]["ids"]
                }}),
            },
            Tool {
                name: "item_info",
                description: "Resolve one vanilla item debug name and read static stack-size, category, capability flags and placed-entity template mapping from the running world. Known absent templates and unavailable metadata are distinct. Mapping does not establish placement eligibility, footprint, build rights, spawn authorization or crafting feasibility.",
                schema: json!({"type":"object","properties":{"item":{"type":"string"}},"required":["item"]}),
            },
            Tool {
                name: "entity_health",
                description: "Typed NetworkHealth for nearest placed entities within radius (default 3 m, maximum 16), at most limit (default 10, maximum 20). Reports absent components and decode errors separately; does not scan all combat actors. Output is capped, underlying spatial discovery is not.",
                schema: health,
            },
            Tool {
                name: "status",
                description: "Whether a world runs here, its key, save, player count and published server clock (hours, day/night, configured lengths and current speed). Unavailable differs from midnight; clock observations do not verify rendered lighting.",
                schema: json!({"type": "object"}),
            },
            Tool {
                name: "players",
                description: "Every connected player: name (client-supplied), account key, slot, admin, position in metres (y up), and nullable native health/stamina/mana meters. Raw SDK integer values; unavailable differs from zero and UI conversion is unverified.",
                schema: json!({"type": "object"}),
            },
            Tool {
                name: "voxel_sample",
                description: "The voxel at a point in metres: material (air, terrain id or building id) and fill 0-255.",
                schema: sample,
            },
            Tool {
                name: "props_in",
                description: "Placed props in a box (each side at most 64 m): session-only entity, item id, transform, observed template GUID in file-order hex and nullable label presence. At most 500; metadata reads only for returned rows. Label presence is not label text or proof of safe undo; other held state remains unknown.",
                schema: box_schema,
            },
            Tool {
                name: "entities_near",
                description: "Entities within radius m (default 8, at most 32) of a point, nearest first, by kind: placed (every placed entity, the default), used_item / static_used_item (those with that component), containers (built chests, showcases, stations). Each: entity (this session only), position, distance, item it was placed from, template GUID, which of UsedItem / StaticUsedItem it has. At most 200.",
                schema: near,
            },
            Tool {
                name: "entity_components",
                description: "Every reflected ECS component of the placed entities within radius m (default 3, at most 16) of a point, nearest first, by the game's type names (keen::ecs::...). Each: entity (this session only), position, distance, template GUID, components. At most `limit` entities (default 10, at most 20).",
                schema: describe,
            },
            Tool {
                name: "give_item",
                description: "ADMIN WRITE: caller-identified inventory grant, dry by default. Required operation/player/item; count 1..100. Real calls require exact current actor/item/session preview, fresh flag and confirm server-write. Repeat the original payload dry to inspect retained status without redispatch. Payload conflicts, stale sessions and reopened durable IDs refuse writes. At most 64 identities. Native callback commit is not inventory read-back; no automatic retry or undo. Removing a flag cannot cancel a submitted SDK request.",
                schema: policy::give_item::schema(),
            },
        ];
        list.extend(offline_tools());
        list.extend(crate::construction::tools());
        list.push(Tool {
            name: "player_reposition",
            description: "ADMIN WRITE, dry by default. Submit one bounded server teleport request for the sole connected living player, using engine free-spot search. Requested target must be within 16 m of observed position. Exact actor/session preview, fresh write flag and confirmation required. Durable same-ID replay never dispatches again. Submission is not arrival; spawn adjustment is unbounded by this interface. No walking, camera, menu, native undo or automatic retry. Observe players separately; return requires a new gated request.",
            schema: policy::reposition::schema(),
        });
        list.push(crate::reposition::status_tool());
        list.push(crate::clock_control::tool());
        list.push(crate::server_shutdown::tool());
        list.push(crate::server_shutdown::status_tool());
        list.push(crate::server_save::tool());
        let grants = mod_dir
            .clone()
            .zip(construction.clone())
            .map(|(dir, lifecycle)| crate::give_item::Grants::new(dir, lifecycle));
        let save = mod_dir
            .clone()
            .zip(construction.clone())
            .map(|(dir, lifecycle)| crate::server_save::SaveControl::new(dir, lifecycle));
        let shutdown = mod_dir
            .clone()
            .zip(construction.clone())
            .map(|(dir, lifecycle)| crate::server_shutdown::Shutdown::new(dir, lifecycle));
        let clock_control = mod_dir
            .clone()
            .zip(construction.clone())
            .map(|(dir, lifecycle)| crate::clock_control::ClockControl::new(dir, lifecycle));
        Self {
            game,
            components,
            mod_dir,
            construction,
            reposition,
            recovery,
            clock_control,
            shutdown,
            save,
            grants,
            list,
        }
    }
}

impl Tools for GameTools {
    fn list(&self) -> &[Tool] {
        &self.list
    }

    fn call(&self, name: &str, a: &Map<String, Value>) -> Result<Value, String> {
        if name == "construction_budget" {
            return crate::journal::report(
                self.mod_dir.as_deref().ok_or("mod folder unavailable")?,
                a,
            );
        }
        if let Some(result) = offline_call(name, a) {
            return result;
        }
        let Game {
            world,
            clock,
            weather,
            buffs,
            game_settings,
            voxels,
            scheduler,
            used_item,
            static_used_item,
            inventories,
            item_layout,
            network_health,
            factory_station,
            interaction_targets,
            native_knowledge: _,
        } = self.game;
        match name {
            "entity_buffs_read" => {
                let (entity, limit) = policy::buff_read::check(a)?;
                on_game_thread(scheduler, move || {
                    let mut result = crate::buff_read::read(buffs, entity, limit)?;
                    result["world_key"] = json!(world.key());
                    Ok(result)
                })?
            }
            "weather_read" => {
                let position = policy::weather_read::check(a)?;
                on_game_thread(scheduler, move || {
                    let mut result = crate::weather_read::read(weather, position);
                    result["world_key"] = json!(world.key());
                    result
                })
            }
            "factory_station_read" => {
                let entity = policy::factory_station::check(a)?;
                let game = self.game;
                on_game_thread(scheduler, move || {
                    crate::factory_station::read(game, entity)
                })?
            }
            "knowledge_read" => {
                let spec = policy::knowledge_read::check(a)?;
                let game = self.game;
                on_game_thread(scheduler, move || crate::knowledge_read::read(game, spec))?
            }
            "player_interaction_target" => {
                let selector = policy::interaction_target::check(a)?;
                let game = self.game;
                on_game_thread(scheduler, move || {
                    crate::interaction_target::read(game, &selector)
                })?
            }
            "building_access" => {
                let spec = policy::building_access::check(a)?;
                on_game_thread(scheduler, move || {
                    let actor = spec
                        .player
                        .as_deref()
                        .map(|selector| {
                            inspection::unique_player(world.players().into_iter().filter(|p| {
                                policy::building_access::matches_player(selector, &p.name, p.key.0)
                            }))
                            .map_err(|_| {
                                "player selector must match exactly one connected actor".to_owned()
                            })
                        })
                        .transpose()?;
                    let persistence = crate::building_access::persistence(
                        world
                            .persists(spec.min, spec.max)
                            .map_err(|e| e.to_string()),
                    );
                    let rights = crate::building_access::rights(
                        actor
                            .as_ref()
                            .map(|p| world.can_build(p, spec.min, spec.max)),
                    );
                    Ok::<_, String>(
                        json!({"world_key":world.key(),"min":spec.min,"max":spec.max,
                        "player":actor.as_ref().map(|p|json!({"key":p.key.0.to_string(),"entity":p.entity.0})),
                        "persistence":persistence,"build_rights":rights,"world_mutated":false,
                        "scope":"explicit caller box only; not an item footprint, collision/contact query, save/reload proof or authorization"}),
                    )
                })?
            }
            "player_restore_health" => {
                let spec = policy::health_recovery::check(a)?;
                self.recovery
                    .as_ref()
                    .ok_or("native health recovery unavailable")?
                    .call(self.game, spec)
            }
            "enemy_pacification" => {
                let spec = policy::pacification::check(a)?;
                crate::pacification::gate(&spec, self.mod_dir.as_deref())?;
                let settings = game_settings.ok_or("game settings unavailable")?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("world lifecycle unavailable")?;
                let admitted = construction.active_session()?;
                let dir = self.mod_dir.clone();
                on_game_thread(scheduler, move || {
                    let active = construction.active_session()?;
                    policy::construction::check_session(&admitted, &active)?;
                    crate::pacification::run(settings, spec, &active, dir.as_deref())
                })?
            }
            "server_save" => {
                let spec = policy::server_save::check(a)?;
                self.save
                    .as_ref()
                    .ok_or("save lifecycle unavailable")?
                    .call(self.game.world, scheduler, spec)
            }
            "server_shutdown_status" => {
                let operation = policy::server_shutdown::check_status(a)?;
                self.shutdown
                    .as_ref()
                    .ok_or("shutdown lifecycle unavailable")?
                    .status(&operation)
            }
            "server_shutdown" => {
                let spec = policy::server_shutdown::check(a)?;
                self.shutdown
                    .as_ref()
                    .ok_or("shutdown lifecycle unavailable")?
                    .call(self.game.world, scheduler, spec)
            }
            "clock_advance" => {
                let spec = policy::clock_control::check(a)?;
                self.clock_control
                    .as_ref()
                    .ok_or("clock control lifecycle unavailable")?
                    .call(self.game, spec)
            }
            "player_reposition_status" => {
                let operation = policy::reposition::check_status(a)?;
                let reposition = self
                    .reposition
                    .clone()
                    .ok_or("native reposition unavailable")?;
                let game = self.game;
                on_game_thread(scheduler, move || reposition.observe(game, &operation))?
            }
            "player_reposition" => {
                let spec = policy::reposition::check(a)?;
                let reposition = self
                    .reposition
                    .clone()
                    .ok_or("native reposition unavailable")?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("world lifecycle unavailable")?;
                let admitted_session = construction.active_session()?;
                let game = self.game;
                on_game_thread(scheduler, move || {
                    policy::construction::check_session(
                        &admitted_session,
                        &construction.active_session()?,
                    )?;
                    reposition.run(game, spec)
                })?
            }
            "building_apply_plan" => {
                let spec = policy::construction::check_plan(a)?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("construction unavailable")?;
                let admission_session = construction.session();
                let game = self.game;
                on_game_thread(scheduler, move || {
                    policy::construction::check_session(
                        &admission_session,
                        &construction.session(),
                    )
                    .map_err(|_| "world session changed while request was queued".to_owned())?;
                    construction.apply_plan(game, spec)
                })?
            }
            "building_place_cell" => {
                let spec = policy::construction::check_place(a)?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("construction unavailable")?;
                let admission_session = construction.session();
                let game = self.game;
                on_game_thread(scheduler, move || {
                    policy::construction::check_session(
                        &admission_session,
                        &construction.session(),
                    )
                    .map_err(|_| "world session changed while request was queued".to_owned())?;
                    construction.place(game, spec)
                })?
            }
            "building_operation" => {
                let operation = policy::construction::check_status(a)?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("construction unavailable")?;
                let game = self.game;
                on_game_thread(scheduler, move || construction.status(game, operation))?
            }
            "building_accept" => {
                let spec = policy::construction::check_accept(a)?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("construction unavailable")?;
                let admission_session = construction.session();
                let game = self.game;
                on_game_thread(scheduler, move || {
                    policy::construction::check_session(
                        &admission_session,
                        &construction.session(),
                    )
                    .map_err(|_| "world session changed while request was queued".to_owned())?;
                    construction.accept(game, spec)
                })?
            }
            "building_undo_cell" => {
                let spec = policy::construction::check_undo(a)?;
                let construction = self
                    .construction
                    .clone()
                    .ok_or("construction unavailable")?;
                let admission_session = construction.session();
                let game = self.game;
                on_game_thread(scheduler, move || {
                    policy::construction::check_session(
                        &admission_session,
                        &construction.session(),
                    )
                    .map_err(|_| "world session changed while request was queued".to_owned())?;
                    construction.undo(game, spec)
                })?
            }
            "building_survey" => {
                let spec = policy::building_survey::check(a)?;
                let game = self.game;
                on_game_thread(scheduler, move || crate::building_survey::read(game, spec))?
            }
            "item_lookup" => {
                let ids = policy::item_lookup::check(a)?;
                let layout = item_layout.ok_or("vanilla item registry layout unavailable")?;
                on_game_thread(scheduler, move || {
                    let key = enshwick::content::types::ITEM_REGISTRY_RESOURCE
                        .key(enshwick::content::items::ITEM_REGISTRY)
                        .map_err(|e| e.to_string())?;
                    let host = enshwick::host().ok_or("game host unavailable")?;
                    let bytes = host
                        .game_resource(key)
                        .ok_or("vanilla item registry unavailable")?;
                    crate::item_lookup::parse(layout, &bytes, &ids)
                })?
            }
            "capabilities" => Ok(json!({
                "mod_version":env!("CARGO_PKG_VERSION"),
                "native_interaction_target":interaction_targets.is_some(),
                "native_reposition":self.reposition.is_some(),
                "native_clock_advance":self.clock_control.is_some(),
                "native_clock_advance_live_verified":false,
                "native_reposition_live_verified":false,
                "enshwick":"1.0.0-alpha.17",
                "compiled_framework_revision":"6a151a9550ed7a6ade99c23fa817a91e25b71bc9",
                "compiled_framework_abi":format!("{}.{}",enshwick::abi::ABI_MAJOR,enshwick::abi::ABI_MINOR),
                "game_build":null,
                "game_build_status":"not_observed",
                "offline_tools":offline_tools().iter().map(|tool|tool.name).collect::<Vec<_>>(),
                "new_inspection_tools_live_verified":false,
                "evidence":"handles resolved at mod initialization; live reads not implied",
                "inventory_read":inventories.is_some(),
                "item_info":inventories.is_some(),
                "item_lookup":item_layout.is_some(),
                "building_survey":true,
                "construction_available":self.construction.is_some(),
                "typed_network_health":network_health.is_some(),
                "typed_factory_station":factory_station.is_some(),
                "resolved_component_names":self.components.len(),
                "limits":{"health_radius_m":inspection::HEALTH_RADIUS,"health_entities":inspection::HEALTH_LIMIT,"inventory_page_slots":inspection::SLOT_LIMIT,"inventory_offset":inspection::SLOT_OFFSET_MAX,"survey_cells":policy::building_survey::MAX_CELLS,"survey_side_m":policy::building_survey::MAX_SIDE_M,"wait_seconds":GAME_WAIT.as_secs()},
                "limitations":["output limits do not cap enshwick spatial discovery or virtual-inventory traversal","a response timeout does not cancel a queued read","entity IDs and slot references are session-only","inventory pages are independent snapshots"]
            })),
            "inventory_read" => {
                let request = inspection::inventory(a)?;
                let inventories = inventories.ok_or("inventory API unavailable on this build")?;
                on_game_thread(scheduler, move || {
                    let entity = match request.target {
                        inspection::InventoryTarget::Entity(id) => enshwick::Entity(id),
                        inspection::InventoryTarget::Player(who) => {
                            let matches: Vec<_> = world
                                .players()
                                .into_iter()
                                .filter(|p| p.key.0.to_string() == who || p.is_named(&who))
                                .collect();
                            match matches.as_slice() {
                                [player] => player.entity,
                                [] => {
                                    return Err(
                                        "no connected player matches this selector".to_owned()
                                    );
                                }
                                _ => {
                                    return Err(
                                        "player selector is ambiguous; use the account key"
                                            .to_owned(),
                                    );
                                }
                            }
                        }
                    };
                    let reader = inventories.read();
                    let slots = reader.slots(entity);
                    let total = slots.len();
                    let item_counts = request
                        .item_ids
                        .as_ref()
                        .map(|ids| {
                            inspection::inventory_item_counts(
                                ids,
                                slots
                                    .iter()
                                    .map(|slot| (slot.at.0, slot.item.0, slot.count)),
                            )
                        })
                        .transpose()?;
                    let listed: Vec<Value> = slots.iter().skip(request.offset).take(request.limit).map(|slot| {
                        let info = (!slot.is_empty()).then(|| inventories.item_info(slot.item)).flatten();
                        let durability = slot.pide.and_then(|p| reader.durability(p)).map(|bar| json!({"current":bar.current,"max":bar.max}));
                        let meter = slot.pide.and_then(|p| reader.meter(p)).map(|bar| json!({"current":bar.current,"max":bar.max}));
                        json!({"slot_ref":slot.at.0.to_string(),"inventory_entity":slot.at.inventory().0,"index":slot.at.index(),"category":slot.category.0,"item_id":slot.item.0,"count":slot.count,"empty":slot.is_empty(),"item_entity":slot.pide.map(|p|p.0),"item_info":info.map(item_metadata),"durability":durability,"meter":meter})
                    }).collect();
                    let end = request.offset + listed.len();
                    let mut result = json!({"world_key":world.key(),"entity":entity.0,"identity_scope":"current session","total_slots":total,"offset":request.offset,"next_offset":(end < total).then_some(end),"slots":listed,"note":"slots follow virtual links; pages are not an atomic inventory snapshot; no slots may mean absent or uninitialized inventory"});
                    if let Some(counts) = item_counts {
                        result["item_counts"] = counts;
                    }
                    Ok(result)
                })?
            }
            "item_info" => {
                let name = inspection::item(a)?;
                let inventories = inventories.ok_or("inventory API unavailable on this build")?;
                on_game_thread(scheduler, move || {
                    let guid =
                        enshwick::content::vanilla::item(&name).map_err(|e| e.to_string())?;
                    let item = enshwick::ItemId::from_guid(guid);
                    let mut result = item_inspection(inventories.item_info(item));
                    result["debug_name"] = json!(name);
                    result["guid"] = json!(guid.to_string());
                    result["world_key"] = json!(world.key());
                    Ok::<_, String>(result)
                })?
            }
            "entity_health" => {
                let (point, radius, limit) = inspection::health(a)?;
                let health = network_health
                    .ok_or("typed NetworkHealth unavailable or incompatible on this build")?;
                on_game_thread(scheduler, move || {
                    let found = world.near(point, radius, world.transform_type());
                    let listed: Vec<Value> = found.iter().take(limit).map(|e| {
                        let mut result = json!({"entity":e.entity.0,"position":e.position,"distance":e.distance(point)});
                        match health.read(world,e.entity) {
                            Ok(Some(value)) => {result["state"]=json!("present"); result["health"]=json!(value.health); result["health_max"]=json!(value.health_max);},
                            Ok(None) => {result["state"]=json!("absent");},
                            Err(error) => {result["state"]=json!("read_error"); result["error"]=json!(error.to_string());},
                        }
                        result
                    }).collect();
                    json!({"world_key":world.key(),"component":"keen::ecs::NetworkHealth","scope":"nearest placed entities","total_candidates":found.len(),"truncated":found.len()>limit,"entities":listed})
                })
            }
            "status" => {
                let hosting = enshwick::host().is_some_and(|h| h.hosting());
                if !hosting {
                    return Ok(
                        json!({"hosting": false, "clock":crate::world_clock::encode(None), "note": "no world runs in this process; load or host a world"}),
                    );
                }
                on_game_thread(scheduler, move || {
                    let save = world.save_info().map(|s| json!({"size": s.size, "saved_at_unix": s.saved_at.as_secs(), "backups": s.backups}));
                    json!({"hosting": true, "world_key": world.key(), "save": save, "players": world.players().len(), "clock":crate::world_clock::encode(clock.now())})
                })
            }
            "players" => on_game_thread(scheduler, move || {
                Value::Array(
                    world
                        .players()
                        .into_iter()
                        .map(|p| json!({"name": p.name, "key": p.key.0.to_string(), "slot": p.slot, "admin": p.is_admin(), "position": world.position(p.entity), "vitals": crate::player_vitals::encode(world.vitals(p.entity))}))
                        .collect(),
                )
            }),
            "voxel_sample" => {
                let p = args::point(a, "x", "y", "z");
                if !bounds::in_world(p) {
                    return Err(format!(
                        "point {p:?} is outside the world (0..={} m across, 0..={} m up)",
                        bounds::EDGE,
                        bounds::HEIGHT
                    ));
                }
                let at = VoxelPos::from_metres(p).ok_or("point is outside the voxel grid")?;
                on_game_thread(scheduler, move || match voxels.sample(at) {
                    Ok(v) => Ok(
                        json!({"material": format!("{:?}", v.material()), "fill": v.fill(), "voxel": format!("{at:?}")}),
                    ),
                    Err(e) => Err(format!("voxel read failed: {e}")),
                })?
            }
            "props_in" => {
                let p1 = args::point(a, "x1", "y1", "z1");
                let p2 = args::point(a, "x2", "y2", "z2");
                if !bounds::small_box(p1, p2, PROPS_BOX) {
                    return Err(format!(
                        "the box must lie in the world with no side over {PROPS_BOX} m"
                    ));
                }
                on_game_thread(scheduler, move || match world.props_in(p1, p2) {
                    Ok(props) => {
                        let total = props.len();
                        let listed = crate::prop_observation::rows(&props, |entity| {
                            (
                                world.template(entity).map(|guid| guid.0),
                                world.has_label(entity),
                            )
                        });
                        Ok(json!({"total": total, "props": listed,
                            "truncated":total > policy::inspection::PROP_LIMIT,
                            "recovery":"label presence only; label text and other held state are not observed; no safe-removal or undo claim"}))
                    }
                    Err(e) => Err(format!("prop read failed: {e}")),
                })?
            }
            "entities_near" => {
                let p = args::point(a, "x", "y", "z");
                let radius = args::num(a, "radius", 8.0);
                if !bounds::in_world(p) || radius <= 0.0 {
                    return Err("the point must lie in the world and the radius be above 0".into());
                }
                let kind = a
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("placed")
                    .to_owned();
                let ty = match kind.as_str() {
                    "placed" => Some(world.transform_type()),
                    "used_item" => {
                        Some(used_item.ok_or("UsedItem is not reflected on this build")?)
                    }
                    "static_used_item" => Some(
                        static_used_item.ok_or("StaticUsedItem is not reflected on this build")?,
                    ),
                    _ => None, // containers
                };
                on_game_thread(scheduler, move || {
                    let found: Vec<Placed> = match ty {
                        Some(ty) => world.near(p, radius, ty),
                        None => {
                            let mut c: Vec<Placed> = world
                                .containers(ContainerKinds::PLACED)
                                .into_iter()
                                .filter(|c| c.distance(p) <= radius)
                                .collect();
                            c.sort_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
                            c
                        }
                    };
                    let has =
                        |e, ty: Option<ComponentType>| ty.map(|t| world.component(e, t).is_some());
                    let listed: Vec<Value> = found
                        .iter()
                        .take(NEAR_MAX)
                        .map(|e| {
                            json!({
                                "entity": e.entity.0,
                                "position": e.position,
                                "distance": (e.distance(p) * 100.0).round() / 100.0,
                                "item": world.used_item(e.entity).map(|i| i.0),
                                "template": world.template(e.entity).map(|guid| hex(guid.0)),
                                "used_item": has(e.entity, used_item),
                                "static_used_item": has(e.entity, static_used_item),
                            })
                        })
                        .collect();
                    json!({"kind": kind, "total": found.len(), "entities": listed})
                })
            }
            "entity_components" => {
                let p = args::point(a, "x", "y", "z");
                let radius = args::num(a, "radius", 3.0);
                let limit = a.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
                if !bounds::in_world(p) || radius <= 0.0 {
                    return Err("the point must lie in the world and the radius be above 0".into());
                }
                if self.components.is_empty() {
                    return Err("no component names loaded: put components.txt (scripts/component_names.py) in the mod's folder and restart".into());
                }
                let components = Arc::clone(&self.components);
                on_game_thread(scheduler, move || {
                    let found = world.near(p, radius, world.transform_type());
                    let described: Vec<Value> = found
                        .iter()
                        .take(limit)
                        .map(|e| {
                            let names: Vec<&str> = components
                                .iter()
                                .filter(|(_, ty)| world.component(e.entity, *ty).is_some())
                                .map(|(name, _)| name.strip_prefix("keen::ecs::").unwrap_or(name))
                                .collect();
                            json!({
                                "entity": e.entity.0,
                                "position": e.position,
                                "distance": (e.distance(p) * 100.0).round() / 100.0,
                                "template": world.template(e.entity).map(|guid| hex(guid.0)),
                                "components": names,
                            })
                        })
                        .collect();
                    json!({"total": found.len(), "known_types": components.len(), "entities": described})
                })
            }
            "give_item" => self
                .grants
                .as_ref()
                .ok_or("grant lifecycle unavailable")?
                .call(self.game, a),
            _ => Err(format!("unknown tool: {name}")),
        }
    }
}

impl GameTools {
    /// Syncs one bounded audit record, admitting optional outcome headroom.
    pub(crate) fn journal(
        dir: Option<&std::path::Path>,
        tool: &str,
        result: &Value,
        reserve: usize,
    ) -> Result<(), String> {
        let dir = dir.ok_or("no mod folder for write journal")?;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        let line = json!({"unix": secs, "tool": tool, "result": result}).to_string() + "\n";
        write_gate::journal_budget(0, line.len(), reserve)?;
        crate::journal::append(dir, line.as_bytes(), reserve, None)
    }
}

fn item_metadata(info: enshwick::inventory::ItemInfo) -> Value {
    json!({"item_id":info.id.0,"max_stack_size":info.max_stack_size,"category":info.category,"flags":info.flags,"has_item_entity":info.has_pide(),"is_container":info.is_container(),"is_blueprint":info.is_blueprint()})
}

fn item_inspection(info: Option<enshwick::inventory::ItemInfo>) -> Value {
    let (state, template) = match info {
        Some(info) => match info.placed_entity_template {
            Some(template) => ("observed", Some(hex(template.0))),
            None => ("known_no_template", None),
        },
        None => ("unavailable", None),
    };
    json!({"metadata":info.map(item_metadata),"placement":{"state":state,"template_guid":template,
        "source":"enshwick_item_info","guid_encoding":"lowercase hex; 16 GUID bytes in file order",
        "limitations":"mapping only; no placement eligibility, footprint, build-rights or spawn authorization claim"}})
}

fn offline_tools() -> [Tool; 3] {
    [
        crate::knowledge::tool(),
        crate::recipe_plan::tool(),
        crate::building_plan::tool(),
    ]
}

/// Dispatch before accessing any game handles, so research planning works
/// even when no world is hosted.
fn offline_call(name: &str, args: &Map<String, Value>) -> Option<Result<Value, String>> {
    match name {
        "knowledge_search" => Some(crate::knowledge::search(args)),
        "recipe_plan" => Some(crate::recipe_plan::call(args)),
        "building_plan" => Some(crate::building_plan::call(args)),
        _ => None,
    }
}

/// A GUID's 16 bytes in file order as lowercase hex.
fn hex(bytes: [u8; 16]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Runs `read` on the game thread and waits up to [`GAME_WAIT`] for it.
fn on_game_thread<T: Send + 'static>(
    scheduler: Scheduler,
    read: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = mpsc::sync_channel(1);
    scheduler.next_tick(move || {
        let _gone = tx.send(read()); // allow: the caller gave up waiting
    });
    rx.recv_timeout(GAME_WAIT).map_err(|_| {
        format!(
            "the game thread did not answer within {} s (is a world loaded?)",
            GAME_WAIT.as_secs()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_journal_preserves_construction_recovery_allowance() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .canonicalize()
            .unwrap();
        let dir = base.join(format!(
            "generic-journal-reservation-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        crate::journal::reserve(&dir, "a", 100, 0).unwrap();
        let file = fs::File::create(dir.join("journal.jsonl")).unwrap();
        let size = policy::construction::JOURNAL_MAX_BYTES as u64 - 100;
        file.set_len(size).unwrap();
        drop(file);
        assert!(GameTools::journal(Some(&dir), "test", &json!({}), 0).is_err());
        assert_eq!(fs::metadata(dir.join("journal.jsonl")).unwrap().len(), size);
        crate::journal::reset(&dir).unwrap();
        let resolved = dir.canonicalize().unwrap();
        assert!(resolved.starts_with(&base) && resolved != base);
        fs::remove_dir_all(resolved).unwrap();
    }
    #[test]
    fn give_gate_refuses_directory_and_future_flag_but_allows_valid_preview() {
        use std::{
            fs,
            time::{Duration, SystemTime, UNIX_EPOCH},
        };
        let dir = std::env::temp_dir().join(format!(
            "mcp-give-gate-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let flag = dir.join(write_gate::FLAG);
        fs::create_dir(&flag).unwrap();
        assert!(give_gate(false, Some(&dir), Some("server-write")).is_err());
        fs::remove_dir(&flag).unwrap();
        let file = fs::File::create(&flag).unwrap();
        file.set_times(
            fs::FileTimes::new().set_modified(SystemTime::now() + Duration::from_secs(60)),
        )
        .unwrap();
        assert!(give_gate(false, Some(&dir), Some("server-write")).is_err());
        assert!(give_gate(true, Some(&dir), None).is_ok());
        file.set_times(fs::FileTimes::new().set_modified(SystemTime::now()))
            .unwrap();
        assert!(give_gate(false, Some(&dir), Some("server-write")).is_ok());
        assert!(give_gate(false, Some(&dir), None).is_err());
        drop(file);
        fs::remove_file(&flag).unwrap();
        let dispatched = std::cell::Cell::new(false);
        let result = give_dispatch(Some(&dir), Some("server-write"), || dispatched.set(true));
        assert!(result.is_err(), "revoked flag admitted native dispatch");
        assert!(!dispatched.get());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn item_inspection_distinguishes_mapping_absence_and_unavailable_metadata() {
        let info = enshwick::inventory::ItemInfo {
            id: enshwick::ItemId::new(7),
            max_stack_size: 24,
            flags: 0,
            category: 3,
            placed_entity_template: Some(enshwick::content::Guid([
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255,
            ])),
        };
        let expected_metadata = json!({"item_id":7,"max_stack_size":24,"category":3,"flags":0,
            "has_item_entity":false,"is_container":false,"is_blueprint":false});
        let mapped = item_inspection(Some(info));
        assert_eq!(mapped["metadata"], expected_metadata);
        assert_eq!(mapped["placement"]["state"], "observed");
        assert_eq!(
            mapped["placement"]["template_guid"],
            "000102030405060708090a0b0c0d0eff"
        );
        let absent = item_inspection(Some(enshwick::inventory::ItemInfo {
            placed_entity_template: None,
            ..info
        }));
        assert_eq!(absent["metadata"], expected_metadata);
        assert_eq!(absent["placement"]["state"], "known_no_template");
        assert_eq!(absent["placement"]["template_guid"], Value::Null);
        let unavailable = item_inspection(None);
        assert_eq!(unavailable["metadata"], Value::Null);
        assert_eq!(unavailable["placement"]["state"], "unavailable");
        assert_eq!(unavailable["placement"]["template_guid"], Value::Null);
        for result in [mapped, absent, unavailable] {
            assert_eq!(result["placement"]["source"], "enshwick_item_info");
            assert!(
                result["placement"]["limitations"]
                    .as_str()
                    .unwrap()
                    .contains("no placement eligibility")
            );
        }
    }

    #[test]
    fn registered_offline_tools_dispatch_without_game_handles() {
        let examples = [
            ("knowledge_search", json!({"query":"inventory"})),
            (
                "recipe_plan",
                json!({"catalog_label":"test only", "recipes":[], "stock":[], "goals":[{"item":"wood","count":1}], "stations":[], "unlocks":[]}),
            ),
            (
                "building_plan",
                json!({"kind":"floor","x":1,"y":1,"z":1,"width":1,"height":0.5,"depth":1,"material":"test:unvalidated"}),
            ),
        ];
        let tools = offline_tools();
        assert_eq!(tools.len(), examples.len());
        for (name, value) in examples {
            let args = value.as_object().unwrap();
            let tool = tools.iter().find(|tool| tool.name == name).unwrap();
            args::check(&tool.schema, args).unwrap();
            let result = offline_call(name, args).unwrap();
            assert!(result.is_ok(), "{name}: {result:?}");
        }
        assert!(offline_call("not_a_tool", &Map::new()).is_none());
    }

    #[test]
    fn offline_dispatch_rejects_unknown_arguments() {
        for tool in offline_tools() {
            let args = json!({"unrecognized":true});
            assert!(
                offline_call(tool.name, args.as_object().unwrap())
                    .unwrap()
                    .is_err()
            );
        }
    }
}
