//! One server teleport request, never an arrival, walking or camera controller.
//! Call only through the server game-thread scheduler. Replay never dispatches.
use crate::{construction::Construction, tools::Game};
use enshwick::{
    Load, Off,
    effects::{Effects, Teleport},
};
use policy::{
    reposition::{self, ActorContext, ActorFacts, Spec},
    write_gate,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct Reposition {
    dir: PathBuf,
    effects: Effects,
    construction: Construction,
    generation: Arc<AtomicU64>,
    operations: Arc<Mutex<BTreeMap<String, Operation>>>,
}
#[derive(Debug)]
struct Operation {
    request: Value,
    session: String,
    outcome: Value,
}

fn bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    reposition::check_record_size(bytes.len())?;
    Ok(bytes)
}
fn journal_size(dir: &Path) -> Result<u64, String> {
    match fs::metadata(dir.join("journal.jsonl")) {
        Ok(meta) if meta.is_file() => Ok(meta.len()),
        Ok(_) => Err("reposition journal must be a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(format!("reposition journal unavailable: {e}")),
    }
}
fn append(dir: &Path, value: &Value) -> Result<(), String> {
    crate::journal::append(dir, &bytes(value)?, 0, None)
}
fn reserve(dir: &Path, operation: &str, intent: &Value) -> Result<(), String> {
    let bytes = bytes(intent)?;
    let folder = dir.join("reposition-operations");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(folder.join(format!("{operation}.json")))
        .map_err(|e| {
            format!("reposition identity reserved or unavailable; never retry dispatch: {e}")
        })?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("reposition intent durability failed: {e}"))
}
fn disk_count(dir: &Path) -> Result<usize, String> {
    match fs::read_dir(dir.join("reposition-operations")) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries {
                entry.map_err(|e| e.to_string())?;
                count += 1;
                reposition::check_capacity(count)?;
            }
            Ok(count)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e.to_string()),
    }
}
// The once marker already exists. Recording any result must never remove it.
fn record_outcome(
    dir: &Path,
    operation: &str,
    op: &mut Operation,
    result: Result<(), enshwick::WorldError>,
) -> Value {
    op.outcome["native_request_accepted"] = json!(result.is_ok());
    if let Err(error) = result {
        op.outcome["native_error"] = json!(error.to_string());
    }
    if let Err(error) = append(
        dir,
        &json!({"kind":"reposition_request_outcome","operation":operation,"outcome":op.outcome}),
    ) {
        op.outcome["journal_error"] = json!(error);
    }
    op.outcome.clone()
}

impl Reposition {
    pub(crate) fn new(load: &mut Load<'_>, construction: Construction) -> Result<Self, Off> {
        let generation = Arc::new(AtomicU64::new(0));
        let joined = generation.clone();
        load.on::<enshwick::events::PlayerJoined>(move |_| {
            joined.fetch_add(1, Ordering::SeqCst);
        })?;
        let left = generation.clone();
        load.on::<enshwick::events::PlayerLeft>(move |_| {
            left.fetch_add(1, Ordering::SeqCst);
        })?;
        let died = generation.clone();
        load.on::<enshwick::events::PlayerDied>(move |_| {
            died.fetch_add(1, Ordering::SeqCst);
        })?;
        Ok(Self {
            dir: load.mod_dir()?,
            effects: load.effects(),
            construction,
            generation,
            operations: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }
    fn gate(&self, spec: &Spec) -> Result<(), String> {
        write_gate::check(
            spec.dry,
            true,
            write_gate::flag_age(&self.dir.join(write_gate::FLAG)),
            spec.confirm.as_deref(),
        )
    }
    fn context(&self, game: Game, spec: &Spec) -> Result<ActorContext, String> {
        let session = self.construction.active_session()?;
        let players = game.world.players();
        let actors: Vec<_> = players
            .iter()
            .filter(|p| p.is_named(&spec.player) || p.key.0.to_string() == spec.player)
            .collect();
        let actor = actors.first().copied();
        let health = actor
            .and_then(|p| game.world.vitals(p.entity))
            .map(|v| v.health.current);
        reposition::check_actor(ActorFacts {
            connected_players: players.len(),
            matching_actors: actors.len(),
            health,
        })?;
        let actor = actor.ok_or("no unique current server player")?;
        let position = game
            .world
            .position(actor.entity)
            .ok_or("player position unavailable")?;
        reposition::check_distance(position, spec.target)?;
        Ok(ActorContext {
            session,
            actor_generation: self.generation.load(Ordering::SeqCst),
            entity: actor.entity.0,
            account: actor.key.0,
            position,
        })
    }
    pub(crate) fn run(&self, game: Game, spec: Spec) -> Result<Value, String> {
        self.gate(&spec)?;
        let session = self.construction.active_session()?;
        let request = reposition::canonical_request(&spec);
        let mut ops = self
            .operations
            .lock()
            .map_err(|_| "reposition state poisoned")?;
        if let Some(op) = ops.get(&spec.operation) {
            if op.session != session {
                return Err("stale reposition session; never redispatch".into());
            }
            reposition::same_request(&op.request, &request)?;
            return Ok(op.outcome.clone());
        }
        let path = self
            .dir
            .join("reposition-operations")
            .join(format!("{}.json", spec.operation));
        if path.exists() {
            return Err("stale/unknown durable reposition identity; never redispatch".into());
        }
        reposition::check_capacity(ops.len())?;
        reposition::check_capacity(disk_count(&self.dir)?)?;
        reposition::check_journal_headroom(journal_size(&self.dir)?)?;
        crate::journal::check(&self.dir, 2 * reposition::MAX_RECORD_BYTES)?;
        let context = self.context(game, &spec)?;
        if spec.dry {
            return Ok(
                json!({"state":"dry","operation":spec.operation,"preview":reposition::preview_token(&context,spec.target)?,"current_position":context.position,"requested_position":spec.target,"actor_generation":context.actor_generation,"native_search_best_spawn":true,"orientation_preserved":true,"world_mutated":false,"collision_verified":false,"arrival_verified":false,"camera_control":false,"automatic_retry":false}),
            );
        }
        reposition::check_preview(spec.preview.as_deref().unwrap_or(""), &context, spec.target)?;
        let intent = json!({"kind":"reposition_intent","operation":spec.operation,"request":request,"session":context.session,"actor_account":context.account,"actor_entity_session_only":context.entity,"actor_generation":context.actor_generation,"before_position":context.position,"requested_position":spec.target,"native_search_best_spawn":true,"orientation_preserved":true});
        let mut op = Operation {
            request,
            session: context.session.clone(),
            outcome: json!({"state":"not_issued_unknown","operation":spec.operation,"automatic_retry":false,"arrival_verified":false}),
        };
        // Remember even a partial reservation or failed append. No path retries native input.
        if let Err(error) =
            reserve(&self.dir, &spec.operation, &intent).and_then(|()| append(&self.dir, &intent))
        {
            op.outcome["journal_error"] = json!(error);
            ops.insert(spec.operation, op);
            return Err(
                "reposition intent failed; identity blocked, no native request issued".into(),
            );
        }
        let ready = self
            .gate(&spec)
            .and_then(|()| self.context(game, &spec))
            .and_then(|current| {
                reposition::check_preview(
                    spec.preview.as_deref().unwrap_or(""),
                    &current,
                    spec.target,
                )
            });
        if let Err(error) = ready {
            op.outcome["refusal"] = json!(error);
            ops.insert(spec.operation, op);
            return Err("reposition pre-dispatch checks changed; identity blocked, no native request issued".into());
        }
        op.outcome = json!({"state":"issued_unknown","operation":spec.operation,"requested_position":spec.target,"before_position":context.position,"native_search_best_spawn":true,"orientation_preserved":true,"arrival_verified":false,"collision_verified":false,"camera_control":false,"automatic_retry":false});
        ops.insert(spec.operation.clone(), op); // Once marker precedes the native call.
        let result = self.effects.teleport_player(
            enshwick::PlayerKey::new(context.account),
            spec.target,
            Teleport::new().search_best_spawn_position(),
        );
        let op = ops
            .get_mut(&spec.operation)
            .ok_or("reposition marker disappeared")?;
        Ok(record_outcome(&self.dir, &spec.operation, op, result))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn direct_append_preserves_other_construction_recovery() {
        let d = directory();
        crate::journal::reserve(&d, "retained", 100, 0).unwrap();
        let file = std::fs::File::create(d.join("journal.jsonl")).unwrap();
        file.set_len(policy::construction::JOURNAL_MAX_BYTES as u64 - 100)
            .unwrap();
        drop(file);
        assert!(append(&d, &json!({})).is_err());
        crate::journal::reset(&d).unwrap();
        fs::remove_dir_all(d).unwrap();
    }
    use super::*;
    static ADAPTER: Mutex<Option<(Reposition, Game)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let game = Game {
                world: load.world()?,
                native_knowledge: load.knowledge(),
                clock: load.clock(),
                game_settings: None,
                voxels: load.voxels(),
                scheduler: load.scheduler()?,
                used_item: None,
                static_used_item: None,
                inventories: None,
                item_layout: None,
                network_health: None,
                factory_station: None,
                interaction_targets: None,
            };
            let construction = Construction::new(load)?;
            *ADAPTER.lock().unwrap() = Some((Reposition::new(load, construction)?, game));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.reposition"
side = "server""#
    );
    fn directory() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "mcp-reposition-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }
    #[test]
    fn exclusive_intent_does_not_overwrite_and_journal_is_bounded() {
        let dir = directory();
        reserve(&dir, "once", &json!({"intent":1})).unwrap();
        assert!(reserve(&dir, "once", &json!({"intent":2})).is_err());
        assert_eq!(
            fs::read_to_string(dir.join("reposition-operations/once.json")).unwrap(),
            "{\"intent\":1}\n"
        );
        append(&dir, &json!({"kind":"intent"})).unwrap();
        assert_eq!(disk_count(&dir).unwrap(), 1);
        let file = OpenOptions::new()
            .write(true)
            .open(dir.join("journal.jsonl"))
            .unwrap();
        file.set_len(policy::construction::JOURNAL_MAX_BYTES as u64)
            .unwrap();
        assert!(append(&dir, &json!({"kind":"outcome"})).is_err());
        drop(file);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn oversized_intent_is_refused_before_a_tombstone_is_created() {
        let dir = directory();
        assert!(
            reserve(
                &dir,
                "once",
                &json!({"value":"x".repeat(reposition::MAX_RECORD_BYTES)})
            )
            .is_err()
        );
        assert!(!dir.join("reposition-operations").exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn native_error_is_journaled_as_unknown_without_erasing_marker() {
        let dir = directory();
        let request = json!({"operation":"refused","target":[1,2,3]});
        let mut op = Operation {
            request: request.clone(),
            session: "session".into(),
            outcome: json!({"state":"issued_unknown","arrival_verified":false,"automatic_retry":false}),
        };
        let result = record_outcome(&dir, "refused", &mut op, Err(enshwick::WorldError::Refused));
        assert_eq!(result["state"], "issued_unknown");
        assert_eq!(result["native_request_accepted"], false);
        assert!(result["native_error"].is_string());
        assert_eq!(result["arrival_verified"], false);
        assert_eq!(op.request, request);
        let record: Value =
            serde_json::from_slice(&fs::read(dir.join("journal.jsonl")).unwrap()).unwrap();
        assert_eq!(record["outcome"], result);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn fake_host_dispatches_once_and_never_retries_or_claims_arrival() {
        use enshwick::abi::{Header, PlayerInfo, Status};
        use enshwick::{Entity, content::EngineFacts};
        let dir = directory();
        let mut health = 10u32.to_le_bytes().to_vec();
        health.extend(8u16.to_le_bytes());
        health.extend([1, 8]);
        health.extend(
            [150i32, 0, 0, 0, 0, 0, 0, 200]
                .into_iter()
                .flat_map(i32::to_le_bytes),
        );
        let host = enshwick::test::FakeHost::new()
            .world()
            .mod_dir(dir.to_str().unwrap())
            .type_hash("attribute_set.health", 1)
            .type_hash("attribute_set.stamina", 2)
            .type_hash("attribute_set.mana", 3)
            .type_hash("attribute_set.level", 4)
            .type_hash("attribute_set.experience", 5)
            .constant("ecs.level_data_offset", 4)
            .constant("ecs.level_flags", 6)
            .constant("ecs.attribute_count", 7)
            .constant("attribute.health", 10)
            .constant("attribute.health_max_slot", 7)
            .constant("attribute.stamina", 20)
            .constant("attribute.stamina_max_slot", 2)
            .constant("attribute.mana", 30)
            .constant("attribute.mana_max_slot", 1)
            .constant("attribute.level", 40)
            .constant("attribute.experience", 50)
            .constant("attribute.experience_needed_slot", 2)
            .component(Entity(7), 1, &health)
            .template(Entity(7), enshwick::content::Guid([7; 16]));
        let transform = host
            .hash(enshwick::test::facts::CURRENT_TRANSFORM.id())
            .unwrap();
        let mut name = [0; 128];
        name[..5].copy_from_slice(b"actor");
        let standing: Vec<u8> = [100i64 << 32, 20i64 << 32, 100i64 << 32]
            .into_iter()
            .flat_map(i64::to_le_bytes)
            .collect();
        let loader = host
            .placed(transform, Entity(7), [100.0, 20.0, 100.0])
            .component(Entity(7), transform, &standing)
            .player(PlayerInfo {
                header: Header::of::<PlayerInfo>(),
                slot: 0,
                entity: Entity(7),
                key: enshwick::PlayerKey::new(9),
                permissions: 0,
                name_len: 5,
                name,
            })
            .install();
        // SAFETY: exports are generated by this test module's declared probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (adapter, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let mut spec = reposition::check(
            json!({"operation":"once","player":"actor","x":101,"y":20,"z":100})
                .as_object()
                .unwrap(),
        )
        .unwrap();
        let dry = adapter.run(game, spec.clone()).unwrap();
        assert_eq!(dry["state"], "dry");
        assert!(loader.teleports().is_empty());
        assert!(!dir.join("reposition-operations").exists());
        spec.dry = false;
        spec.confirm = Some("server-write".into());
        spec.preview = Some(dry["preview"].as_str().unwrap().to_owned());
        assert!(adapter.run(game, spec.clone()).is_err());
        fs::write(dir.join(write_gate::FLAG), []).unwrap();
        let outcome = adapter.run(game, spec.clone()).unwrap();
        assert_eq!(outcome["state"], "issued_unknown");
        assert_eq!(outcome["arrival_verified"], false);
        assert_eq!(outcome["native_request_accepted"], true);
        let audit = fs::read(dir.join("journal.jsonl")).unwrap();
        assert_eq!(adapter.run(game, spec.clone()).unwrap(), outcome);
        assert_eq!(fs::read(dir.join("journal.jsonl")).unwrap(), audit);
        assert_eq!(loader.teleports().len(), 1);
        assert_eq!(
            loader.teleports()[0].flags,
            enshwick::abi::TeleportRequest::BEST_SPAWN
        );
        // Exercise the production recorder after the once marker is retained.
        // These synthetic native results do not dispatch additional commands.
        fs::remove_file(dir.join("journal.jsonl")).unwrap();
        fs::create_dir(dir.join("journal.jsonl")).unwrap();
        for (operation, native) in [
            ("outcome-io", Ok(())),
            ("outcome-native", Err(enshwick::WorldError::Refused)),
        ] {
            let mut replay = spec.clone();
            replay.operation = operation.into();
            let mut marker = Operation {
                request: reposition::canonical_request(&replay),
                session: adapter.construction.active_session().unwrap(),
                outcome: json!({"state":"issued_unknown","arrival_verified":false,"automatic_retry":false}),
            };
            reserve(
                &dir,
                operation,
                &json!({"kind":"reposition_intent","operation":operation}),
            )
            .unwrap();
            let recorded = record_outcome(&dir, operation, &mut marker, native);
            assert_eq!(recorded["state"], "issued_unknown");
            assert!(recorded["journal_error"].is_string());
            adapter
                .operations
                .lock()
                .unwrap()
                .insert(operation.into(), marker);
            assert_eq!(adapter.run(game, replay.clone()).unwrap(), recorded);
            assert_eq!(adapter.run(game, replay).unwrap(), recorded);
            assert!(
                dir.join("reposition-operations")
                    .join(format!("{operation}.json"))
                    .exists()
            );
            assert_eq!(loader.teleports().len(), 1);
        }
        fs::remove_dir(dir.join("journal.jsonl")).unwrap();
        fs::write(dir.join("journal.jsonl"), &audit).unwrap();
        let mut changed = spec.clone();
        changed.target[0] += 1.0;
        assert!(adapter.run(game, changed).is_err());
        let mut stale = spec.clone();
        stale.operation = "stale-preview".into();
        let left = enshwick::abi::PlayerData {
            header: Header::of::<enshwick::abi::PlayerData>(),
            slot: 0,
            entity: Entity(7),
            key: enshwick::PlayerKey::new(9),
        };
        assert!(loader.raise::<enshwick::events::PlayerLeft>(&left) > 0);
        assert!(adapter.run(game, stale).is_err());
        assert_eq!(loader.teleports().len(), 1);
        assert!(
            !dir.join("reposition-operations/stale-preview.json")
                .exists()
        );
        // Losing the in-memory receipt never permits durable identity reuse.
        adapter.operations.lock().unwrap().clear();
        assert!(adapter.run(game, spec).is_err());
        assert_eq!(loader.teleports().len(), 1);
        health[8..12].copy_from_slice(&0i32.to_le_bytes());
        assert!(loader.set_component(Entity(7), 1, &health));
        let dead = reposition::check(
            json!({"operation":"dead","player":"actor","x":101,"y":20,"z":100})
                .as_object()
                .unwrap(),
        )
        .unwrap();
        assert!(adapter.run(game, dead).is_err());
        assert!(!dir.join("reposition-operations/dead.json").exists());
        assert_eq!(loader.teleports().len(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}
