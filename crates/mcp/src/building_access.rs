//! Explicit-box permission observations, not item placement or save verification.
use enshwick::world::BuildRefusal;
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_access",
        description: "Read-only native permission/persistence observations for explicit ordered site corners, each side >0 and <=64 metres. Optional player name/account key must match exactly one connected actor; omitted player leaves build rights unknown. No item footprint, collision, terrain contact, save/reload proof or write authority. Errors remain unknown, not false.",
        schema: policy::building_access::schema(),
    }
}

pub(crate) fn persistence(result: Result<bool, String>) -> Value {
    match result {
        Ok(value) => json!({"state":"observed","kept_by_zone_rule":value}),
        Err(error) => json!({"state":"unknown","kept_by_zone_rule":null,"error":error}),
    }
}

pub(crate) fn rights(result: Option<Result<(), BuildRefusal>>) -> Value {
    match result {
        None => json!({"state":"not_requested","allowed":null}),
        Some(Ok(())) => json!({"state":"observed","allowed":true}),
        Some(Err(BuildRefusal::Unchecked(error))) => {
            json!({"state":"unknown","allowed":null,"error":error.to_string()})
        }
        Some(Err(error)) => json!({"state":"refused","allowed":false,"reason":error.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{WorldError, world::PlayerPermissions};
    #[test]
    fn missing_and_unchecked_never_become_negative_observations() {
        assert_eq!(persistence(Ok(false))["kept_by_zone_rule"], false);
        assert!(persistence(Err("unavailable".into()))["kept_by_zone_rule"].is_null());
        assert_eq!(rights(Some(Ok(())))["allowed"], true);
        assert_eq!(
            rights(Some(Err(BuildRefusal::NoPermission(
                PlayerPermissions::CanEditBase
            ))))["allowed"],
            false
        );
        assert_eq!(
            rights(Some(Err(BuildRefusal::Unchecked(WorldError::Missing))))["state"],
            "unknown"
        );
        assert!(rights(None)["allowed"].is_null());
    }
}
