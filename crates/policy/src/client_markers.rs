//! Bounded transient client map annotation; no server identity or world authority.
use serde_json::{Map, Value, json};

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub position: [f64; 3],
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Spec {
    pub markers: Option<Vec<Marker>>,
    pub dry: bool,
    pub preview: Option<String>,
    pub confirm: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "markers":{"type":"array","maxItems":1,"items":marker_schema()},
        "dry":{"type":"boolean","default":true},
        "preview":{"type":"string","minLength":1,"maxLength":1024},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]}
    }})
}

fn marker_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "x":{"type":"number"},"y":{"type":"number"},"z":{"type":"number"},
        "label":{"type":"string","minLength":1,"maxLength":64}
    },"required":["x","y","z","label"]})
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

/// # Errors
/// Refuses unknown fields, unbounded labels/positions, multiple markers and unpreviewed writes.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let preview = args.get("preview").and_then(Value::as_str);
    let confirm = args.get("confirm").and_then(Value::as_str);
    let markers=args.get("markers").map(|v| -> Result<Vec<Marker>,String> {
        let values=v.as_array().ok_or("markers must be an array")?;
        if values.len()>1 {return Err("at most one mod-owned marker is admitted".into());}
        values.iter().map(|value| {
            let fields=value.as_object().ok_or("marker must be an object")?;
            crate::args::check(&marker_schema(),fields)?;
            let position=crate::args::point(fields,"x","y","z");
            let label=fields["label"].as_str().unwrap();
            if !crate::bounds::in_world(position) || !text(label,64) {
                return Err("marker requires finite world bounds and 1..64 UTF-8 label bytes without controls".into());
            }
            Ok(Marker{position,label:label.into()})
        }).collect()
    }).transpose()?;
    if markers.is_none() && args.keys().any(|key| key != "markers") {
        return Err("cached marker status takes no write fields".into());
    }
    if preview.is_some_and(|p| !text(p, 1024)) || (!dry && preview.is_none()) {
        return Err("real marker request requires a bounded dry preview".into());
    }
    Ok(Spec {
        markers,
        dry,
        preview: preview.map(str::to_owned),
        confirm: confirm.map(str::to_owned),
    })
}

/// # Errors
/// Nonempty annotations require a fresh in-game frame; clearing still requires freshness.
pub fn context(in_game: bool, age_ms: Option<u128>, markers: &[Marker]) -> Result<(), String> {
    if age_ms.is_none_or(|age| age > 1000) || (!in_game && !markers.is_empty()) {
        return Err("marker request requires a fresh applicable client UI frame".into());
    }
    Ok(())
}

#[must_use]
pub fn encode(markers: &[Marker]) -> Value {
    json!(
        markers
            .iter()
            .map(|m| json!({"position":m.position,"label":m.label}))
            .collect::<Vec<_>>()
    )
}

#[must_use]
pub fn preview(context: &str, markers: &[Marker]) -> String {
    json!({"context":context,"markers":encode(markers)}).to_string()
}

/// # Errors
/// Refuses any change in epoch/revision or annotation since the preview.
pub fn check_preview(token: &str, context: &str, markers: &[Marker]) -> Result<(), String> {
    if token != preview(context, markers) {
        return Err("marker preview no longer matches client context/request".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn marker() -> Value {
        json!({"x":12,"y":30,"z":40,"label":"Fixture"})
    }
    #[test]
    fn status_and_dry_are_explicit_and_payloads_are_bounded() {
        assert!(
            check(json!({}).as_object().unwrap())
                .unwrap()
                .markers
                .is_none()
        );
        assert!(
            check(json!({"markers":[marker()]}).as_object().unwrap())
                .unwrap()
                .dry
        );
        for value in [
            json!({"dry":true}),
            json!({"markers":[marker(),marker()]}),
            json!({"markers":[{"x":1}]}),
            json!({"markers":[],"dry":false}),
            json!({"markers":[],"preview":"x".repeat(1025)}),
            json!({"markers":[],"clear":true}),
            json!({"markers":[],"confirm":"client-ui"}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
        for (key, value) in [
            ("label", json!("界".repeat(22))),
            ("label", json!("bad\nlabel")),
            ("label", json!(" ")),
            ("x", json!(-1)),
            ("y", json!(1025)),
            ("z", json!(10241)),
            ("type", json!(1)),
        ] {
            let mut m = marker();
            m[key] = value;
            assert!(check(json!({"markers":[m]}).as_object().unwrap()).is_err());
        }
    }
    #[test]
    fn freshness_and_exact_epoch_revision_and_request_are_required() {
        let markers = check(json!({"markers":[marker()]}).as_object().unwrap())
            .unwrap()
            .markers
            .unwrap();
        assert!(context(true, Some(1000), &markers).is_ok());
        assert!(context(false, Some(0), &[]).is_ok());
        assert!(context(false, Some(0), &markers).is_err());
        assert!(context(true, None, &markers).is_err());
        assert!(context(true, Some(1001), &markers).is_err());
        let token = preview("boot-1-2", &markers);
        assert!(check_preview(&token, "boot-1-2", &markers).is_ok());
        assert!(check_preview(&token, "boot-2-2", &markers).is_err());
        assert!(check_preview(&token, "boot-1-3", &markers).is_err());
        assert!(check_preview(&token, "boot-1-2", &[]).is_err());
    }
}
