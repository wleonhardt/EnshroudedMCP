//! Synthetic SDK inventories remain readable across lifecycle events.
use crate::tools::GameTools;
use enshwick::{
    Entity, Load, Mod, Off, PlayerKey,
    abi::{Header, PlayerInfo, Status},
    inventory::Inventories,
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
const HOLDER: Entity = Entity(40);
const SETUP: u32 = 0x7001;
const INVENTORY: u32 = 0x7002;
const ITEM: u32 = 3073;
#[derive(Clone)]
struct Context {
    tools: Arc<GameTools>,
    inventories: Inventories,
    scheduler: Scheduler,
    world: World,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            inventories: load.inventories()?,
            scheduler: load.scheduler()?,
            world: load.world()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(
    Reader,
    "id = \"test.inventory-lifecycle\"\nside = \"server\""
);
fn start() -> (enshwick::test::Installed, Context) {
    let mut player = PlayerInfo {
        header: Header::of::<PlayerInfo>(),
        slot: 0,
        entity: HOLDER,
        key: PlayerKey::new(7),
        permissions: 0,
        name_len: 10,
        name: [0; PlayerInfo::NAME_MAX],
    };
    player.name[..10].copy_from_slice(b"Researcher");
    // Published FakeHost inventory preset/fixture layout; no game memory access.
    let mut setup = vec![0u8; 0x54];
    setup[..4].copy_from_slice(&41u32.to_le_bytes());
    setup[0x40] = 4;
    setup[0x53] = 1;
    let mut stacks = vec![0u8; 0x60];
    stacks[..4].copy_from_slice(&ITEM.to_le_bytes());
    stacks[4..8].copy_from_slice(&7u32.to_le_bytes());
    let loader = enshwick::test::FakeHost::new()
        .world()
        .inventory_requests()
        .type_hash("inventory_setup", SETUP)
        .type_hash("inventory", INVENTORY)
        .player(player)
        .component(HOLDER, SETUP, &setup)
        .component(Entity(41), INVENTORY, &stacks)
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
    assert_eq!(c.world.players()[0].entity, HOLDER);
    let reader = c.inventories.read();
    let slots = reader.slots(HOLDER);
    assert!(!slots.is_empty());
    assert_eq!(slots[0].item.0, ITEM);
    assert_eq!(slots[0].count, 7);
}
fn args(player: bool) -> Value {
    let mut value = if player {
        json!({"player":"7"})
    } else {
        json!({"entity":40})
    };
    value["limit"] = json!(1);
    value["item_ids"] = json!([ITEM]);
    value
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
    assert!(worker.is_finished(), "inventory request did not complete");
    worker.join().unwrap()
}
fn queued(player: bool) {
    for phase in ["unchanged", "reload", "shutdown"] {
        let (loader, c) = start();
        let _ = loader.tick(Duration::from_millis(1));
        let scheduler = c.scheduler;
        assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
        let tools = c.tools.clone();
        let request = args(player);
        let worker =
            thread::spawn(move || tools.call("inventory_read", request.as_object().unwrap()));
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
            assert_eq!(value["entity"], 40);
            assert_eq!(value["slots"][0]["count"], 7);
            assert_eq!(value["item_counts"]["counts"][0]["observed_count"], 7);
            assert_eq!(value["item_counts"]["crafting_eligibility"], "unknown");
        } else {
            let error = result
                .expect_err("queued changed-context inventory read returned retained slots/counts");
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
fn queued_entity_inventory_refuses_lifecycle_changes_with_retained_slots() {
    queued(false);
}
#[test]
fn queued_player_inventory_refuses_lifecycle_changes_with_retained_roster_and_slots() {
    queued(true);
}
#[test]
fn shutdown_before_inventory_admission_refuses_both_selectors() {
    let (loader, c) = start();
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    for player in [false, true] {
        let tools = c.tools.clone();
        let request = args(player);
        let worker =
            thread::spawn(move || tools.call("inventory_read", request.as_object().unwrap()));
        let error = finish(&loader, worker).unwrap_err();
        assert!(error.contains("active"), "{error}");
    }
}
