//! Admission for restoring a sole living server actor to its observed maximum.
use serde_json::{Map, Value, json};

#[derive(Clone, Debug)]
pub struct Spec {
    pub player: String,
    pub dry: bool,
    pub preview: Option<String>,
    pub confirm: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "player":{"type":"string","minLength":1,"maxLength":128},
        "dry":{"type":"boolean","default":true},
        "preview":{"type":"string","minLength":1,"maxLength":1024},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]}
    },"required":["player"]})
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

/// # Errors
/// Refuses recovery while the server has any roster other than one connected actor.
pub fn check_roster(connected: usize) -> Result<(), String> {
    if connected != 1 {
        return Err("health recovery requires a sole connected player".into());
    }
    Ok(())
}

/// # Errors
/// Refuses unknown/malformed arguments and real requests without a bounded preview.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let player = args["player"].as_str().unwrap();
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let preview = args.get("preview").and_then(Value::as_str);
    if !text(player, 128) || preview.is_some_and(|p| !text(p, 1024)) || (!dry && preview.is_none())
    {
        return Err(
            "health recovery requires a bounded player selector and real calls require preview"
                .into(),
        );
    }
    Ok(Spec {
        player: player.into(),
        dry,
        preview: preview.map(str::to_owned),
        confirm: args
            .get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub session: String,
    pub generation: u64,
    pub account: u64,
    pub entity: u32,
    pub current: i32,
    pub max: i32,
}

/// # Errors
/// Refuses missing actor identity, invalid meters or nonpositive health. Does not imply safety.
pub fn preview(context: &Context) -> Result<String, String> {
    if !text(&context.session, 128)
        || context.account == 0
        || context.entity == 0
        || context.current <= 0
        || context.max <= 0
        || context.current > context.max
    {
        return Err("health recovery requires a living actor with valid observed health".into());
    }
    Ok(
        json!({"session":context.session,"generation":context.generation,
        "account":context.account.to_string(),"entity":context.entity,
        "current":context.current,"max":context.max})
        .to_string(),
    )
}

/// # Errors
/// Refuses a different session, actor, lifecycle generation or health observation.
pub fn check_preview(token: &str, context: &Context) -> Result<(), String> {
    if token != preview(context)? {
        return Err("health recovery preview no longer matches current actor context".into());
    }
    Ok(())
}

#[must_use]
pub fn restored(before: &Context, after: &Context) -> bool {
    after.session == before.session
        && after.generation == before.generation
        && after.account == before.account
        && after.entity == before.entity
        && after.max == before.max
        && after.current == before.max
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            session: "session".into(),
            generation: 1,
            account: 2,
            entity: 3,
            current: 17,
            max: 100,
        }
    }
    #[test]
    fn strict_requests_default_to_dry() {
        assert!(check_roster(1).is_ok());
        assert!(check_roster(0).is_err());
        assert!(check_roster(2).is_err());
        assert!(
            check(json!({"player":"actor"}).as_object().unwrap())
                .unwrap()
                .dry
        );
        for bad in [
            json!({}),
            json!({"player":" "}),
            json!({"player":"a\n"}),
            json!({"player":"a","dry":false}),
            json!({"player":"a","amount":100}),
            json!({"player":"a","dry":"false"}),
            json!({"player":"a","preview":"x".repeat(1025)}),
        ] {
            assert!(check(bad.as_object().unwrap()).is_err());
        }
    }
    #[test]
    fn preview_admits_only_valid_living_health_and_exact_actor_context() {
        let original = context();
        let token = preview(&original).unwrap();
        check_preview(&token, &original).unwrap();
        for changed in [
            Context {
                session: "other".into(),
                ..original.clone()
            },
            Context {
                generation: 2,
                ..original.clone()
            },
            Context {
                account: 4,
                ..original.clone()
            },
            Context {
                entity: 4,
                ..original.clone()
            },
            Context {
                current: 18,
                ..original.clone()
            },
            Context {
                max: 101,
                ..original.clone()
            },
        ] {
            assert!(check_preview(&token, &changed).is_err());
        }
        for (current, max) in [(0, 100), (-1, 100), (101, 100), (1, 0), (1, -1)] {
            assert!(
                preview(&Context {
                    current,
                    max,
                    ..original.clone()
                })
                .is_err()
            );
        }
        assert!(restored(
            &original,
            &Context {
                current: 100,
                ..original.clone()
            }
        ));
        assert!(!restored(
            &original,
            &Context {
                current: 100,
                entity: 4,
                ..original.clone()
            }
        ));
        assert!(!restored(
            &original,
            &Context {
                current: 100,
                max: 101,
                ..original.clone()
            }
        ));
        assert!(!restored(&original, &original));
    }
}
