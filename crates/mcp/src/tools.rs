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
/// The most props one answer lists.
const PROPS_MAX: usize = 500;
/// The largest `entities_near` radius, in metres.
const NEAR_RADIUS: f64 = 32.0;
/// The most entities one `entities_near` answer lists.
const NEAR_MAX: usize = 200;
/// The most entities one `entity_components` answer describes.
const DESCRIBE_MAX: usize = 20;

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
    pub(crate) world: World,
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
}

#[derive(Debug)]
pub(crate) struct GameTools {
    game: Game,
    components: Components,
    /// The mod's folder: the write flag and the journal live there.
    mod_dir: Option<std::path::PathBuf>,
    list: Vec<Tool>,
}

impl GameTools {
    pub(crate) fn new(
        game: Game,
        components: Components,
        mod_dir: Option<std::path::PathBuf>,
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
            crate::item_lookup::tool(),
            Tool {
                name: "capabilities",
                description: "Resolved inspection handles and limits in this mod process. Availability is initialization evidence, not proof of successful live reads. Underlying nearby-entity discovery and inventory-link traversal are not capped by output limits.",
                schema: json!({"type":"object"}),
            },
            Tool {
                name: "inventory_read",
                description: "Read a page of slots from exactly one connected player (name/account key) or session-only entity ID. Empty slots included. Follows virtual inventory links through enshwick; paging caps output and per-slot metadata, not upstream traversal. Offsets are not stable across requests. Empty output cannot distinguish an absent/uninitialized inventory. Default 64, maximum 128 slots.",
                schema: json!({"type":"object","properties": {
                    "player":{"type":"string"},
                    "entity":{"type":"integer","minimum":1,"maximum":u32::MAX},
                    "offset":{"type":"integer","minimum":0,"maximum":inspection::SLOT_OFFSET_MAX},
                    "limit":{"type":"integer","minimum":1,"maximum":inspection::SLOT_LIMIT}
                }}),
            },
            Tool {
                name: "item_info",
                description: "Resolve one vanilla item debug name and read static stack-size, category and capability flags from the running world. Does not infer recipes, rarity, unlocks, or crafting eligibility.",
                schema: json!({"type":"object","properties":{"item":{"type":"string"}},"required":["item"]}),
            },
            Tool {
                name: "entity_health",
                description: "Typed NetworkHealth for nearest placed entities within radius (default 3 m, maximum 16), at most limit (default 10, maximum 20). Reports absent components and decode errors separately; does not scan all combat actors. Output is capped, underlying spatial discovery is not.",
                schema: health,
            },
            Tool {
                name: "status",
                description: "Whether a world runs here, its key, its save, and how many players are in it.",
                schema: json!({"type": "object"}),
            },
            Tool {
                name: "players",
                description: "Every connected player: name (client-supplied), account key, slot, admin, position in metres (y up).",
                schema: json!({"type": "object"}),
            },
            Tool {
                name: "voxel_sample",
                description: "The voxel at a point in metres: material (air, terrain id or building id) and fill 0-255.",
                schema: sample,
            },
            Tool {
                name: "props_in",
                description: "Placed props in a box (each side at most 64 m): entity (valid this session only), item id, position, orientation quaternion x,y,z,w. At most 500.",
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
                description: "WRITE. Gives a connected player `count` (default 1, at most 100) of a vanilla item by its debug name (such as Craftingstation_Workbench), through the game's inventory transaction. An unknown name answers with up to five similar names. `dry: true` only resolves the player and the item. A real give needs the ALLOW_WRITES flag file (under 30 min old) in the mod's folder and confirm: \"server-write\", and is journaled.",
                schema: json!({"type": "object", "properties": {
                    "player": {"type": "string"},
                    "item": {"type": "string"},
                    "count": {"type": "integer", "maximum": 100},
                    "dry": {"type": "boolean"},
                    "confirm": {"type": "string"},
                }, "required": ["player", "item"]}),
            },
        ];
        list.extend(offline_tools());
        Self {
            game,
            components,
            mod_dir,
            list,
        }
    }
}

impl Tools for GameTools {
    fn list(&self) -> &[Tool] {
        &self.list
    }

    fn call(&self, name: &str, a: &Map<String, Value>) -> Result<Value, String> {
        if let Some(result) = offline_call(name, a) {
            return result;
        }
        let Game {
            world,
            voxels,
            scheduler,
            used_item,
            static_used_item,
            inventories,
            item_layout,
            network_health,
        } = self.game;
        match name {
            "item_lookup" => {
                let ids = policy::item_lookup::check(a)?;
                let layout = item_layout.ok_or("vanilla item registry layout unavailable")?;
                on_game_thread(scheduler, move || {
                    let key = enshwick::content::items::ITEM_REGISTRY
                        .key(enshwick::content::types::ITEM_REGISTRY_RESOURCE)
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
                "enshwick":"1.0.0-alpha.14",
                "game_build":null,
                "game_build_status":"not_observed",
                "offline_tools":offline_tools().iter().map(|tool|tool.name).collect::<Vec<_>>(),
                "new_inspection_tools_live_verified":false,
                "evidence":"handles resolved at mod initialization; live reads not implied",
                "inventory_read":inventories.is_some(),
                "item_info":inventories.is_some(),
                "item_lookup":item_layout.is_some(),
                "typed_network_health":network_health.is_some(),
                "resolved_component_names":self.components.len(),
                "limits":{"health_radius_m":inspection::HEALTH_RADIUS,"health_entities":inspection::HEALTH_LIMIT,"inventory_page_slots":inspection::SLOT_LIMIT,"inventory_offset":inspection::SLOT_OFFSET_MAX,"wait_seconds":GAME_WAIT.as_secs()},
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
                    let listed: Vec<Value> = slots.iter().skip(request.offset).take(request.limit).map(|slot| {
                        let info = (!slot.is_empty()).then(|| inventories.item_info(slot.item)).flatten();
                        let durability = slot.pide.and_then(|p| reader.durability(p)).map(|bar| json!({"current":bar.current,"max":bar.max}));
                        let meter = slot.pide.and_then(|p| reader.meter(p)).map(|bar| json!({"current":bar.current,"max":bar.max}));
                        json!({"slot_ref":slot.at.0.to_string(),"inventory_entity":slot.at.inventory().0,"index":slot.at.index(),"category":slot.category.0,"item_id":slot.item.0,"count":slot.count,"empty":slot.is_empty(),"item_entity":slot.pide.map(|p|p.0),"item_info":info.map(item_metadata),"durability":durability,"meter":meter})
                    }).collect();
                    let end = request.offset + listed.len();
                    Ok(
                        json!({"world_key":world.key(),"entity":entity.0,"identity_scope":"current session","total_slots":total,"offset":request.offset,"next_offset":(end < total).then_some(end),"slots":listed,"note":"slots follow virtual links; pages are not an atomic inventory snapshot; no slots may mean absent or uninitialized inventory"}),
                    )
                })?
            }
            "item_info" => {
                let name = inspection::item(a)?;
                let inventories = inventories.ok_or("inventory API unavailable on this build")?;
                on_game_thread(scheduler, move || {
                    let guid =
                        enshwick::content::vanilla::item(&name).map_err(|e| e.to_string())?;
                    let item = enshwick::ItemId::from_guid(guid);
                    let info = inventories
                        .item_info(item)
                        .ok_or("item metadata unavailable; ensure a world is loaded")?;
                    Ok::<_, String>(
                        json!({"debug_name":name,"guid":guid.to_string(),"world_key":world.key(),"metadata":item_metadata(info)}),
                    )
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
                        json!({"hosting": false, "note": "no world runs in this process; load or host a world"}),
                    );
                }
                on_game_thread(scheduler, move || {
                    let save = world.save_info().map(|s| json!({"size": s.size, "saved_at_unix": s.saved_at.as_secs(), "backups": s.backups}));
                    json!({"hosting": true, "world_key": world.key(), "save": save, "players": world.players().len()})
                })
            }
            "players" => on_game_thread(scheduler, move || {
                Value::Array(
                    world
                        .players()
                        .into_iter()
                        .map(|p| json!({"name": p.name, "key": p.key.0.to_string(), "slot": p.slot, "admin": p.is_admin(), "position": world.position(p.entity)}))
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
                        let listed: Vec<Value> = props
                            .iter()
                            .take(PROPS_MAX)
                            .map(|p| json!({"entity": p.entity.0, "item": p.item.0, "position": p.transform.position, "orientation": p.transform.orientation}))
                            .collect();
                        Ok(json!({"total": total, "props": listed}))
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
                                "template": world.template(e.entity).map(hex),
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
                                "template": world.template(e.entity).map(hex),
                                "components": names,
                            })
                        })
                        .collect();
                    json!({"total": found.len(), "known_types": components.len(), "entities": described})
                })
            }
            "give_item" => {
                let dry = a.get("dry").and_then(Value::as_bool).unwrap_or(false);
                let flag_age = self.mod_dir.as_ref().and_then(|d| {
                    let modified = std::fs::metadata(d.join(write_gate::FLAG))
                        .ok()?
                        .modified()
                        .ok()?;
                    Some(modified.elapsed().unwrap_or_default())
                });
                write_gate::check(
                    dry,
                    true,
                    flag_age,
                    a.get("confirm").and_then(Value::as_str),
                )?;
                let inventories =
                    inventories.ok_or("this build lacks the inventory transactions")?;
                let who = a["player"].as_str().unwrap_or("").to_owned();
                let name = a["item"].as_str().unwrap_or("").to_owned();
                let count = a.get("count").and_then(Value::as_u64).unwrap_or(1);
                let count = u32::try_from(count)
                    .ok()
                    .filter(|c| *c >= 1)
                    .ok_or("count must be 1 to 100")?;
                let (entity, player, item, guid) = on_game_thread(scheduler, move || {
                    // The item first, so a dry run finds names with no player on.
                    let guid =
                        enshwick::content::vanilla::item(&name).map_err(|e| e.to_string())?;
                    let player = world
                        .players()
                        .into_iter()
                        .find(|p| p.key.0.to_string() == who || p.is_named(&who))
                        .ok_or_else(|| {
                            format!("no connected player with the key or name {who:?}")
                        })?;
                    Ok::<_, String>((
                        player.entity,
                        player.name,
                        enshwick::ItemId::from_guid(guid),
                        guid.to_string(),
                    ))
                })??;
                let plan = json!({"player": player, "item": name_of(a), "item_id": item.0, "guid": guid, "count": count});
                if dry {
                    return Ok(json!({"dry": true, "would_give": plan}));
                }
                let (tx, rx) = mpsc::sync_channel(1);
                inventories.give(entity, item, count, move |outcome| {
                    let _gone = tx.send((outcome.is_committed(), format!("{:?}", outcome.done))); // allow: the caller gave up waiting
                });
                let (committed, done) = rx.recv_timeout(GAME_WAIT).map_err(|_| {
                    "the inventory transaction did not answer within 5 s".to_owned()
                })?;
                let result = json!({"given": committed, "outcome": done, "plan": plan});
                self.journal("give_item", &result);
                Ok(result)
            }
            _ => Err(format!("unknown tool: {name}")),
        }
    }
}

impl GameTools {
    /// Appends one line to `journal.jsonl` in the mod's folder; a failure is
    /// logged, as the write itself already happened.
    fn journal(&self, tool: &str, result: &Value) {
        let Some(dir) = &self.mod_dir else {
            enshwick::warn!("no mod folder: {tool} not journaled");
            return;
        };
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        let line = json!({"unix": secs, "tool": tool, "result": result}).to_string() + "\n";
        let written = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("journal.jsonl"))
            .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
        if let Err(e) = written {
            enshwick::warn!("{tool} not journaled: {e}");
        }
    }
}

fn name_of(a: &Map<String, Value>) -> &str {
    a.get("item").and_then(Value::as_str).unwrap_or("")
}

fn item_metadata(info: enshwick::inventory::ItemInfo) -> Value {
    json!({"item_id":info.id.0,"max_stack_size":info.max_stack_size,"category":info.category,"flags":info.flags,"has_item_entity":info.has_pide(),"is_container":info.is_container(),"is_blueprint":info.is_blueprint()})
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
