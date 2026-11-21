//! Retained typed SDK health is not proof of an active, unchanged MCP world.
use crate::tools::GameTools;
use enshwick::{
    Entity, Load, Mod, Off,
    abi::Status,
    content::EngineFacts,
    resources::{Field, Kind, Type, fnv1a32},
    schedule::Scheduler,
    typed::{ComponentType, ecs::NetworkHealth},
    world::World,
};
use policy::mcp::Tools;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
const NAME: &str = "keen::ecs::NetworkHealth";
#[derive(Clone)]
struct Context {
    tools: Arc<GameTools>,
    world: World,
    health: ComponentType<NetworkHealth>,
    scheduler: Scheduler,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            world: load.world()?,
            health: ComponentType::resolve(load)?,
            scheduler: load.scheduler()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Reader, "id = \"test.health-lifecycle\"\nside = \"server\"");
fn start() -> (enshwick::test::Installed, Context) {
    // Deliberately synthetic padded layout; published named reflection only.
    let types = vec![
        Type::new("keen::uint32", Kind::Uint32, 4, 4),
        Type::new("keen::ecs::Component", Kind::Struct, 1, 1),
        Type::new(NAME, Kind::Struct, 16, 4)
            .with_inner("keen::ecs::Component")
            .with_fields(vec![
                Field::new("health", 4, "keen::uint32"),
                Field::new("healthMax", 12, "keen::uint32"),
            ]),
    ];
    let hash = fnv1a32(NAME.as_bytes());
    let fake = enshwick::test::FakeHost::new()
        .world()
        .world_key("fixture-health-world");
    let transform = fake
        .hash(enshwick::test::facts::CURRENT_TRANSFORM.id())
        .unwrap();
    let mut bytes = [0u8; 16];
    bytes[4..8].copy_from_slice(&37u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&83u32.to_le_bytes());
    let loader = fake
        .types(types)
        .type_hash("ecs::NetworkHealth", hash)
        .type_size("ecs::NetworkHealth", 16)
        .placed(transform, Entity(40), [1., 2., 3.])
        .component(Entity(40), hash, &bytes)
        .placed(transform, Entity(41), [2., 2., 3.])
        .placed(transform, Entity(42), [3., 2., 3.])
        .component(Entity(42), hash, &[0; 16])
        .placed(transform, Entity(43), [100.; 3])
        .install();
    // SAFETY: generated exports belong to this isolated fake-only reader.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let c = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
    positive(&c);
    (loader, c)
}
fn positive(c: &Context) {
    let found = c.world.near([1., 2., 3.], 3., c.world.transform_type());
    assert_eq!(found.len(), 3);
    let value = c.health.read(c.world, Entity(40)).unwrap().unwrap();
    assert_eq!((value.health, value.health_max), (37, 83));
    assert!(c.health.read(c.world, Entity(41)).unwrap().is_none());
    let zero = c.health.read(c.world, Entity(42)).unwrap().unwrap();
    assert_eq!((zero.health, zero.health_max), (0, 0));
    assert_eq!(c.world.key().as_deref(), Some("fixture-health-world"));
}
fn request(tools: Arc<GameTools>, limit: u32) -> thread::JoinHandle<Result<Value, String>> {
    thread::spawn(move || {
        tools.call(
            "entity_health",
            json!({"x":1,"y":2,"z":3,"radius":3,"limit":limit})
                .as_object()
                .unwrap(),
        )
    })
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
    assert!(worker.is_finished(), "health request did not complete");
    worker.join().unwrap()
}
fn queued(phase: &str) {
    let (loader, c) = start();
    let _ = loader.tick(Duration::from_millis(1));
    let scheduler = c.scheduler;
    assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
    let worker = request(c.tools.clone(), 3);
    let until = Instant::now() + Duration::from_secs(2);
    // Pinned SDK Debug is only an enqueue barrier for this isolated fake.
    while format!("{scheduler:?}") != "Scheduler { pending: true }" {
        assert!(!worker.is_finished(), "health read did not queue");
        assert!(Instant::now() < until, "health queue barrier");
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
        assert_eq!(value["total_candidates"], 3);
        assert_eq!(value["truncated"], false);
        assert_eq!(value["component"], NAME);
        assert_eq!(value["scope"], "nearest placed entities");
        assert_eq!(value["world_key"], "fixture-health-world");
        let rows = value["entities"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        let present = rows.iter().find(|v| v["entity"] == 40).unwrap();
        assert_eq!(present["state"], "present");
        assert_eq!(present["health"], 37);
        assert_eq!(present["health_max"], 83);
        assert_eq!(present["position"], json!([1., 2., 3.]));
        assert_eq!(present["distance"], 0.);
        let absent = rows.iter().find(|v| v["entity"] == 41).unwrap();
        assert_eq!(absent["state"], "absent");
        assert!(absent.get("health").is_none());
        let zero = rows.iter().find(|v| v["entity"] == 42).unwrap();
        assert_eq!(zero["state"], "present");
        assert_eq!(zero["health"], 0);
        assert_eq!(zero["health_max"], 0);
        let limited = finish(&loader, request(c.tools.clone(), 1)).unwrap();
        assert_eq!(limited["total_candidates"], 3);
        assert_eq!(limited["truncated"], true);
        assert_eq!(limited["entities"].as_array().unwrap().len(), 1);
    } else {
        let error = result.expect_err("changed-context health read returned retained SDK data");
        assert!(
            error.contains(if phase == "reload" {
                "session"
            } else {
                "active"
            }),
            "{error}"
        );
    }
    positive(&c);
    assert!(loader.labels().is_empty());
    assert!(loader.destroyed().is_empty());
}
#[test]
fn unchanged_health_preserves_present_absent_and_limits() {
    queued("unchanged");
}
#[test]
fn queued_health_refuses_retained_values_after_reload() {
    queued("reload");
}
#[test]
fn queued_health_refuses_retained_values_after_shutdown() {
    queued("shutdown");
}
#[test]
fn health_refuses_shutdown_before_admission_without_queueing() {
    let (loader, c) = start();
    let _ = loader.tick(Duration::from_millis(1));
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    let worker = request(c.tools.clone(), 3);
    let scheduler = c.scheduler;
    let until = Instant::now() + Duration::from_secs(2);
    while !worker.is_finished()
        && format!("{scheduler:?}") != "Scheduler { pending: true }"
        && Instant::now() < until
    {
        thread::yield_now();
    }
    let queued = format!("{scheduler:?}") == "Scheduler { pending: true }";
    let result = finish(&loader, worker);
    assert!(
        !queued,
        "inactive health request consumed a game-thread queue slot"
    );
    assert!(result.unwrap_err().contains("active"));
    positive(&c);
}
