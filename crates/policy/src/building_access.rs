//! Read-only admission for explicit site boxes, never placement footprints.
use serde_json::{Map, Value, json};

#[derive(Debug)]
pub struct Spec {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub player: Option<String>,
}

#[must_use]
pub fn matches_player(selector: &str, name: &str, account: u64) -> bool {
    name.to_lowercase() == selector.to_lowercase() || account.to_string() == selector
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "x1":{"type":"number"},"y1":{"type":"number"},"z1":{"type":"number"},
        "x2":{"type":"number"},"y2":{"type":"number"},"z2":{"type":"number"},
        "player":{"type":"string","minLength":1,"maxLength":128}},
        "required":["x1","y1","z1","x2","y2","z2"]})
}

/// Refuses reversed, degenerate, oversized or out-of-world boxes and invalid selectors.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let min = ["x1", "y1", "z1"].map(|k| args[k].as_f64().unwrap_or(f64::NAN));
    let max = ["x2", "y2", "z2"].map(|k| args[k].as_f64().unwrap_or(f64::NAN));
    if !crate::bounds::in_world(min)
        || !crate::bounds::in_world(max)
        || (0..3).any(|i| max[i] <= min[i] || max[i] - min[i] > 64.0)
    {
        return Err("site box requires ordered finite world corners with each side greater than zero and at most 64 metres".into());
    }
    let player = args
        .get("player")
        .map(|v| {
            let s = v.as_str().ok_or("player must be a string")?;
            if s.is_empty() || s.len() > 128 || s.trim() != s || s.chars().any(char::is_control) {
                return Err(
                    "player requires 1..128 UTF-8 bytes without surrounding whitespace or controls",
                );
            }
            Ok(s.to_owned())
        })
        .transpose()?;
    Ok(Spec { min, max, player })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_name_case_matching_preserves_ambiguous_roster_refusal() {
        assert!(matches_player("éMILE", "Émile", 7));
        assert!(!matches_player("Emile", "Émile", 7));
        let roster = [("Researcher", 7), ("RESEARCHER", 8)];
        assert!(
            crate::inspection::unique_player(
                roster
                    .into_iter()
                    .filter(|(name, key)| matches_player("Researcher", name, *key))
            )
            .is_err()
        );
        let roster = [("8", 7), ("Other", 8)];
        assert!(
            crate::inspection::unique_player(
                roster
                    .into_iter()
                    .filter(|(name, key)| matches_player("8", name, *key))
            )
            .is_err()
        );
        assert!(matches_player("7", "Unrelated", 7));
    }
    fn box_args() -> Value {
        json!({"x1":1,"y1":2,"z1":3,"x2":65,"y2":3,"z2":4})
    }
    #[test]
    fn bounded_boxes_and_optional_selector_are_strict() {
        assert!(check(box_args().as_object().unwrap()).is_ok());
        for (key, value) in [
            ("x2", json!(66)),
            ("x2", json!(1)),
            ("y2", json!(0)),
            ("x1", json!(-1)),
            ("z2", json!(10241)),
            ("dry", json!(true)),
            ("player", json!(" ")),
            ("player", json!("a\n")),
            ("player", json!("界".repeat(43))),
        ] {
            let mut args = box_args();
            args[key] = value;
            assert!(check(args.as_object().unwrap()).is_err(), "{key}");
        }
        let mut args = box_args();
        args["player"] = json!("known actor");
        assert_eq!(
            check(args.as_object().unwrap()).unwrap().player.as_deref(),
            Some("known actor")
        );
    }
}
