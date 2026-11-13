//! Bounds and admission for geometric previews. These do not authorize writes.

use serde_json::{Map, Value, json};

/// Maximum candidate cells, before any doorway cells are omitted.
pub const MAX_CELLS: i32 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Floor,
    Wall,
    Doorway,
}

/// A checked half-metre grid specification. Cell bounds are half-open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub shape: Shape,
    pub origin: [i32; 3],
    pub size: [i32; 3],
    pub yaw: i32,
    pub material: String,
    /// Local x offset, width and height, in cells; starts at local y = 0.
    pub door: Option<[i32; 3]>,
    pub min: [i32; 3],
    pub max_exclusive: [i32; 3],
}

/// The complete input schema. All dimensions are metres on the 0.5 m grid.
#[must_use]
pub fn schema() -> Value {
    json!({
        "type":"object", "additionalProperties":false,
        "properties": {
            "kind":{"type":"string","enum":["floor","wall","doorway"]},
            "x":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
            "y":{"type":"number","minimum":0,"maximum":crate::bounds::HEIGHT,"multipleOf":0.5},
            "z":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5},
            "width":{"type":"number","minimum":0.5,"maximum":16,"multipleOf":0.5,"description":"Local x extent in metres."},
            "height":{"type":"number","minimum":0.5,"maximum":16,"multipleOf":0.5,"description":"Local y extent in metres; floor must be 0.5."},
            "depth":{"type":"number","minimum":0.5,"maximum":16,"multipleOf":0.5,"description":"Local z extent in metres; wall/doorway must be 0.5."},
            "yaw":{"type":"integer","enum":[0,90,180,270]},
            "material":{"type":"string","minLength":1,"maxLength":128,"description":"Unvalidated material reference, not a runtime voxel/item mapping."},
            "door_offset":{"type":"number","minimum":0.5,"maximum":15,"multipleOf":0.5,"description":"Door opening's local x start in metres."},
            "door_width":{"type":"number","minimum":0.5,"maximum":15,"multipleOf":0.5},
            "door_height":{"type":"number","minimum":0.5,"maximum":15,"multipleOf":0.5}
        },
        "required":["kind","x","y","z","width","height","depth","material"]
    })
}

fn grid(args: &Map<String, Value>, key: &str, min: f64, max: f64) -> Result<i32, String> {
    let n = args
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("missing number '{key}'"))?;
    if !n.is_finite() || !(min..=max).contains(&n) || (n * 2.0).fract() != 0.0 {
        return Err(format!(
            "'{key}' must be a 0.5 m grid value from {min} to {max}"
        ));
    }
    Ok((n * 2.0) as i32)
}

/// Validate every admission, dimension, allocation and world-bound constraint.
///
/// # Errors
/// Returns a diagnostic for invalid, excessive, or out-of-world input.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let shape = match args.get("kind").and_then(Value::as_str) {
        Some("floor") => Shape::Floor,
        Some("wall") => Shape::Wall,
        Some("doorway") => Shape::Doorway,
        _ => return Err("unknown building kind".into()),
    };
    let origin = [
        grid(args, "x", 0.0, crate::bounds::EDGE)?,
        grid(args, "y", 0.0, crate::bounds::HEIGHT)?,
        grid(args, "z", 0.0, crate::bounds::EDGE)?,
    ];
    let size = [
        grid(args, "width", 0.5, 16.0)?,
        grid(args, "height", 0.5, 16.0)?,
        grid(args, "depth", 0.5, 16.0)?,
    ];
    if size.iter().product::<i32>() > MAX_CELLS {
        return Err(format!("preview exceeds {MAX_CELLS} candidate cells"));
    }
    if shape == Shape::Floor && size[1] != 1 {
        return Err("floor height must be 0.5 m".into());
    }
    if shape != Shape::Floor && size[2] != 1 {
        return Err("wall and doorway depth must be 0.5 m".into());
    }
    let material = args
        .get("material")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if material.is_empty()
        || material.len() > 128
        || !material
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c))
    {
        return Err("material must be 1..128 ASCII letters, digits or _ . : -".into());
    }
    let door = if shape == Shape::Doorway {
        let offset = grid(args, "door_offset", 0.5, 15.0)?;
        let width = grid(args, "door_width", 0.5, 15.0)?;
        let height = grid(args, "door_height", 0.5, 15.0)?;
        if offset + width >= size[0] || height >= size[1] {
            return Err(
                "doorway must leave at least one cell on both sides and above the opening".into(),
            );
        }
        Some([offset, width, height])
    } else {
        if ["door_offset", "door_width", "door_height"]
            .iter()
            .any(|k| args.contains_key(*k))
        {
            return Err("door arguments are only valid for kind doorway".into());
        }
        None
    };
    let yaw = args.get("yaw").and_then(Value::as_i64).unwrap_or(0) as i32;
    let [w, h, d] = size;
    let (offset, extent) = match yaw {
        0 => ([0, 0, 0], [w, h, d]),
        90 => ([-d, 0, 0], [d, h, w]),
        180 => ([-w, 0, -d], [w, h, d]),
        270 => ([0, 0, -w], [d, h, w]),
        _ => return Err("yaw must be 0, 90, 180 or 270".into()),
    };
    let min = std::array::from_fn(|i| origin[i] + offset[i]);
    let max_exclusive = std::array::from_fn(|i| min[i] + extent[i]);
    if !crate::bounds::in_world(min.map(|n| f64::from(n) * 0.5))
        || !crate::bounds::in_world(max_exclusive.map(|n| f64::from(n) * 0.5))
    {
        return Err("rotated footprint or full cell extent is outside world bounds".into());
    }
    Ok(Spec {
        shape,
        origin,
        size,
        yaw,
        material: material.to_owned(),
        door,
        min,
        max_exclusive,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Map<String, Value> {
        json!({"kind":"wall","x":100,"y":10,"z":100,"width":4,"height":3,"depth":0.5,"material":"material:stone"}).as_object().unwrap().clone()
    }
    #[test]
    fn dimensions_alignment_and_unknown_args_are_refused() {
        for (key, value) in [
            ("width", json!(-1)),
            ("width", json!(16.5)),
            ("height", json!(0)),
            ("depth", json!(1)),
            ("x", json!(1.25)),
            ("width", json!("4")),
            ("write", json!(true)),
            ("yaw", json!(45)),
        ] {
            let mut a = base();
            a.insert(key.into(), value);
            assert!(check(&a).is_err(), "{a:?}");
        }
    }
    #[test]
    fn material_references_are_bounded_and_not_paths() {
        for value in [
            "".to_owned(),
            "x".repeat(129),
            "../file".to_owned(),
            "stone\nwrite".to_owned(),
        ] {
            let mut a = base();
            a.insert("material".into(), json!(value));
            assert!(check(&a).is_err());
        }
    }
    #[test]
    fn bounds_include_full_rotated_cells() {
        let mut a = base();
        a.insert("x".into(), json!(0));
        a.insert("yaw".into(), json!(90));
        assert!(check(&a).is_err());
        a.insert("yaw".into(), json!(0));
        a.insert("y".into(), json!(1024));
        assert!(check(&a).is_err());
        a.insert("y".into(), json!(1021));
        assert!(check(&a).is_ok());
    }
    #[test]
    fn candidate_cells_are_capped_before_allocation() {
        let mut a = base();
        a.insert("width".into(), json!(16));
        a.insert("height".into(), json!(16));
        assert!(check(&a).is_ok());
        a.insert("kind".into(), json!("floor"));
        a.insert("height".into(), json!(0.5));
        a.insert("depth".into(), json!(16));
        assert!(check(&a).is_ok());
        a.insert("height".into(), json!(16));
        assert!(check(&a).unwrap_err().contains("candidate"));
    }
    #[test]
    fn doorway_requires_complete_opening_and_frame() {
        let mut a = base();
        a.insert("kind".into(), json!("doorway"));
        assert!(check(&a).is_err());
        for (key, n) in [
            ("door_offset", 1.0),
            ("door_width", 1.5),
            ("door_height", 2.0),
        ] {
            a.insert(key.into(), json!(n));
        }
        assert!(check(&a).is_ok());
        a.insert("door_width".into(), json!(3));
        assert!(check(&a).is_err());
        a.insert("kind".into(), json!("wall"));
        assert!(check(&a).is_err());
    }
}
