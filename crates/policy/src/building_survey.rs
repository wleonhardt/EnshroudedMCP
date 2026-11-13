//! Admission for bounded, read-only voxel surveys. This never authorizes edits.

use serde_json::{Map, Value, json};

pub const MAX_SIDE_M: f64 = 8.0;
pub const MAX_CELLS: usize = 4096;

/// A validated box of complete half-metre cells with half-open bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub min: [u32; 3],
    pub size: [u32; 3],
    pub max_exclusive: [u32; 3],
    pub cell_count: usize,
    pub player: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    json!({
        "type":"object", "additionalProperties":false,
        "properties":{
            "x":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
            "y":{"type":"number","minimum":0,"maximum":crate::bounds::HEIGHT,"multipleOf":0.5},
            "z":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
            "width":{"type":"number","minimum":0.5,"maximum":MAX_SIDE_M,"multipleOf":0.5},
            "height":{"type":"number","minimum":0.5,"maximum":MAX_SIDE_M,"multipleOf":0.5},
            "depth":{"type":"number","minimum":0.5,"maximum":MAX_SIDE_M,"multipleOf":0.5},
            "player":{"type":"string","minLength":1,"maxLength":128,"description":"Optional connected-player name or account key; at most 128 UTF-8 bytes, no controls. Checks build rights only, not survival eligibility."}
        },
        "required":["x","y","z","width","height","depth"]
    })
}

fn grid(args: &Map<String, Value>, key: &str, min: f64, max: f64) -> Result<u32, String> {
    let n = args
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("missing number '{key}'"))?;
    if !n.is_finite() || !(min..=max).contains(&n) || (n * 2.0).fract() != 0.0 {
        return Err(format!(
            "'{key}' must be a 0.5 m grid value from {min} to {max}"
        ));
    }
    Ok((n * 2.0) as u32)
}

/// Validate fields, grid alignment, full-cell world extents and allocation size.
///
/// # Errors
/// Refuses malformed selectors, unknown arguments and excessive/outside boxes.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let min = [
        grid(args, "x", 0.0, crate::bounds::EDGE)?,
        grid(args, "y", 0.0, crate::bounds::HEIGHT)?,
        grid(args, "z", 0.0, crate::bounds::EDGE)?,
    ];
    let size = [
        grid(args, "width", 0.5, MAX_SIDE_M)?,
        grid(args, "height", 0.5, MAX_SIDE_M)?,
        grid(args, "depth", 0.5, MAX_SIDE_M)?,
    ];
    let cell_count = size
        .iter()
        .try_fold(1usize, |count, &axis| count.checked_mul(axis as usize))
        .filter(|&count| count <= MAX_CELLS)
        .ok_or_else(|| format!("survey exceeds {MAX_CELLS} cells"))?;
    let max_exclusive = std::array::from_fn(|i| min[i] + size[i]);
    if !crate::bounds::in_world(max_exclusive.map(|n| f64::from(n) * 0.5)) {
        return Err("complete survey cells must lie inside world bounds".into());
    }
    let player = args
        .get("player")
        .map(|value| {
            value
                .as_str()
                .filter(|s| {
                    !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control)
                })
                .map(str::to_owned)
                .ok_or_else(|| {
                    "player must contain 1 to 128 UTF-8 bytes without control characters".to_owned()
                })
        })
        .transpose()?;
    Ok(Spec {
        min,
        size,
        max_exclusive,
        cell_count,
        player,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Map<String, Value> {
        json!({"x":100,"y":10,"z":100,"width":2,"height":1,"depth":0.5})
            .as_object()
            .unwrap()
            .clone()
    }

    #[test]
    fn box_is_complete_half_open_cells() {
        let spec = check(&base()).unwrap();
        assert_eq!(spec.min, [200, 20, 200]);
        assert_eq!(spec.size, [4, 2, 1]);
        assert_eq!(spec.max_exclusive, [204, 22, 201]);
        assert_eq!(spec.cell_count, 8);
        assert_eq!(spec.player, None);
    }

    #[test]
    fn largest_box_has_exact_bounded_allocation() {
        let mut args = base();
        for key in ["width", "height", "depth"] {
            args.insert(key.into(), json!(8));
        }
        let spec = check(&args).unwrap();
        assert_eq!(spec.cell_count, MAX_CELLS);
        assert_eq!(spec.size, [16; 3]);
    }

    #[test]
    fn malformed_dimensions_grid_types_and_unknown_fields_are_refused() {
        for (key, value) in [
            ("width", json!(0)),
            ("height", json!(-0.5)),
            ("depth", json!(8.5)),
            ("width", json!(0.75)),
            ("x", json!(100.25)),
            ("y", json!(-0.5)),
            ("z", json!(10240.5)),
            ("width", json!("2")),
            ("depth", json!(true)),
            ("height", Value::Null),
            ("write", json!(true)),
            ("dry", json!(true)),
        ] {
            let mut args = base();
            args.insert(key.into(), value);
            assert!(check(&args).is_err(), "{args:?}");
        }
        let mut args = base();
        args.remove("depth");
        assert!(check(&args).is_err());
    }

    #[test]
    fn full_extents_may_touch_but_never_cross_world_edges() {
        let mut args = base();
        for (key, value) in [
            ("x", 10239.5),
            ("y", 1023.5),
            ("z", 10239.5),
            ("width", 0.5),
            ("height", 0.5),
            ("depth", 0.5),
        ] {
            args.insert(key.into(), json!(value));
        }
        assert_eq!(check(&args).unwrap().max_exclusive, [20480, 2048, 20480]);
        for (key, value) in [("x", 10240.0), ("y", 1024.0), ("z", 10240.0)] {
            let mut outside = args.clone();
            outside.insert(key.into(), json!(value));
            assert!(check(&outside).is_err());
        }
    }

    #[test]
    fn selectors_are_optional_byte_bounded_and_never_controls() {
        for value in [
            json!(""),
            json!("   "),
            json!("a".repeat(129)),
            json!("é".repeat(65)),
            json!("name\n"),
            json!("name\u{0085}"),
            json!("name\0"),
            json!(7),
            Value::Null,
        ] {
            let mut args = base();
            args.insert("player".into(), value);
            assert!(check(&args).is_err());
        }
        for selector in [
            "Example".to_owned(),
            "123456789".to_owned(),
            "a".repeat(128),
            "é".repeat(64),
        ] {
            let mut args = base();
            args.insert("player".into(), json!(selector));
            assert_eq!(
                check(&args).unwrap().player.as_deref(),
                Some(selector.as_str())
            );
        }
    }
}
