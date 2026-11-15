//! Bounded read admission; knowledge observations never authorize progression writes.
use serde_json::{Map, Value, json};

#[derive(Debug)]
pub struct Spec {
    pub ids: Vec<u32>,
    pub player: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    let mut schema = crate::item_lookup::schema();
    schema["properties"]["player"] =
        crate::interaction_target::schema()["properties"]["player"].clone();
    schema
}

/// Refuses unbounded/duplicate IDs, invalid actor text and all write fields.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let ids = crate::item_lookup::check(json!({"ids":args["ids"]}).as_object().unwrap())?;
    let player = args
        .get("player")
        .map(|value| crate::interaction_target::check(json!({"player":value}).as_object().unwrap()))
        .transpose()?;
    Ok(Spec { ids, player })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_ids_preserve_order_and_optional_query_actor() {
        let world = check(json!({"ids":[7,2,4294967295_u64]}).as_object().unwrap()).unwrap();
        assert_eq!(world.ids, vec![7, 2, u32::MAX]);
        assert!(world.player.is_none());
        assert_eq!(
            check(
                json!({"ids":[1],"player":"Researcher"})
                    .as_object()
                    .unwrap()
            )
            .unwrap()
            .player
            .as_deref(),
            Some("Researcher")
        );
        assert!(check(json!({"ids":[1,2,3,4,5,6,7,8]}).as_object().unwrap()).is_ok());
    }
    #[test]
    fn malformed_or_write_requests_never_reach_native_reads() {
        for value in [
            json!({}),
            json!({"ids":[]}),
            json!({"ids":[1,1]}),
            json!({"ids":[0]}),
            json!({"ids":[4294967296_u64]}),
            json!({"ids":[-1]}),
            json!({"ids":[1.5]}),
            json!({"ids":["1"]}),
            json!({"ids":[1,2,3,4,5,6,7,8,9]}),
            json!({"ids":[1],"player":" padded"}),
            json!({"ids":[1],"player":"bad\nname"}),
            json!({"ids":[1],"player":"界".repeat(43)}),
            json!({"ids":[1],"confirm":"server-write"}),
            json!({"ids":[1],"grant":true}),
            json!({"ids":[1],"entity":1}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
    }
}
