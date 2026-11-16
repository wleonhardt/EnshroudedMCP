//! Exercise the real construction adapter against the published SDK fake host.
//! Its transaction runner reports Applied without changing its voxel store;
//! this fixture proves refusal on that disagreement, not successful placement.
#[path = "../src/construction.rs"]
#[allow(dead_code)] // This binary exercises only the construction adapter.
mod adapter;
#[path = "../src/journal.rs"]
#[allow(dead_code)]
mod journal;

// Use the production adapter with its exact SDK handle types. A separate test
// binary isolates the SDK's once-per-process reflection cache from other probes.
mod tools {
    #[derive(Clone, Copy)]
    pub(crate) struct Game {
        pub(crate) world: enshwick::world::World,
        pub(crate) voxels: enshwick::voxels::Voxels,
    }
}
use adapter::Construction;
use enshwick::abi::{Header, PlayerInfo, Status};
use enshwick::{
    Load, Off,
    voxels::{Voxel, VoxelMaterial},
};
use policy::{construction, write_gate};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

static ADAPTER: Mutex<Option<(Construction, crate::tools::Game)>> = Mutex::new(None);
struct Probe;
impl enshwick::Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let game = crate::tools::Game {
            world: load.world()?,
            voxels: load.voxels(),
        };
        *ADAPTER.lock().unwrap() = Some((Construction::new(load)?, game));
        Ok(Self)
    }
}
enshwick::test::declare_mod!(
    Probe,
    r#"id = "test.construction-flow"
side = "server""#
);

#[test]
fn applied_receipt_without_postimage_blocks_acceptance_progression_and_multicell_undo() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../gamedata/tests");
    fs::create_dir_all(&base).unwrap();
    let base = base.canonicalize().unwrap();
    let folder = base.join(format!(
        "construction-flow-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join(write_gate::FLAG), []).unwrap();
    let loader = enshwick::test::FakeHost::new()
        .world()
        .items()
        .type_hash("animal_saved", 901)
        .type_hash("active_npc_state_saved", 902)
        .type_hash("base_id_component", 903)
        .hosting(true)
        .player(PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: enshwick::Entity(7),
            key: enshwick::PlayerKey::new(9),
            permissions: PlayerInfo::CAN_EDIT_BASE | PlayerInfo::CAN_EDIT_WORLD,
            name_len: 0,
            name: [0; 128],
        })
        .building_zone([0.0; 3], [100.0; 3])
        .voxels(
            [32, 32, 32].into(),
            [8; 3],
            &vec![Voxel::new(VoxelMaterial::Air, 0); 512],
        )
        .voxels(
            [48, 32, 32].into(),
            [1; 3],
            &[Voxel::new(VoxelMaterial::Building(0x83), 7)],
        )
        .mod_dir(folder.to_str().unwrap())
        .install();
    // SAFETY: Probe generates this module's init export.
    assert_eq!(unsafe { loader.init(enshwick_mod_init) }, Status::OK);
    loader.set_terrain(&[]);
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let (control, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
    game.voxels
        .sample([48, 32, 32].into())
        .expect("source voxel fixture");
    game.world
        .can_build(&game.world.players()[0], [16.0; 3], [17.0, 16.5, 16.5])
        .expect("build rights fixture");
    game.world
        .persists([16.0; 3], [17.0, 16.5, 16.5])
        .expect("persistence fixture");
    game.world
        .capture([32, 32, 32].into(), [39, 39, 39].into())
        .expect("whole capture fixture");
    let apply = |request: &Value| {
        control.apply_plan(
            game,
            construction::check_plan(request.as_object().unwrap())?,
        )
    };
    let request = json!({"operation":"phase-a","player":"9","source_x":24,"source_y":16,"source_z":16,
        "plan":{"kind":"floor","x":16,"y":16,"z":16,"width":1,"height":0.5,"depth":0.5}});
    let preview = apply(&request).unwrap();
    assert_eq!(preview["state"], "dry");
    assert_eq!(preview["planned_cells"], 2);
    assert!(loader.transactions().is_empty());
    assert!(!folder.join("journal.jsonl").exists());
    let budget = || crate::journal::report(&folder, &serde_json::Map::new()).unwrap();
    assert_eq!(budget()["recovery_reservation_count"], 0);
    let mut real = request.clone();
    real["dry"] = json!(false);
    real["confirm"] = json!("server-write");
    real["session"] = preview["session"].clone();
    assert_eq!(apply(&real).unwrap()["state"], "pending");
    assert_eq!(loader.transactions().len(), 1);
    assert_eq!(budget()["recovery_reservation_count"], 1);
    assert_eq!(
        budget()["retained_recovery_bytes"],
        3 * construction::MAX_RECORD_BYTES
    );
    loader.run_transactions();
    let status = control.status(game, "phase-a".into()).unwrap();
    assert_eq!(status["state"], "verification_conflict_unknown");
    assert_eq!(status["whole_capture_expected_equal"], false);
    assert_eq!(status["whole_capture_baseline_equal"], true);
    assert_eq!(status["settled"], false);
    let journal = fs::read(folder.join("journal.jsonl")).unwrap();
    let retained = budget();
    for dry in [true, false] {
        let args = json!({"operation":"phase-a","dry":dry,"confirm":"server-write"});
        assert!(
            control
                .accept(
                    game,
                    construction::check_accept(args.as_object().unwrap()).unwrap()
                )
                .unwrap_err()
                .contains("exact expected whole capture")
        );
        assert!(
            control
                .undo(
                    game,
                    construction::check_undo(args.as_object().unwrap()).unwrap()
                )
                .unwrap_err()
                .contains("exact expected voxel postimage")
        );
    }
    let mut replay = real.clone();
    replay["dry"] = json!(true);
    assert_eq!(apply(&real).unwrap()["state"], status["state"]);
    assert_eq!(apply(&replay).unwrap()["state"], status["state"]);
    let mut next = real.clone();
    next["operation"] = json!("phase-b");
    assert!(apply(&next).is_err());
    next["dry"] = json!(true);
    assert!(apply(&next).is_err());
    assert!(!folder.join("construction-operations/phase-b.json").exists());
    let mut conflict = real.clone();
    conflict["plan"]["x"] = json!(16.5);
    assert!(apply(&conflict).is_err());
    assert_eq!(loader.transactions().len(), 1);
    assert_eq!(fs::read(folder.join("journal.jsonl")).unwrap(), journal);
    assert_eq!(budget(), retained);
    assert_eq!(
        loader.transactions()[0].info.state,
        enshwick::abi::TransactionInfo::APPLIED
    );
    let refused = control.status(game, "phase-a".into()).unwrap();
    assert_eq!(refused["accepted"], false);
    assert_eq!(refused["undo_requested"], false);
    fs::remove_file(folder.join(write_gate::FLAG)).unwrap();
    assert!(apply(&real).is_err());
    assert_eq!(apply(&replay).unwrap()["state"], status["state"]);
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    assert_eq!(budget()["recovery_reservation_count"], 0);
    assert!(apply(&request).is_err());
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let stale = control.status(game, "phase-a".into()).unwrap();
    assert_eq!(stale["state"], "stale_session_unknown");
    assert_eq!(stale["undo_available"], false);
    assert_eq!(loader.transactions().len(), 1);
    assert_eq!(fs::read(folder.join("journal.jsonl")).unwrap(), journal);
    *ADAPTER.lock().unwrap() = None;
    crate::journal::reset(&folder).unwrap();
    let resolved = folder.canonicalize().unwrap();
    assert!(resolved.starts_with(&base) && resolved != base);
    fs::remove_dir_all(resolved).unwrap();
}
