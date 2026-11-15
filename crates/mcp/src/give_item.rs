//! Queue legacy grants with fresh dispatch admission and durable audit records.
use crate::tools::{Game, GameTools, give_dispatch};
use policy::write_gate::{self, PendingWrite};
use serde_json::{Map, Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};

const WAIT: Duration = Duration::from_secs(5);

fn outcome(dir: Option<&std::path::Path>, request: &str, result: Value) -> Result<Value, String> {
    GameTools::journal(dir, "give_item", &json!({"kind":"give_item_outcome","request":request,"result":result}), 0)
        .map_err(|error| format!("inventory outcome may have committed; outcome journal failed; inspect inventory and do not blindly retry: {error}"))?;
    Ok(result)
}

pub(crate) fn call(
    game: Game,
    dir: Option<PathBuf>,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    let inventories = game
        .inventories
        .ok_or("this build lacks the inventory transactions")?;
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(false);
    let who = args["player"].as_str().unwrap_or("").to_owned();
    let name = args["item"].as_str().unwrap_or("").to_owned();
    let confirm = args
        .get("confirm")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let count = args.get("count").and_then(Value::as_u64).unwrap_or(1);
    let count = u32::try_from(count)
        .ok()
        .filter(|count| (1..=100).contains(count))
        .ok_or("count must be 1 to 100")?;
    let pending = Arc::new(PendingWrite::default());
    let queued = pending.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    game.scheduler.next_tick(move || {
        if !queued.begin() { return; }
        let result = (|| -> Result<(), String> {
            let guid = enshwick::content::vanilla::item(&name).map_err(|error| error.to_string())?;
            let player = policy::inspection::unique_player(game.world.players().into_iter()
                .filter(|player| player.key.0.to_string() == who || player.is_named(&who)))?;
            let item = enshwick::ItemId::from_guid(guid);
            let plan = json!({"player":player.name,"item":name,"item_id":item.0,"guid":guid.to_string(),"count":count});
            if dry {
                let _ = tx.send(Ok(json!({"dry":true,"would_give":plan})));
                return Ok(());
            }
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let request = format!("{}-{}-{}",std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos(), NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed));
            // Admission before recording intent, and again immediately before SDK submission.
            give_dispatch(dir.as_deref(),confirm.as_deref(), || ())?;
            GameTools::journal(dir.as_deref(), "give_item", &json!({"kind":"give_item_intent","request":request,"plan":plan}),write_gate::JOURNAL_RECORD_BYTES)?;
            let callback_dir = dir.clone();
            let callback_request = request.clone();
            let callback_tx = tx.clone();
            match give_dispatch(dir.as_deref(),confirm.as_deref(), || inventories.give(player.entity,item,count,move |native| {
                let result = json!({"given":native.is_committed(),"outcome":format!("{:?}",native.done),"plan":plan,"request":callback_request});
                let recorded = outcome(callback_dir.as_deref(),&callback_request,result);
                if let Err(error) = &recorded { enshwick::warn!("{error}"); }
                let _ = callback_tx.send(recorded);
            })) {
                Ok(_) => Ok(()),
                Err(error) => {
                    GameTools::journal(dir.as_deref(),"give_item",&json!({"kind":"give_item_outcome","request":request,"result":{"given":false,"native_submitted":false,"refusal":error}}),0)
                        .map_err(|failure| format!("inventory not submitted; refusal journal failed: {failure}"))?;
                    Err(error)
                }
            }
        })();
        if let Err(error) = result { let _ = tx.send(Err(error)); }
    });
    rx.recv_timeout(WAIT).map_err(|_| -> String {
        let cancelled = pending.cancel();
        if dry {
            "give_item dry read did not answer within 5 s; no inventory grant requested".into()
        } else if cancelled {
            "give_item timed out before game-thread execution; queued request cancelled without inventory submission".into()
        } else {
            "give_item execution started but did not answer within 5 s; outcome unknown; inspect journal/inventory and do not blindly retry".into()
        }
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{Header, PlayerInfo, Status};
    use enshwick::{Load, Mod, Off};
    use std::{fs, sync::Mutex, time::Instant};

    static GAME: Mutex<Option<Game>> = Mutex::new(None);
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *GAME.lock().unwrap() = Some(Game {
                world: load.world()?,
                native_knowledge: load.knowledge(),
                clock: load.clock(),
                game_settings: None,
                voxels: load.voxels(),
                scheduler: load.scheduler()?,
                used_item: None,
                static_used_item: None,
                inventories: Some(load.inventories()?),
                item_layout: None,
                network_health: None,
                interaction_targets: None,
            });
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.give"
side = "server""#
    );

    #[test]
    fn queued_grants_cancel_before_start_and_journal_native_outcomes_after_timeout() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let dir = root.join("target").join(format!(
            "give-audit-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join(write_gate::FLAG), []).unwrap();
        let player = |key, entity, label: &str| {
            let mut name = [0; 128];
            name[..label.len()].copy_from_slice(label.as_bytes());
            PlayerInfo {
                header: Header::of::<PlayerInfo>(),
                slot: 0,
                entity: enshwick::Entity(entity),
                key: enshwick::PlayerKey::new(key),
                permissions: 0,
                name_len: u32::try_from(label.len()).unwrap(),
                name,
            }
        };
        let loader = enshwick::test::FakeHost::new()
            .world()
            .inventory_requests()
            .items()
            .workshops()
            .vanilla_items(&[(enshwick::content::Guid([7; 16]), "fixture_item")])
            .player(player(9, 7, "actor"))
            .player(player(10, 8, "actor"))
            .player(player(11, 9, "9"))
            .install();
        // SAFETY: exports are generated by this module's test probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let game = GAME.lock().unwrap().unwrap();
        let args = json!({"player":"10","item":"fixture_item","confirm":"server-write"});
        let records = || -> Vec<Value> {
            fs::read_to_string(dir.join("journal.jsonl"))
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        };
        let drive = |worker: &std::thread::JoinHandle<Result<Value, String>>,
                     stop_on_inventory: bool| {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished()
                && (!stop_on_inventory || loader.inventory_requests().is_empty())
            {
                assert!(
                    Instant::now() < deadline,
                    "fake game-thread work did not settle"
                );
                let _ = loader.tick(Duration::from_millis(50));
                std::thread::sleep(Duration::from_millis(1));
            }
        };
        for selector in ["actor", "9"] {
            let mut ambiguous = args.clone();
            ambiguous["player"] = json!(selector);
            let ambiguous_dir = dir.clone();
            let worker = std::thread::spawn(move || {
                call(game, Some(ambiguous_dir), ambiguous.as_object().unwrap())
            });
            drive(&worker, false);
            assert!(worker.join().unwrap().unwrap_err().contains("ambiguous"));
            assert!(loader.inventory_requests().is_empty() && !dir.join("journal.jsonl").exists());
        }
        let mut dry = args.clone();
        dry["dry"] = json!(true);
        let worker = std::thread::spawn(move || call(game, None, dry.as_object().unwrap()));
        drive(&worker, false);
        assert_eq!(worker.join().unwrap().unwrap()["dry"], true);
        assert!(loader.inventory_requests().is_empty() && !dir.join("journal.jsonl").exists());

        fs::create_dir(dir.join("journal.jsonl")).unwrap();
        let bad_dir = dir.clone();
        let bad_args = args.clone();
        let worker =
            std::thread::spawn(move || call(game, Some(bad_dir), bad_args.as_object().unwrap()));
        drive(&worker, false);
        assert!(worker.join().unwrap().is_err());
        assert!(loader.inventory_requests().is_empty());
        fs::remove_dir(dir.join("journal.jsonl")).unwrap();

        let late_dir = dir.clone();
        let late_args = args.clone();
        let worker =
            std::thread::spawn(move || call(game, Some(late_dir), late_args.as_object().unwrap()));
        drive(&worker, true);
        assert_eq!(loader.inventory_requests().len(), 1);
        let intent = records();
        assert_eq!(intent.len(), 1);
        assert_eq!(intent[0]["result"]["kind"], "give_item_intent");
        assert!(
            worker
                .join()
                .unwrap()
                .unwrap_err()
                .contains("outcome unknown")
        );
        assert!(loader.finish_inventory(enshwick::abi::InventoryResult::COMMITTED, &[]));
        let settled = records();
        assert_eq!(settled.len(), 2);
        assert_eq!(settled[1]["result"]["kind"], "give_item_outcome");
        assert_eq!(
            settled[1]["result"]["request"],
            intent[0]["result"]["request"]
        );
        assert_eq!(settled[1]["result"]["result"]["given"], true);

        let cancelled_dir = dir.clone();
        let cancelled_args = args.clone();
        let worker = std::thread::spawn(move || {
            call(
                game,
                Some(cancelled_dir),
                cancelled_args.as_object().unwrap(),
            )
        });
        assert!(
            worker
                .join()
                .unwrap()
                .unwrap_err()
                .contains("queued request cancelled")
        );
        let _ = loader.tick(Duration::from_millis(50));
        assert!(loader.inventory_requests().is_empty());
        assert_eq!(records(), settled);
        fs::remove_file(dir.join(write_gate::FLAG)).unwrap();
        fs::remove_file(dir.join("journal.jsonl")).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
