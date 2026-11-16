//! New public SDK helper only; every transaction below belongs to a fake host.
use enshwick::abi::Status;
use enshwick::world::{Settled, TransactionId, TransactionStatus, World, WorldTransform};
use enshwick::{Entity, ItemId, Load, Mod, Off, WorldError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static WORLD: Mutex<Option<World>> = Mutex::new(None);
struct Probe;
impl Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *WORLD.lock().unwrap() = Some(load.world()?);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(
    Probe,
    "id = \"test.settlement-contract\"\nside = \"server\""
);
const ITEM: ItemId = ItemId::new(30);
fn start() -> (enshwick::test::Installed, World) {
    let host = enshwick::test::FakeHost::new()
        .world()
        .item(ITEM, 1, 0)
        .item_places(ITEM, enshwick::content::Guid([0x3a; 16]))
        .install();
    // SAFETY: exports are generated for this isolated probe's test mod.
    assert_eq!(
        unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    host.set_terrain(&[]); // Enable the fake transaction API; no terrain is edited.
    let world = WORLD.lock().unwrap().unwrap();
    pass(&host); // Bind subsequent scheduler work to this fake game.
    (host, world)
}
fn pass(host: &enshwick::test::Installed) {
    let _ = host.tick(Duration::from_secs(1));
}
fn place(world: World) -> TransactionId {
    world
        .transaction()
        .place_prop(ITEM, WorldTransform::at([100.0, 40.0, 100.0]))
        .commit()
        .unwrap()
}
fn watch(world: World, id: TransactionId, bound: Duration) -> Arc<Mutex<Vec<Settled>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let copy = seen.clone();
    world
        .when_settled(id, bound, move |v| copy.lock().unwrap().push(v.clone()))
        .unwrap();
    seen
}
#[test]
fn callback_bound_expiry_liveness_and_game_scope_are_not_recovery_authority() {
    // A bound ends observation, not the native transaction.
    {
        let (host, world) = start();
        let id = place(world);
        let seen = watch(world, id, Duration::ZERO);
        pass(&host);
        assert_eq!(
            seen.lock().unwrap()[0].status,
            Some(TransactionStatus::Pending)
        );
        assert_eq!(seen.lock().unwrap()[0].standing(), None);
        host.run_transactions();
        pass(&host);
        assert_eq!(
            world.transaction_status(id),
            Some(TransactionStatus::Applied)
        );
        assert_eq!(host.props().len(), 1);
        assert_eq!(seen.lock().unwrap().len(), 1);
    }
    // Even zero bound waits the extra post-write pass, once applied.
    {
        let (host, world) = start();
        let id = place(world);
        let seen = watch(world, id, Duration::ZERO);
        host.run_transactions();
        pass(&host);
        assert!(seen.lock().unwrap().is_empty());
        pass(&host);
        let values = seen.lock().unwrap();
        assert_eq!(values[0].status, Some(TransactionStatus::Applied));
        assert!(values[0].standing().is_some());
        drop(values);
        pass(&host);
        assert_eq!(seen.lock().unwrap().len(), 1);
    }
    // Cached placed IDs survive receipt expiry; undo authority does not.
    {
        let (host, world) = start();
        let id = place(world);
        let seen = watch(world, id, Duration::from_secs(60));
        host.run_transactions();
        pass(&host);
        let entity = world.transaction_entities(id)[0];
        host.expire_transactions();
        assert!(world.transaction_entities(id).is_empty());
        pass(&host);
        let values = seen.lock().unwrap();
        assert_eq!(values[0].status, Some(TransactionStatus::Expired));
        assert_eq!(values[0].standing(), Some(entity));
        assert!(world.undo(id).is_err());
    }
    // Another transaction can remove the prop before the callback.
    {
        let (host, world) = start();
        let id = place(world);
        let seen = watch(world, id, Duration::from_secs(60));
        host.run_transactions();
        pass(&host);
        let entity = world.transaction_entities(id)[0];
        let removal = world.transaction().remove_prop(entity).commit().unwrap();
        host.run_transactions();
        assert_eq!(
            world.transaction_status(removal),
            Some(TransactionStatus::Applied)
        );
        assert!(host.props().iter().all(|(e, ..)| *e != entity));
        pass(&host);
        let values = seen.lock().unwrap();
        assert_eq!(values[0].standing(), Some(entity)); // Historical receipt, not current liveness.
    }
    // Failure is observed without a placed entity or a retry.
    {
        let (host, world) = start();
        let id = world
            .transaction()
            .remove_prop(Entity(999))
            .commit()
            .unwrap();
        let seen = watch(world, id, Duration::from_secs(60));
        host.run_transactions();
        pass(&host);
        let values = seen.lock().unwrap();
        assert_eq!(
            values[0].status,
            Some(TransactionStatus::Failed(WorldError::NotAProp))
        );
        assert_eq!(values[0].standing(), None);
        assert!(values[0].props.is_empty());
        assert_eq!(host.transactions().len(), 1);
    }
    // Undoing counts as an end to this observer, not completed undo.
    {
        let (host, world) = start();
        let id = place(world);
        host.run_transactions();
        let seen = watch(world, id, Duration::from_secs(60));
        world.undo(id).unwrap();
        pass(&host);
        let values = seen.lock().unwrap();
        assert_eq!(values[0].status, Some(TransactionStatus::Undoing));
        assert_eq!(values[0].standing(), None);
        drop(values);
        host.run_transactions();
        pass(&host);
        assert_eq!(
            world.transaction_status(id),
            Some(TransactionStatus::Undone)
        );
        assert_eq!(seen.lock().unwrap().len(), 1);
    }
    // Game-owned waits disappear on the next game; no completion callback is promised.
    {
        let (host, world) = start();
        let id = place(world);
        let seen = watch(world, id, Duration::from_secs(60));
        host.end_game();
        pass(&host);
        host.run_transactions();
        pass(&host);
        pass(&host);
        assert!(seen.lock().unwrap().is_empty());
    }
}
