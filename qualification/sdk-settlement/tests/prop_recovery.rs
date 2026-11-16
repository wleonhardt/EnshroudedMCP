//! Published fake-host contracts only; no real props, item data or game access.
use enshwick::abi::{Header, PlayerInfo, Status};
use enshwick::content::Guid;
use enshwick::test::{FakeHost, Installed};
use enshwick::world::{HeldState, TransactionStatus, World, WorldTransform};
use enshwick::{Entity, ItemId, Load, Mod, Off, PlayerKey, WorldError};
use std::sync::Mutex;

static WORLD: Mutex<Option<World>> = Mutex::new(None);
const ITEM: ItemId = ItemId::new(30);
const OLD: Entity = Entity(40);
const AT: [f64; 3] = [100.0, 40.0, 100.0];
const HELD: [HeldState; 9] = [
    HeldState::Inventory,
    HeldState::Growth,
    HeldState::FactoryStation,
    HeldState::NpcBed,
    HeldState::StoreAssociatedNpcIds,
    HeldState::CustomPlayerString,
    HeldState::Fuel,
    HeldState::LogicGraphNodeConnected,
    HeldState::LogicGraphTrigger,
];
struct Probe;
impl Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *WORLD.lock().unwrap() = Some(load.world()?);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(
    Probe,
    "id = \"test.prop-recovery-contract\"\nside = \"server\""
);

fn start(host: FakeHost) -> (Installed, World) {
    let host = host
        .world()
        .item(ITEM, 1, 0)
        .item_places(ITEM, Guid([0x3a; 16]))
        .install();
    // SAFETY: the exports above belong to this isolated test mod.
    assert_eq!(
        unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    host.set_terrain(&[]);
    (host, WORLD.lock().unwrap().unwrap())
}

#[test]
fn prop_permissions_and_held_state_are_independent_recovery_gates() {
    // The anchor is allowed in both cases. Turning the long box intersects
    // a denied zone far from the anchor; admission alone is not acceptance.
    {
        let (host, world) = start(
            FakeHost::new()
                .placement_box(ITEM, [-0.5, 0.0, -5.0], [0.5, 2.0, 5.0])
                .player(PlayerInfo {
                    header: Header::of::<PlayerInfo>(),
                    slot: 2,
                    entity: Entity(2),
                    key: PlayerKey::new(2),
                    permissions: PlayerInfo::CAN_EDIT_WORLD,
                    name_len: 0,
                    name: [0; PlayerInfo::NAME_MAX],
                })
                .building_area([0.0; 3], [1000.0; 3])
                .no_build_zone([104.0, 0.0, 0.0], [106.0, 1000.0, 1000.0], true),
        );
        let actor = world.player(Entity(2)).unwrap();
        let along = world
            .transaction_for(&actor)
            .place_prop(ITEM, WorldTransform::at(AT))
            .commit()
            .unwrap();
        let quarter = core::f32::consts::FRAC_1_SQRT_2;
        let across = world
            .transaction_for(&actor)
            .place_prop(
                ITEM,
                WorldTransform {
                    position: AT,
                    orientation: [0.0, quarter, 0.0, quarter],
                },
            )
            .commit()
            .unwrap();
        assert_eq!(
            world.transaction_status(along),
            Some(TransactionStatus::Pending)
        );
        assert_eq!(
            world.transaction_status(across),
            Some(TransactionStatus::Pending)
        );
        assert!(host.props().is_empty());
        host.run_transactions();
        assert_eq!(
            world.transaction_status(along),
            Some(TransactionStatus::Applied)
        );
        assert!(matches!(
            world.transaction_status(across),
            Some(TransactionStatus::NotAllowed(_))
        ));
        assert_eq!(host.props().len(), 1);
        assert!(world.transaction_entities(across).is_empty());
    }
    for held in HELD {
        // State changes after admission must prevent the entire mixed write,
        // including an unrelated placement ordered before the removal.
        {
            let (host, world) = start(FakeHost::new().prop(OLD, ITEM, AT));
            let before = host.props();
            let id = world
                .transaction()
                .place_prop(ITEM, WorldTransform::at([110.0, 40.0, 100.0]))
                .remove_prop(OLD)
                .commit()
                .unwrap();
            host.set_prop_state(OLD, &[held]);
            host.run_transactions();
            match world.transaction_status(id) {
                Some(TransactionStatus::Failed(WorldError::HoldsState(state))) => {
                    assert_eq!(state.iter().collect::<Vec<_>>(), vec![held]);
                }
                other => panic!("{held:?}: {other:?}"),
            }
            assert_eq!(host.props(), before);
            assert!(world.transaction_entities(id).is_empty());
        }
        // A completed mixed write is still not unconditional undo authority.
        // State acquired after the undo request is queued blocks both removing
        // its new prop and restoring its old prop. No automatic retry follows.
        {
            let (host, world) = start(FakeHost::new().prop(OLD, ITEM, AT));
            let id = world
                .transaction()
                .remove_prop(OLD)
                .place_prop(ITEM, WorldTransform::at([110.0, 40.0, 100.0]))
                .commit()
                .unwrap();
            host.run_transactions();
            assert_eq!(
                world.transaction_status(id),
                Some(TransactionStatus::Applied)
            );
            let created = world.transaction_entities(id);
            assert_eq!(created.len(), 1);
            let before = host.props();
            assert_eq!(before.len(), 1);
            assert_eq!(before[0].2, [110.0, 40.0, 100.0]);
            world.undo(id).unwrap();
            assert_eq!(
                world.transaction_status(id),
                Some(TransactionStatus::Undoing)
            );
            host.set_prop_state(created[0], &[held]);
            host.run_transactions();
            match world.transaction_status(id) {
                Some(TransactionStatus::UndoRefused(WorldError::HoldsState(state))) => {
                    assert_eq!(state.iter().collect::<Vec<_>>(), vec![held]);
                }
                other => panic!("{held:?}: {other:?}"),
            }
            assert_eq!(host.props(), before);
            assert_eq!(world.transaction_entities(id), created);
        }
    }
    // An empty-state control verifies this fixture can observe completed undo.
    {
        let (host, world) = start(FakeHost::new().prop(OLD, ITEM, AT));
        let id = world
            .transaction()
            .remove_prop(OLD)
            .place_prop(ITEM, WorldTransform::at([110.0, 40.0, 100.0]))
            .commit()
            .unwrap();
        host.run_transactions();
        world.undo(id).unwrap();
        host.run_transactions();
        assert_eq!(
            world.transaction_status(id),
            Some(TransactionStatus::Undone)
        );
        let restored = host.props();
        assert_eq!(restored.len(), 1);
        assert_eq!((restored[0].1, restored[0].2), (ITEM, AT));
        assert!(world.transaction_entities(id).is_empty());
    }
}
