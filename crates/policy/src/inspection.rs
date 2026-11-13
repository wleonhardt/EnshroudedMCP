//! Bounded selectors for read-only runtime inspection.

use serde_json::{Map, Value};

use crate::{args, bounds};

pub const HEALTH_RADIUS: f64 = 16.0;
pub const HEALTH_LIMIT: usize = 20;
pub const SLOT_LIMIT: usize = 128;
pub const SLOT_OFFSET_MAX: usize = 65_536;

#[derive(Debug, PartialEq, Eq)]
pub enum InventoryTarget {
    Player(String),
    Entity(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryRequest {
    pub target: InventoryTarget,
    pub offset: usize,
    pub limit: usize,
}

/// Validate even when called without the MCP schema admission layer.
pub fn inventory(a: &Map<String, Value>) -> Result<InventoryRequest, String> {
    let target = match (a.get("player"), a.get("entity")) {
        (Some(player), None) => InventoryTarget::Player(selector(player, "player")?),
        (None, Some(entity)) => InventoryTarget::Entity(
            entity
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n != 0)
                .ok_or("entity must be an integer from 1 to 4294967295")?,
        ),
        _ => return Err("provide exactly one of player or entity".into()),
    };
    Ok(InventoryRequest {
        target,
        offset: integer(a, "offset", 0, 0, SLOT_OFFSET_MAX)?,
        limit: integer(a, "limit", 64, 1, SLOT_LIMIT)?,
    })
}

pub fn item(a: &Map<String, Value>) -> Result<String, String> {
    selector(a.get("item").ok_or("item is required")?, "item")
}

fn selector(value: &Value, name: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control))
        .map(str::to_owned)
        .ok_or_else(|| {
            format!("{name} must contain 1 to 128 UTF-8 bytes without control characters")
        })
}

pub fn health(a: &Map<String, Value>) -> Result<([f64; 3], f64, usize), String> {
    let p = args::point(a, "x", "y", "z");
    let radius = match a.get("radius") {
        Some(v) => v.as_f64().ok_or("radius must be a number")?,
        None => 3.0,
    };
    if !bounds::in_world(p) || !radius.is_finite() || radius <= 0.0 || radius > HEALTH_RADIUS {
        return Err("point must be inside the world and radius above 0 and at most 16 m".into());
    }
    Ok((p, radius, integer(a, "limit", 10, 1, HEALTH_LIMIT)?))
}

fn integer(
    a: &Map<String, Value>,
    name: &str,
    default: usize,
    min: usize,
    max: usize,
) -> Result<usize, String> {
    match a.get(name) {
        None => Ok(default),
        Some(v) => v
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| (min..=max).contains(n))
            .ok_or_else(|| format!("{name} must be an integer from {min} to {max}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn inventory_requires_one_valid_selector() {
        for v in [
            json!({}),
            json!({"player":"p","entity":1}),
            json!({"entity":0}),
            json!({"entity":-1}),
            json!({"entity":4294967296_u64}),
            json!({"entity":1.5}),
            json!({"player":" "}),
            json!({"player":"a\nb"}),
            json!({"player":"x".repeat(129)}),
        ] {
            assert!(inventory(&obj(v.clone())).is_err(), "{v}");
        }
        assert_eq!(
            inventory(&obj(json!({"player":"p"}))).unwrap(),
            InventoryRequest {
                target: InventoryTarget::Player("p".into()),
                offset: 0,
                limit: 64
            }
        );
        assert_eq!(
            inventory(&obj(json!({"entity":4294967295_u64})))
                .unwrap()
                .target,
            InventoryTarget::Entity(u32::MAX)
        );
    }

    #[test]
    fn inventory_paging_never_wraps_or_silently_defaults() {
        for field in ["offset", "limit"] {
            for bad in [json!(-1), json!(1.5), json!("1"), json!(u64::MAX)] {
                let mut a = obj(json!({"entity":1}));
                a.insert(field.into(), bad);
                assert!(inventory(&a).is_err());
            }
        }
        for v in [
            json!({"entity":1,"limit":0}),
            json!({"entity":1,"limit":129}),
            json!({"entity":1,"offset":65537}),
        ] {
            assert!(inventory(&obj(v)).is_err());
        }
        assert!(inventory(&obj(json!({"entity":1,"limit":128,"offset":65536}))).is_ok());
    }

    #[test]
    fn health_bounds_are_enforced_without_schema() {
        assert!(health(&obj(json!({"x":1,"y":1,"z":1,"radius":16,"limit":20}))).is_ok());
        for v in [
            json!({"x":-1,"y":1,"z":1}),
            json!({"x":1,"y":1025,"z":1}),
            json!({"x":1,"y":1,"z":1,"radius":0}),
            json!({"x":1,"y":1,"z":1,"radius":16.1}),
            json!({"x":1,"y":1,"z":1,"limit":-1}),
            json!({"x":1,"y":1,"z":1,"limit":0}),
            json!({"x":1,"y":1,"z":1,"limit":21}),
        ] {
            assert!(health(&obj(v)).is_err());
        }
    }

    #[test]
    fn item_names_are_bounded() {
        assert!(item(&obj(json!({"item":"Craftingstation_Workbench"}))).is_ok());
        for value in [
            json!(null),
            json!(""),
            json!("x".repeat(129)),
            json!("a\0b"),
        ] {
            assert!(item(&obj(json!({"item":value}))).is_err());
        }
    }
}
