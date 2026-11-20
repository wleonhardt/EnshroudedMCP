//! Actual dispatcher lifecycle regression over published synthetic typed data.
#[path = "../tests/support/factory.rs"]
mod fixture;

use crate::tools::GameTools;
use enshwick::{
    Entity, Load, Mod, Off,
    abi::Status,
    resources::fnv1a32,
    schedule::Scheduler,
    typed::{ComponentType, ecs::FactoryStation},
    world::World,
};
use policy::mcp::Tools;
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Context {
    tools: Arc<GameTools>,
    world: World,
    station: ComponentType<FactoryStation>,
    scheduler: Scheduler,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            world: load.world()?,
            station: ComponentType::resolve(load)?,
            scheduler: load.scheduler()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Reader, "id = \"test.factory-lifecycle\"\nside = \"server\"");
fn start() -> (enshwick::test::Installed, Context) {
    let hash = fnv1a32(fixture::FACTORY.as_bytes());
    let mut bytes = [0u8; 128];
    bytes[0] = 4; // Synthetic reflected Running enum value.
    bytes[8..12].copy_from_slice(&42u32.to_le_bytes());
    let loader = enshwick::test::FakeHost::new()
        .world()
        .types(fixture::types())
        .type_hash("ecs::FactoryStation", hash)
        .type_size("ecs::FactoryStation", 128)
        .component(Entity(77), hash, &bytes)
        .install();
    // SAFETY: these exports are generated for this isolated fake-only reader.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let context = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
    (loader, context)
}
fn assert_retained(context: &Context) {
    let value = context
        .station
        .read(context.world, Entity(77))
        .unwrap()
        .unwrap();
    assert_eq!(value.running_recipe.value, 42);
    assert_eq!(format!("{:?}", value.state), "Running");
}
#[test]
fn queued_station_reads_refuse_changed_lifecycle_with_retained_typed_data() {
    for phase in ["unchanged", "reload", "shutdown"] {
        let (loader, context) = start();
        let _ = loader.tick(Duration::from_millis(1));
        let scheduler = context.scheduler;
        assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
        let tools = context.tools.clone();
        let worker = thread::spawn(move || {
            tools.call(
                "factory_station_read",
                json!({"entity":77}).as_object().unwrap(),
            )
        });
        // Pinned SDK Debug is only an enqueue barrier in an isolated queue.
        let until = Instant::now() + Duration::from_secs(2);
        while format!("{scheduler:?}") != "Scheduler { pending: true }" {
            assert!(!worker.is_finished(), "{phase}: read did not queue");
            assert!(Instant::now() < until, "{phase}: queue barrier not reached");
            thread::yield_now();
        }
        match phase {
            "reload" => assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0),
            "shutdown" => assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0),
            _ => {}
        }
        assert_retained(&context);
        let until = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() && Instant::now() < until {
            let _ = loader.tick(Duration::from_millis(1));
            thread::yield_now();
        }
        let result = worker.join().unwrap();
        if phase == "unchanged" {
            let value = result.unwrap();
            assert_eq!(value["state"], "observed");
            assert_eq!(value["factory"]["running_recipe_id"], 42);
            assert_eq!(value["factory"]["station_state"], "Running");
        } else {
            let error =
                result.expect_err("queued changed-context station read returned an observation");
            assert!(
                error.contains(if phase == "reload" {
                    "session"
                } else {
                    "active"
                }),
                "{phase}: {error}"
            );
        }
    }
}
#[test]
fn shutdown_before_station_admission_refuses_retained_typed_data() {
    let (loader, context) = start();
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    assert_retained(&context);
    assert!(
        context
            .tools
            .call(
                "factory_station_read",
                json!({"entity":77}).as_object().unwrap()
            )
            .unwrap_err()
            .contains("active")
    );
}
