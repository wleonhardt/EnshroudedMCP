//! One server prompt target; not a terrain raycast or interaction executor.
use enshwick::{Entity, interactions::Targets};
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "player_interaction_target",
        description: "Read one uniquely selected connected server player's native interaction-prompt target. Session-only server entity, optional position/template; unavailable differs from no target. Not a terrain hit, camera raycast, collision test, client entity mapping or interaction executor. No writes.",
        schema: policy::interaction_target::schema(),
    }
}

pub(crate) fn observe(
    result: Result<Option<Entity>, String>,
    mut metadata: impl FnMut(Entity) -> (Option<[f64; 3]>, Option<[u8; 16]>),
) -> Value {
    match result {
        Err(error) => json!({"state":"unknown","target":null,"error":error}),
        Ok(None) => json!({"state":"none","target":null}),
        Ok(Some(entity)) => {
            let (position, template) = metadata(entity);
            json!({"state":"observed","target":{"entity":entity.0,"position":position,
                "template_guid":template.map(|bytes|bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>())}})
        }
    }
}

pub(crate) fn read(game: crate::tools::Game, selector: &str) -> Result<Value, String> {
    let player = policy::inspection::unique_player(
        game.world
            .players()
            .into_iter()
            .filter(|p| policy::building_access::matches_player(selector, &p.name, p.key.0)),
    )?;
    let result = game
        .interaction_targets
        .ok_or_else(|| "native interaction target reader unavailable".to_owned())
        .and_then(|targets: Targets| targets.target(player.entity).map_err(|e| e.to_string()));
    Ok(
        json!({"world_key":game.world.key(),"player":{"key":player.key.0.to_string(),"entity":player.entity.0},
        "observation":observe(result, |entity|(game.world.position(entity),game.world.template(entity))),
        "world_mutated":false,"scope":"server prompt target only; session entity ids; no terrain hit, rendered contact or interaction execution"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Load, Mod, Off};
    use std::{sync::Mutex, time::Duration};
    static OBSERVATIONS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let targets = load.interaction_targets()?;
            load.on_tick(Duration::from_millis(100), move |_| {
                OBSERVATIONS.lock().unwrap().push(observe(
                    targets.target(Entity(40)).map_err(|e| e.to_string()),
                    |_| (Some([1., 2., 3.]), Some([2; 16])),
                ));
            })?;
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.prompt-target"
side = "server""#
    );
    #[test]
    fn published_reader_observes_prompt_slot_instead_of_combat_target() {
        use enshwick::{content::EngineFacts, test::facts};
        let fake = enshwick::test::FakeHost::new()
            .world()
            .interaction_targets();
        let ty = fake.hash(facts::SELECTED_TARGETS.id()).unwrap();
        let size = fake.size(facts::SELECTED_TARGETS.id()).unwrap();
        let stride = fake.size(facts::TARGET_INFO.id()).unwrap();
        let offset = fake.offset(facts::TARGET_INFO_ENTITY.id()).unwrap();
        let mut bytes = vec![0; size];
        bytes[offset..offset + 4].copy_from_slice(&77u32.to_le_bytes());
        let loader = fake.component(Entity(40), ty, &bytes).install();
        // SAFETY: these exports belong to this test module's Probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let _ = loader.tick(Duration::from_millis(100));
        let at = stride * 2 + offset;
        bytes[at..at + 4].copy_from_slice(&512u32.to_le_bytes());
        assert!(loader.set_component(Entity(40), ty, &bytes));
        let _ = loader.tick(Duration::from_millis(100));
        let seen = OBSERVATIONS.lock().unwrap();
        assert_eq!(seen[0]["state"], "none");
        assert_eq!(seen[1]["target"]["entity"], 512);
        assert_eq!(seen[1]["target"]["position"], json!([1., 2., 3.]));
        assert_eq!(seen[1]["target"]["template_guid"], "02".repeat(16));
    }
    #[test]
    fn unknown_and_no_target_never_read_metadata_or_become_each_other() {
        assert_eq!(
            observe(Err("missing reader".into()), |_| panic!())["state"],
            "unknown"
        );
        assert_eq!(observe(Ok(None), |_| panic!())["state"], "none");
    }
    #[test]
    fn only_the_observed_target_is_inspected_and_missing_metadata_stays_null() {
        let mut inspected = Vec::new();
        let result = observe(Ok(Some(Entity(7))), |entity| {
            inspected.push(entity.0);
            (None, None)
        });
        assert_eq!(inspected, vec![7]);
        assert_eq!(result["state"], "observed");
        assert_eq!(result["target"]["entity"], 7);
        assert!(result["target"]["position"].is_null());
        assert!(result["target"]["template_guid"].is_null());
    }
}
