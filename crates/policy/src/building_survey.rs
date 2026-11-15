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
    pub foundation: Option<Foundation>,
}

/// One finite column, with an exclusive upper boundary in half-metre cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foundation {
    pub upper: [u32; 3],
    pub max_cells: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnCell {
    Air,
    Terrain(u8),
    Obstruction,
}

/// Return only the Air span before the first occupied terrain cell.
///
/// # Errors
/// Refuses missing terrain, empty spans, obstructions and excessive depth.
pub fn foundation_span(column: &[ColumnCell], max_cells: usize) -> Result<usize, String> {
    if !(1..=15).contains(&max_cells) {
        return Err("foundation max_cells must be from 1 to 15".into());
    }
    for (depth, cell) in column.iter().take(max_cells + 1).enumerate() {
        match cell {
            ColumnCell::Terrain(fill) if *fill > 0 && depth > 0 => return Ok(depth),
            ColumnCell::Air if depth < max_cells => (),
            _ => return Err("foundation requires a bounded Air span ending before occupied terrain; obstruction, empty span or depth limit".into()),
        }
    }
    Err("foundation terrain stop is absent from the captured column".into())
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
            "player":{"type":"string","minLength":1,"maxLength":128,"description":"Optional connected-player name or account key; at most 128 UTF-8 bytes, no controls. Checks build rights only, not survival eligibility."},
            "foundation":{"type":"object","additionalProperties":false,"description":"Optional single Air-column pier preview, stopping before occupied terrain. Does not measure the rendered terrain surface or authorize edits.","properties":{
                "x":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
                "z":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
                "upper_y":{"type":"number","minimum":0.5,"maximum":crate::bounds::HEIGHT,"multipleOf":0.5},
                "max_cells":{"type":"integer","minimum":1,"maximum":15}
            },"required":["x","z","upper_y","max_cells"]}
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
    let schema = schema();
    crate::args::check(&schema, args)?;
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
    let foundation = args
        .get("foundation")
        .map(|value| -> Result<Foundation, String> {
            let values = value.as_object().ok_or("foundation must be an object")?;
            crate::args::check(&schema["properties"]["foundation"], values)?;
            let upper = [
                grid(values, "x", 0.0, crate::bounds::EDGE)?,
                grid(values, "upper_y", 0.5, crate::bounds::HEIGHT)?,
                grid(values, "z", 0.0, crate::bounds::EDGE)?,
            ];
            if upper[0] < min[0]
                || upper[0] >= max_exclusive[0]
                || upper[2] < min[2]
                || upper[2] >= max_exclusive[2]
                || upper[1] <= min[1]
                || upper[1] > max_exclusive[1]
            {
                return Err("foundation column and upper boundary must be inside survey".into());
            }
            let max_cells = values
                .get("max_cells")
                .and_then(Value::as_u64)
                .filter(|n| (1..=15).contains(n))
                .ok_or("foundation max_cells must be from 1 to 15")?
                as usize;
            Ok(Foundation { upper, max_cells })
        })
        .transpose()?;
    Ok(Spec {
        min,
        size,
        max_exclusive,
        cell_count,
        player,
        foundation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_stops_before_occupied_terrain_without_using_fill_as_height() {
        use ColumnCell::{Air, Obstruction, Terrain};
        for fill in [1, 67, 255] {
            assert_eq!(foundation_span(&[Air, Air, Terrain(fill)], 2).unwrap(), 2);
        }
        assert!(foundation_span(&[Terrain(67)], 2).is_err());
        assert!(foundation_span(&[Air, Terrain(0)], 2).is_err());
        assert!(foundation_span(&[Air, Obstruction, Terrain(67)], 2).is_err());
        assert!(foundation_span(&[Air, Air], 2).is_err());
        assert!(foundation_span(&[Air, Air, Air, Terrain(67)], 2).is_err());
        assert!(foundation_span(&[Air, Terrain(67)], 0).is_err());
    }

    #[test]
    fn foundation_request_must_fit_survey_and_refuse_unknown_nested_fields() {
        let mut args = base();
        args.insert(
            "foundation".into(),
            json!({"x":100,"z":100,"upper_y":11,"max_cells":2}),
        );
        let spec = check(&args).unwrap();
        assert_eq!(spec.foundation.unwrap().upper, [200, 22, 200]);
        for value in [
            json!({"x":102,"z":100,"upper_y":11,"max_cells":2}),
            json!({"x":100,"z":100,"upper_y":10,"max_cells":2}),
            json!({"x":100,"z":100,"upper_y":11.5,"max_cells":2}),
            json!({"x":100.25,"z":100,"upper_y":11,"max_cells":2}),
            json!({"x":100,"z":100,"upper_y":11,"max_cells":0}),
            json!({"x":100,"z":100,"upper_y":11,"max_cells":16}),
            json!({"x":100,"z":100,"upper_y":11,"max_cells":2,"write":true}),
        ] {
            args.insert("foundation".into(), value);
            assert!(check(&args).is_err(), "{args:?}");
        }
    }

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
