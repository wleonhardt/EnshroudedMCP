//! Retained SDK props are not proof of an active, unchanged MCP world.
use crate::tools::GameTools;
use enshwick::content::EngineFacts;
use enshwick::{
    Entity, ItemId, Load, Mod, Off, abi::Status, content::Guid, schedule::Scheduler, world::World,
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
enshwick::test::declare_mod!(Reader, "id = \"test.props-lifecycle\"\nside = \"server\"");
// Synthetic component bytes follow the pinned official SDK capture tests.
// Every layout value comes from its published FakeHost facts, never the game.
fn placed_prop(
    fake: enshwick::test::FakeHost,
    entity: u32,
    item: u32,
    position: [f64; 3],
) -> enshwick::test::FakeHost {
    use enshwick::test::facts;
    let schema = enshwick::test::FakeHost::new().world();
    let base = schema
        .offset(facts::CURRENT_TRANSFORM_TRANSFORM.id())
        .unwrap();
    let p = base + schema.offset(facts::WORLD_TRANSFORM_POSITION.id()).unwrap();
    let o = base
        + schema
            .offset(facts::WORLD_TRANSFORM_ORIENTATION.id())
            .unwrap();
    let mut transform = vec![0u8; (p + 24).max(o + 16)];
    for (i, axis) in enshwick::ecs::fixed(position).iter().enumerate() {
        transform[p + i * 8..p + i * 8 + 8].copy_from_slice(&axis.to_le_bytes());
    }
    for (i, part) in [0f32, 0., 0., 1.].iter().enumerate() {
        transform[o + i * 4..o + i * 4 + 4].copy_from_slice(&part.to_le_bytes());
    }
    let used = schema.hash(facts::USED_ITEM.id()).unwrap();
    let current = schema.hash(facts::CURRENT_TRANSFORM.id()).unwrap();
    let guid = Guid(std::array::from_fn(|i| i as u8));
    fake.placed(used, Entity(entity), position)
        .placed(current, Entity(entity), position)
        .component(Entity(entity), used, &item.to_le_bytes())
        .component(Entity(entity), current, &transform)
        .item(ItemId::new(item), 1, 0)
        .item_places(ItemId::new(item), guid)
        .template(Entity(entity), guid)
        .prop(Entity(entity), ItemId::new(item), position)
}
fn start() -> (enshwick::test::Installed, Context) {
    let fake = enshwick::test::FakeHost::new()
        .world()
        .items()
        .type_hash("animal_saved", 4)
        .type_hash("active_npc_state_saved", 5)
        .type_hash("base_id_component", 6);
    let fake = placed_prop(fake, 40, 7, [4., 5., 6.]);
    let fake = placed_prop(fake, 41, 8, [7., 8., 9.]);
    let loader = placed_prop(fake, 42, 9, [20., 20., 20.]).install();
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
    assert!(enshwick::host().unwrap().hosting());
    let props = c.world.props_in([1.; 3], [10.; 3]).unwrap();
    assert_eq!(props.len(), 2);
    let prop = props.iter().find(|p| p.entity == Entity(40)).unwrap();
    assert_eq!(prop.item, ItemId::new(7));
    assert_eq!(prop.transform.position, [4., 5., 6.]);
    assert_eq!(
        c.world.template(Entity(40)).unwrap().0,
        std::array::from_fn(|i| i as u8)
    );
}
fn request(tools: Arc<GameTools>) -> thread::JoinHandle<Result<Value, String>> {
    thread::spawn(move || {
        tools.call(
            "props_in",
            json!({"x1":1,"y1":1,"z1":1,"x2":10,"y2":10,"z2":10})
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
    assert!(worker.is_finished(), "props request did not complete");
    worker.join().unwrap()
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
        assert_eq!(value["total"], 2);
        assert_eq!(value["truncated"], false);
        let rows = value["props"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        let first = rows.iter().find(|p| p["entity"] == 40).unwrap();
        assert_eq!(first["item"], 7);
        assert_eq!(first["position"], json!([4., 5., 6.]));
        assert_eq!(first["orientation"], json!([0., 0., 0., 1.]));
        assert_eq!(first["template_guid"], "000102030405060708090a0b0c0d0e0f");
        assert_eq!(
            first.get("label_present"),
            Some(
                &c.world
                    .has_label(Entity(40))
                    .map(Value::Bool)
                    .unwrap_or(Value::Null)
            )
        );
        let second = rows.iter().find(|p| p["entity"] == 41).unwrap();
        assert_eq!(second["item"], 8);
        assert_eq!(second["template_guid"], "000102030405060708090a0b0c0d0e0f");
        assert!(
            value["recovery"]
                .as_str()
                .unwrap()
                .contains("no safe-removal or undo claim")
        );
    } else {
        let error = result.expect_err("changed-context props read returned retained SDK data");
        assert!(
            error.contains(if phase == "reload" {
                "session"
            } else {
                "active"
            }),
            "{error}"
        );
    }
    assert_eq!(loader.props().len(), 3);
    assert!(loader.labels().is_empty());
    assert!(loader.destroyed().is_empty());
}
#[test]
fn unchanged_props_preserve_projection_metadata_and_observation_limits() {
    queued("unchanged");
}
#[test]
fn queued_props_refuse_retained_values_after_reload() {
    queued("reload");
}
#[test]
fn queued_props_refuse_retained_values_after_shutdown() {
    queued("shutdown");
}
#[test]
fn props_refuse_shutdown_before_admission_without_queueing() {
    let (loader, c) = start();
    let _ = loader.tick(Duration::from_millis(1));
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    let worker = request(c.tools.clone());
    let until = Instant::now() + Duration::from_secs(2);
    let scheduler = c.scheduler;
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
        "inactive props request consumed a game-thread queue slot"
    );
    assert!(result.unwrap_err().contains("active"));
    assert!(loader.labels().is_empty());
    assert!(loader.destroyed().is_empty());
}
