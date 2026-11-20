//! Bounded selectors for read-only runtime inspection.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::{args, bounds};

pub const HEALTH_RADIUS: f64 = 16.0;
pub const PROP_LIMIT: usize = 500;

/// Preserve observed rows without promoting a configured catalogue to complete reflection.
pub fn component_observations(total: usize, known_types: usize, entities: Vec<Value>) -> Value {
    json!({
        "total": total,
        "known_types": known_types,
        "entities": entities,
        "component_inventory_source": "resolved_configured_names",
        "complete_component_inventory_verified": false,
        "component_absence_verified": false
    })
}

pub fn unique_player<T>(mut matches: impl Iterator<Item = T>) -> Result<T, String> {
    let player = matches
        .next()
        .ok_or_else(|| "no connected player matches this selector".to_owned())?;
    if matches.next().is_some() {
        return Err("player selector is ambiguous; grant refused".into());
    }
    Ok(player)
}
pub const HEALTH_LIMIT: usize = 20;
pub const SLOT_LIMIT: usize = 128;
pub const SLOT_OFFSET_MAX: usize = 65_536;
pub const COUNT_SLOT_LIMIT: usize = 65_536;

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
    pub item_ids: Option<Vec<u32>>,
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
        item_ids: a
            .get("item_ids")
            .map(|ids| {
                crate::item_lookup::check(json!({"ids":ids}).as_object().expect("object literal"))
            })
            .transpose()?,
    })
}

/// Counts unique physical slots from one SDK read, without inferring eligibility
/// or completeness of the SDK's virtual-link traversal.
pub fn inventory_item_counts(
    ids: &[u32],
    slots: impl IntoIterator<Item = (u64, u32, u32)>,
) -> Result<Value, String> {
    let ids = crate::item_lookup::check(json!({"ids":ids}).as_object().expect("object literal"))?;
    let mut counts: BTreeMap<_, u64> = ids.into_iter().map(|id| (id, 0)).collect();
    let mut seen = BTreeMap::new();
    let mut aliases = 0;
    for (index, (slot, item, count)) in slots.into_iter().enumerate() {
        if index >= COUNT_SLOT_LIMIT {
            return Err("inventory item counts exceed 65536 returned slot records".into());
        }
        if let Some(previous) = seen.insert(slot, (item, count)) {
            if previous != (item, count) {
                return Err("conflicting observations for one physical inventory slot".into());
            }
            aliases += 1;
            continue;
        }
        if let Some(total) = counts.get_mut(&item) {
            // At most COUNT_SLOT_LIMIT u32 counts fit exactly in u64.
            *total += u64::from(count);
        }
    }
    Ok(json!({
        "scope":"unique_slots_returned_by_sdk",
        "inventory_completeness":"unknown",
        "crafting_eligibility":"unknown",
        "aliases_removed":aliases,
        "counts":counts.into_iter().map(|(item,count)| json!({"item_id":item,"observed_count":count})).collect::<Vec<_>>()
    }))
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

    #[test]
    fn player_selection_requires_one_match_independent_of_roster_order() {
        assert_eq!(unique_player([7].into_iter()).unwrap(), 7);
        assert!(unique_player(std::iter::empty::<u32>()).is_err());
        for roster in [[7, 8], [8, 7], [7, 7]] {
            assert!(
                unique_player(roster.into_iter()).is_err(),
                "ambiguous recipient selected from roster"
            );
        }
    }
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
                limit: 64,
                item_ids: None
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
    fn inventory_counts_deduplicate_physical_slots_without_claiming_eligibility() {
        let result = inventory_item_counts(
            &[7, 8],
            [(11, 7, 4), (11, 7, 4), (12, 7, 2), (13, 8, 0), (14, 0, 9)],
        )
        .unwrap();
        assert_eq!(
            result["counts"],
            json!([{"item_id":7,"observed_count":6},{"item_id":8,"observed_count":0}])
        );
        assert_eq!(result["aliases_removed"], 1);
        assert_eq!(result["scope"], "unique_slots_returned_by_sdk");
        assert_eq!(result["inventory_completeness"], "unknown");
        assert_eq!(result["crafting_eligibility"], "unknown");
    }

    #[test]
    fn inventory_count_selectors_are_optional_bounded_and_strict() {
        let request = inventory(&obj(json!({"entity":1,"item_ids":[1,4294967295_u64]}))).unwrap();
        assert_eq!(request.item_ids, Some(vec![1, u32::MAX]));
        for ids in [
            json!([]),
            json!([0]),
            json!([1, 1]),
            json!([1.5]),
            json!(["1"]),
            json!([-1]),
            json!([4294967296_u64]),
            json!([1, 2, 3, 4, 5, 6, 7, 8, 9]),
            json!(null),
            json!({}),
        ] {
            assert!(inventory(&obj(json!({"entity":1,"item_ids":ids}))).is_err());
        }
        for ids in [vec![], vec![0], vec![1, 1], vec![1, 2, 3, 4, 5, 6, 7, 8, 9]] {
            assert!(inventory_item_counts(&ids, []).is_err());
        }
    }

    #[test]
    fn inventory_counts_refuse_conflicting_aliases_and_ignore_record_order() {
        for conflicting in [
            [(11, 7, 4), (11, 7, 5)],
            [(11, 7, 4), (11, 8, 4)],
            [(11, 7, 4), (11, 7, 0)],
        ] {
            assert!(
                inventory_item_counts(&[7], conflicting)
                    .unwrap_err()
                    .contains("conflicting")
            );
            assert!(inventory_item_counts(&[7], conflicting.into_iter().rev()).is_err());
        }
        let records = [(11, 7, 4), (12, 8, 2), (11, 7, 4), (13, 7, 3)];
        assert_eq!(
            inventory_item_counts(&[7, 8], records).unwrap(),
            inventory_item_counts(&[8, 7], records.into_iter().rev()).unwrap()
        );
    }

    #[test]
    fn inventory_counts_bound_record_work_and_keep_large_totals_exact() {
        let records = (0..COUNT_SLOT_LIMIT).map(|i| (i as u64, 7, u32::MAX));
        let result = inventory_item_counts(&[7], records).unwrap();
        assert_eq!(
            result["counts"][0]["observed_count"],
            281_474_976_645_120_u64
        );
        assert_eq!(result["aliases_removed"], 0);
        assert!(
            inventory_item_counts(&[7], std::iter::repeat_n((1, 7, 1), COUNT_SLOT_LIMIT + 1))
                .unwrap_err()
                .contains("65536")
        );
        let result =
            inventory_item_counts(&[7], std::iter::repeat_n((1, 7, 1), COUNT_SLOT_LIMIT)).unwrap();
        assert_eq!(result["counts"][0]["observed_count"], 1);
        assert_eq!(result["aliases_removed"], COUNT_SLOT_LIMIT - 1);
    }

    #[test]
    fn empty_sdk_result_is_zero_observed_without_proving_absence() {
        let result = inventory_item_counts(&[7], []).unwrap();
        assert_eq!(result["counts"], json!([{"item_id":7,"observed_count":0}]));
        assert_eq!(result["inventory_completeness"], "unknown");
        assert_eq!(result["crafting_eligibility"], "unknown");
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
    fn component_catalogue_rows_never_prove_completeness_or_absence() {
        for rows in [vec![], vec![json!({"entity":17,"components":["UsedItem"]})]] {
            let value = component_observations(3, 1, rows.clone());
            assert_eq!(value["total"], 3);
            assert_eq!(value["known_types"], 1);
            assert_eq!(value["entities"], json!(rows));
            assert_eq!(
                value["component_inventory_source"],
                "resolved_configured_names"
            );
            assert_eq!(value["complete_component_inventory_verified"], false);
            assert_eq!(value["component_absence_verified"], false);
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
