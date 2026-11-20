//! Retained synthetic SDK roster/host facts do not establish current MCP scope.
use crate::tools::GameTools;
use enshwick::{
    Entity, Load, Mod, Off, PlayerKey,
    abi::{Header, PlayerInfo, Side, Status},
    schedule::Scheduler,
    world::World,
};
use policy::mcp::Tools;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone)]
struct Context {
    tools: Arc<GameTools>,
    world: World,
    scheduler: Scheduler,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            world: load.world()?,
            scheduler: load.scheduler()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Reader, "id = \"test.roster-lifecycle\"\nside = \"both\"");
fn start(hosting: bool) -> (enshwick::test::Installed, Context) {
    let mut player = PlayerInfo {
        header: Header::of::<PlayerInfo>(),
        slot: 0,
        entity: Entity(40),
        key: PlayerKey::new(7),
        permissions: 0,
        name_len: 10,
        name: [0; PlayerInfo::NAME_MAX],
    };
    player.name[..10].copy_from_slice(b"Researcher");
    // Only the nonhosting diagnostic uses a published client-side fake. It grants no real client World access.
    let loader = enshwick::test::FakeHost::new()
        .side(if hosting { Side::SERVER } else { Side::CLIENT })
        .hosting(hosting)
        .world()
        .world_key("fixture-roster-world")
        .player(player)
        .install();
    // SAFETY: generated exports belong to this isolated fake-only reader.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let c = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
    (loader, c)
}
fn positive(c: &Context) {
    assert!(enshwick::host().unwrap().hosting());
    assert_eq!(c.world.key().as_deref(), Some("fixture-roster-world"));
    let players = c.world.players();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0].entity, Entity(40));
    assert_eq!(players[0].key.0, 7);
}
fn finish(
    loader: &enshwick::test::Installed,
    worker: thread::JoinHandle<Result<Value, String>>,
) -> Result<Value, String> {
    let until = Instant::now() + Duration::from_secs(2);
    while !worker.is_finished() && Instant::now() < until {
        let _ = loader.tick(Duration::from_millis(1));
        thread::yield_now();
    }
    assert!(worker.is_finished(), "roster request did not complete");
    worker.join().unwrap()
}
fn queue(tool: &'static str) {
    for phase in ["unchanged", "reload", "shutdown"] {
        let (loader, c) = start(true);
        positive(&c);
        let _ = loader.tick(Duration::from_millis(1));
        let scheduler = c.scheduler;
        assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
        let tools = c.tools.clone();
        let worker = thread::spawn(move || tools.call(tool, json!({}).as_object().unwrap()));
        // Pinned SDK Debug is only an enqueue barrier for this isolated queue.
        let until = Instant::now() + Duration::from_secs(2);
        while format!("{scheduler:?}") != "Scheduler { pending: true }" {
            assert!(!worker.is_finished(), "read did not queue");
            assert!(Instant::now() < until, "queue barrier");
            thread::yield_now();
        }
        match phase {
            "reload" => assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0),
            "shutdown" => assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0),
            _ => {}
        }
        positive(&c);
        let result = finish(&loader, worker);
        if phase == "unchanged" {
            let value = result.unwrap();
            if tool == "players" {
                assert_eq!(value.as_array().unwrap().len(), 1);
                assert_eq!(value[0]["key"], "7");
                assert_eq!(value[0]["name"], "Researcher");
            } else {
                assert_eq!(value["hosting"], true);
                assert_eq!(value["players"], 1);
                assert_eq!(value["world_key"], "fixture-roster-world");
            }
        } else {
            let error = result.expect_err(
                "queued changed-context roster/status read returned retained world data",
            );
            assert!(
                error.contains(if phase == "reload" {
                    "session"
                } else {
                    "active"
                }),
                "{tool}/{phase}: {error}"
            );
        }
    }
}
fn shutdown_admission(tool: &'static str) {
    let (loader, c) = start(true);
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    let tools = c.tools.clone();
    let worker = thread::spawn(move || tools.call(tool, json!({}).as_object().unwrap()));
    let error = finish(&loader, worker).unwrap_err();
    assert!(error.contains("active"), "{tool}: {error}");
}
#[test]
fn queued_players_refuse_retained_roster_after_lifecycle_changes() {
    queue("players");
}
#[test]
fn queued_hosted_status_refuses_retained_world_after_lifecycle_changes() {
    queue("status");
}
#[test]
fn players_refuse_shutdown_before_admission() {
    shutdown_admission("players");
}
#[test]
fn hosted_status_refuses_shutdown_before_admission() {
    shutdown_admission("status");
}
#[test]
fn nonhosting_status_remains_immediate_diagnostic_while_inactive() {
    let (loader, c) = start(false);
    assert!(!enshwick::host().unwrap().hosting());
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    let scheduler = c.scheduler;
    assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
    let value = c
        .tools
        .call("status", json!({}).as_object().unwrap())
        .unwrap();
    assert_eq!(value["hosting"], false);
    assert!(value.get("world_key").is_none());
    assert!(value.get("players").is_none());
    assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
}
