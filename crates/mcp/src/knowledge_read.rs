//! Raw scoped native knowledge, never progression grants or crafting eligibility.
use enshwick::knowledge::{Knowledge, KnowledgeId, Known};
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "knowledge_read",
        description: "Read at most eight explicit native knowledge IDs on the server game thread. Without player, query world values; optional unique connected player queries that actor and world-scoped values. Preserve raw type/scope/value/expected flags; unavailable stays unknown, not zero/false. No progression writes, current crafting eligibility or client identity mapping.",
        schema: policy::knowledge_read::schema(),
    }
}

fn rows(ids: &[u32], mut get: impl FnMut(u32) -> Option<Known>) -> Vec<Value> {
    ids.iter().map(|&id| match get(id) {
        Some(known)=>json!({"id":id,"state":"observed","type":format!("{:?}",known.r#type),
            "scope":format!("{:?}",known.scope),"value":known.value,"expected_flags":known.expected_flags}),
        None=>json!({"id":id,"state":"unknown","type":null,"scope":null,"value":null,"expected_flags":null}),
    }).collect()
}

fn observe(knowledge: Knowledge, ids: &[u32], player: Option<enshwick::Entity>) -> Vec<Value> {
    rows(ids, |id| match player {
        Some(entity) => knowledge.get(entity, KnowledgeId::new(id)),
        None => knowledge.world(KnowledgeId::new(id)),
    })
}

pub(crate) fn read(
    game: crate::tools::Game,
    spec: policy::knowledge_read::Spec,
) -> Result<Value, String> {
    let player =
        spec.player
            .as_deref()
            .map(|selector| {
                policy::inspection::unique_player(game.world.players().into_iter().filter(|p| {
                    policy::building_access::matches_player(selector, &p.name, p.key.0)
                }))
            })
            .transpose()?;
    Ok(
        json!({"world_key":game.world.key(),"query":if player.is_some(){"player"}else{"world"},
        "player":player.as_ref().map(|p|json!({"key":p.key.0.to_string(),"entity":p.entity.0})),
        "knowledge":observe(game.native_knowledge,&spec.ids,player.as_ref().map(|p|p.entity)),
        "world_mutated":false,"crafting_eligibility_verified":false,
        "limitations":["unknown includes unknown ID, unavailable read or wrong scope; it is not observed false/zero",
            "player queries may return shared world-scoped values; row scope is authoritative",
            "raw knowledge and recipe flags do not establish current station, stock, unlock or crafting eligibility"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Entity, Load, Mod, Off};
    use std::{sync::Mutex, time::Duration};
    static SEEN: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let knowledge = load.knowledge();
            load.on_tick(Duration::from_millis(100), move |_| {
                SEEN.lock()
                    .unwrap()
                    .push(json!({"world":observe(knowledge,&[7,2,3,9],None),
                    "player":observe(knowledge,&[7,2,3,9],Some(Entity(40))),
                    "other":observe(knowledge,&[2],Some(Entity(41)))}));
            })?;
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.native-knowledge"
side = "server""#
    );
    #[test]
    fn native_scopes_zero_flags_and_missing_are_distinct_without_grants() {
        use enshwick::abi::{Header, KnowledgeInfo, Status};
        let loader = enshwick::test::FakeHost::new().install();
        // SAFETY: these exports belong to this test module's Probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let info = |id, kind, scope, value, flags| KnowledgeInfo {
            header: Header::of::<KnowledgeInfo>(),
            id: KnowledgeId::new(id),
            kind,
            scope,
            reserved: 0,
            value,
            flags,
        };
        loader.set_knowledge(
            &[
                (
                    None,
                    info(7, KnowledgeInfo::BOOL, KnowledgeInfo::WORLD, 0, 0),
                ),
                (
                    Some(Entity(40)),
                    info(2, KnowledgeInfo::FLAGS, KnowledgeInfo::PLAYER, 1, 3),
                ),
                (
                    None,
                    info(3, KnowledgeInfo::COUNTER, KnowledgeInfo::WORLD, 12, 0),
                ),
            ],
            &[],
        );
        let _ = loader.tick(Duration::from_millis(100));
        let seen = SEEN.lock().unwrap();
        let report = &seen[0];
        assert_eq!(report["world"][0]["state"], "observed");
        assert_eq!(report["world"][0]["type"], "Bool");
        assert_eq!(report["world"][0]["value"], 0);
        assert_eq!(report["world"][1]["state"], "unknown");
        assert!(report["world"][1]["value"].is_null());
        assert_eq!(report["player"][0]["scope"], "World");
        assert_eq!(report["player"][1]["scope"], "Player");
        assert_eq!(report["player"][1]["expected_flags"], 3);
        assert_eq!(report["player"][1]["value"], 1);
        assert_eq!(report["player"][2]["value"], 12);
        assert_eq!(report["player"][3]["state"], "unknown");
        assert_eq!(report["other"][0]["state"], "unknown");
        assert!(loader.grants().is_empty());
    }
    #[test]
    fn unknown_rows_keep_requested_order_and_null_metadata() {
        let mut called = Vec::new();
        let report = rows(&[7, 2], |id| {
            called.push(id);
            None
        });
        assert_eq!(called, vec![7, 2]);
        assert_eq!(
            report
                .iter()
                .map(|v| v["id"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![7, 2]
        );
        assert!(report.iter().all(|v| v["value"].is_null()
            && v["scope"].is_null()
            && v["type"].is_null()
            && v["expected_flags"].is_null()));
    }
}
