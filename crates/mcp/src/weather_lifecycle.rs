//! Retained SDK weather is not proof of an active, unchanged MCP world.
use crate::tools::GameTools;
use enshwick::{Load, Mod, Off, abi::Status, schedule::Scheduler, weather::Weather, world::World};
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
    weather: Weather,
    world: World,
    scheduler: Scheduler,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            weather: load.weather(),
            world: load.world()?,
            scheduler: load.scheduler()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Reader, "id = \"test.weather-lifecycle\"\nside = \"server\"");

fn start() -> (enshwick::test::Installed, Context) {
    let loader = enshwick::test::FakeHost::new()
        .world()
        .world_key("fixture-weather-world")
        .install();
    // SAFETY: generated exports belong to this isolated fake-only reader.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    loader.set_weather([0.25, 0.5, 0.25, 0.], &[]);
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let c = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
    positive(&c);
    (loader, c)
}
fn positive(c: &Context) {
    let sample = c.weather.at([1.25, 2.5, 3.75]).unwrap();
    assert_eq!(
        [sample.clear, sample.rain, sample.snow, sample.blizzard],
        [0.25, 0.5, 0.25, 0.]
    );
    assert_eq!(c.world.key().as_deref(), Some("fixture-weather-world"));
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
    assert!(worker.is_finished(), "weather request did not complete");
    worker.join().unwrap()
}
fn request(tools: Arc<GameTools>) -> thread::JoinHandle<Result<Value, String>> {
    thread::spawn(move || {
        tools.call(
            "weather_read",
            json!({"x":1.25,"y":2.5,"z":3.75}).as_object().unwrap(),
        )
    })
}
fn queued(phase: &str) {
    let (loader, c) = start();
    let _ = loader.tick(Duration::from_millis(1));
    let scheduler = c.scheduler;
    assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
    let worker = request(c.tools.clone());
    let until = Instant::now() + Duration::from_secs(2);
    // Pinned SDK Debug is only an enqueue barrier for this isolated fake.
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
        assert_eq!(value["state"], "observed");
        assert_eq!(
            value["weights"],
            json!({"clear":0.25,"rain":0.5,"snow":0.25,"blizzard":0.})
        );
        assert_eq!(value["dominant_sdk_state"], "rain");
        assert_eq!(value["world_key"], "fixture-weather-world");
        assert_eq!(value["world_mutated"], false);
    } else {
        let error = result.expect_err("changed-context weather read returned retained SDK data");
        assert!(
            error.contains(if phase == "reload" {
                "session"
            } else {
                "active"
            }),
            "{error}"
        );
    }
    assert!(loader.forecasts().is_empty());
}
#[test]
fn unchanged_weather_preserves_positive_weights_and_world_key() {
    queued("unchanged");
}
#[test]
fn queued_weather_refuses_retained_weights_after_reload() {
    queued("reload");
}
#[test]
fn queued_weather_refuses_retained_weights_after_shutdown() {
    queued("shutdown");
}
#[test]
fn weather_refuses_shutdown_before_admission() {
    let (loader, c) = start();
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    let error = finish(&loader, request(c.tools.clone())).unwrap_err();
    assert!(error.contains("active"), "{error}");
    assert!(loader.forecasts().is_empty());
}
