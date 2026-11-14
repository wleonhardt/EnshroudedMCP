//! Admission and game-free context gates for a single-cell construction proof.

use serde_json::{Map, Value, json};

pub const MAX_OPERATIONS: usize = 64;
pub const JOURNAL_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RECORD_BYTES: usize = 65_536;
pub const MAX_CAPTURE_PROPS: usize = 16;

pub const PLAN_MAX_CELLS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSpec {
    pub operation: String,
    pub source: [u32; 3],
    pub player: String,
    pub dry: bool,
    pub confirm: Option<String>,
    pub session: Option<String>,
    pub targets: Vec<[u32; 3]>,
    pub elements: Vec<(String, [u32; 3])>,
    pub canonical_plan: Value,
}

/// Bounded geometric plan whose material comes exclusively from the source cell.
#[must_use]
pub fn plan_schema() -> Value {
    let mut schema = place_schema();
    for key in ["x", "y", "z"] {
        schema["properties"].as_object_mut().unwrap().remove(key);
    }
    let mut plan = crate::building_plan::schema();
    plan["properties"]
        .as_object_mut()
        .unwrap()
        .remove("material");
    plan["required"]
        .as_array_mut()
        .unwrap()
        .retain(|key| key != "material");
    schema["properties"]["plan"] = plan;
    schema["required"] = json!([
        "operation",
        "source_x",
        "source_y",
        "source_z",
        "player",
        "plan"
    ]);
    schema
}

/// # Errors
/// Refuses malformed geometry, material labels, over 64 solids, multiple target
/// bricks, source overlap and any unconfirmed or unbound non-dry request.
pub fn check_plan(args: &Map<String, Value>) -> Result<PlanSpec, String> {
    crate::args::check(&plan_schema(), args)?;
    let plan = args
        .get("plan")
        .and_then(Value::as_object)
        .ok_or("missing plan object")?;
    crate::args::check(&plan_schema()["properties"]["plan"], plan)?;
    let mut geometry = plan.clone();
    geometry.insert("material".into(), json!("copied_source_voxel"));
    let spec = crate::building_plan::check(&geometry)?;
    let local = crate::building_plan::local_cells(&spec);
    if local.len() > PLAN_MAX_CELLS {
        return Err("construction plan exceeds 64 solid cells".into());
    }
    let mut elements = Vec::with_capacity(local.len());
    for [x, y, z] in local {
        let cell = crate::building_plan::cell(&spec, x, y, z);
        let target = [
            u32::try_from(cell[0]).map_err(|_| "negative plan coordinate")?,
            u32::try_from(cell[1]).map_err(|_| "negative plan coordinate")?,
            u32::try_from(cell[2]).map_err(|_| "negative plan coordinate")?,
        ];
        elements.push((format!("cell_{x}_{y}_{z}"), target));
    }
    let targets: Vec<_> = elements.iter().map(|(_, p)| *p).collect();
    let first = targets
        .first()
        .ok_or("construction plan has no solid cells")?;
    if targets
        .iter()
        .any(|p| p.map(|n| n / 8) != first.map(|n| n / 8))
    {
        return Err("construction plan targets must occupy one aligned 8-cell brick".into());
    }
    // Reuse identical source, player and mode admission without exposing plan-only fields.
    let mut placement = args.clone();
    placement.remove("plan");
    for (key, n) in ["x", "y", "z"].into_iter().zip(first) {
        placement.insert(key.into(), json!(f64::from(*n) * 0.5));
    }
    let base = check_place(&placement)?;
    if targets.contains(&base.source) {
        return Err("source and target cells must differ".into());
    }
    let mut canonical_plan = json!({"kind":match spec.shape { crate::building_plan::Shape::Floor => "floor", crate::building_plan::Shape::Wall => "wall", crate::building_plan::Shape::Doorway => "doorway" },
        "x":f64::from(spec.origin[0])*0.5,"y":f64::from(spec.origin[1])*0.5,"z":f64::from(spec.origin[2])*0.5,
        "width":f64::from(spec.size[0])*0.5,"height":f64::from(spec.size[1])*0.5,"depth":f64::from(spec.size[2])*0.5,"yaw":spec.yaw});
    if let Some([offset, width, height]) = spec.door {
        canonical_plan["door_offset"] = json!(f64::from(offset) * 0.5);
        canonical_plan["door_width"] = json!(f64::from(width) * 0.5);
        canonical_plan["door_height"] = json!(f64::from(height) * 0.5);
    }
    Ok(PlanSpec {
        operation: base.operation,
        source: base.source,
        player: base.player,
        dry: base.dry,
        confirm: base.confirm,
        session: base.session,
        targets,
        elements,
        canonical_plan,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceSpec {
    pub operation: String,
    pub source: [u32; 3],
    pub target: [u32; 3],
    pub player: String,
    pub session: Option<String>,
    pub dry: bool,
    pub confirm: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoSpec {
    pub operation: String,
    pub dry: bool,
    pub confirm: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Air,
    Building,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub kind: CellKind,
    pub fill: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementFacts {
    pub source: Cell,
    pub target: Cell,
    pub persists: bool,
    pub connected_players: usize,
    pub matching_actors: usize,
}

#[must_use]
pub fn status_schema() -> Value {
    json!({"type":"object", "additionalProperties":false,
        "properties":{"operation":{"type":"string","minLength":1,"maxLength":64}},
        "required":["operation"]})
}

#[must_use]
pub fn undo_schema() -> Value {
    let mut schema = status_schema();
    schema["properties"]["dry"] = json!({"type":"boolean","default":true});
    schema["properties"]["confirm"] = json!({"type":"string","enum":[crate::write_gate::CONFIRM]});
    schema
}

#[must_use]
pub fn place_schema() -> Value {
    let mut schema = undo_schema();
    for (key, max) in [
        ("source_x", crate::bounds::EDGE),
        ("source_y", crate::bounds::HEIGHT),
        ("source_z", crate::bounds::EDGE),
        ("x", crate::bounds::EDGE),
        ("y", crate::bounds::HEIGHT),
        ("z", crate::bounds::EDGE),
    ] {
        schema["properties"][key] =
            json!({"type":"number","minimum":0,"maximum":max - 0.5,"multipleOf":0.5});
    }
    schema["properties"]["session"] = json!({"type":"string","minLength":1,"maxLength":128});
    schema["properties"]["player"] = json!({"type":"string","minLength":1,"maxLength":128});
    schema["required"] = json!([
        "operation",
        "source_x",
        "source_y",
        "source_z",
        "x",
        "y",
        "z",
        "player"
    ]);
    schema
}

fn operation(args: &Map<String, Value>) -> Result<String, String> {
    args.get("operation")
        .and_then(Value::as_str)
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 64
                && s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        })
        .map(str::to_owned)
        .ok_or_else(|| {
            "operation must contain 1 to 64 ASCII letters, digits, underscores or hyphens".into()
        })
}

fn mode(args: &Map<String, Value>) -> Result<(bool, Option<String>), String> {
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let confirm = args
        .get("confirm")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if !dry && confirm.as_deref() != Some(crate::write_gate::CONFIRM) {
        return Err("pass confirm: \"server-write\" with every write".into());
    }
    Ok((dry, confirm))
}

fn point(args: &Map<String, Value>, keys: [&str; 3]) -> Result<[u32; 3], String> {
    let mut out = [0; 3];
    for (i, key) in keys.into_iter().enumerate() {
        let n = args
            .get(key)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("missing number '{key}'"))?;
        let max = if i == 1 {
            crate::bounds::HEIGHT
        } else {
            crate::bounds::EDGE
        };
        if !n.is_finite() || n < 0.0 || n + 0.5 > max || (n * 2.0).fract() != 0.0 {
            return Err(format!(
                "'{key}' must identify a complete 0.5 m cell inside world bounds"
            ));
        }
        out[i] = (n * 2.0) as u32;
    }
    Ok(out)
}

/// Checks argument shape and operation identity; grants no write permission.
/// # Errors
/// Refuses unknown fields and invalid operation identifiers.
pub fn check_status(args: &Map<String, Value>) -> Result<String, String> {
    crate::args::check(&status_schema(), args)?;
    operation(args)
}

/// Checks undo admission; the caller must also apply the flag-file write gate.
/// # Errors
/// Refuses invalid arguments and unconfirmed non-dry requests.
pub fn check_undo(args: &Map<String, Value>) -> Result<UndoSpec, String> {
    crate::args::check(&undo_schema(), args)?;
    let operation = operation(args)?;
    let (dry, confirm) = mode(args)?;
    Ok(UndoSpec {
        operation,
        dry,
        confirm,
    })
}

/// Checks a single-cell copy request; the caller must apply the write gate.
/// # Errors
/// Refuses invalid selectors, incomplete cells, identical cells and unconfirmed writes.
pub fn check_place(args: &Map<String, Value>) -> Result<PlaceSpec, String> {
    crate::args::check(&place_schema(), args)?;
    let operation = operation(args)?;
    let source = point(args, ["source_x", "source_y", "source_z"])?;
    let target = point(args, ["x", "y", "z"])?;
    if source == target {
        return Err("source and target cells must differ".into());
    }
    let player = args
        .get("player")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control))
        .ok_or("player must contain 1 to 128 UTF-8 bytes without control characters")?
        .to_owned();
    let (dry, confirm) = mode(args)?;
    let session = args
        .get("session")
        .map(|value| {
            value
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() <= 128
                        && s.bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                })
                .map(str::to_owned)
                .ok_or_else(|| {
                    "session must contain 1 to 128 ASCII letters, digits, underscores or hyphens"
                        .to_owned()
                })
        })
        .transpose()?;
    if !dry && session.is_none() {
        return Err("non-dry placement requires session from the dry preview".into());
    }
    Ok(PlaceSpec {
        operation,
        source,
        target,
        player,
        session,
        dry,
        confirm,
    })
}

/// Facts independently observed before undo, including complete voxel equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoFacts {
    pub planned_cells: usize,
    pub expected_voxels_equal: bool,
    pub props_unchanged: bool,
    pub persists: bool,
}

/// # Errors
/// Refuses use outside an active world session.
pub fn check_active(active: bool) -> Result<(), String> {
    if active {
        Ok(())
    } else {
        Err("construction world session is inactive".into())
    }
}

/// # Errors
/// Refuses retained handles from another world epoch.
pub fn check_epoch(stored: u64, current: u64) -> Result<(), String> {
    if stored == current {
        Ok(())
    } else {
        Err("stale session cannot be undone".into())
    }
}

/// # Errors
/// Refuses empty or mismatched preview session tokens.
pub fn check_session(requested: &str, current: &str) -> Result<(), String> {
    if !requested.is_empty() && requested == current {
        Ok(())
    } else {
        Err("construction preview session does not match current world session".into())
    }
}

/// # Errors
/// Refuses reusing an operation identity for a different request.
pub fn same_request(existing: &Value, requested: &Value) -> Result<(), String> {
    if existing == requested {
        Ok(())
    } else {
        Err("operation identity already has a different request".into())
    }
}

/// # Errors
/// Refuses missing or ambiguous connected actors.
pub fn check_actor_counts(connected: usize, matching: usize) -> Result<(), String> {
    if connected == 1 && matching == 1 {
        Ok(())
    } else {
        Err("construction requires exactly one connected player and one matching actor".into())
    }
}

/// # Errors
/// Refuses new operations when capacity is exhausted or an earlier proof is unsettled.
pub fn check_operation_capacity(count: usize, unsettled: bool) -> Result<(), String> {
    if count >= MAX_OPERATIONS {
        return Err("construction operation limit reached".into());
    }
    if unsettled {
        return Err("an earlier construction proof remains unsettled".into());
    }
    Ok(())
}

/// # Errors
/// Refuses captures with more props than the fixed proof bound.
pub fn check_capture_props(count: usize) -> Result<(), String> {
    if count <= MAX_CAPTURE_PROPS {
        Ok(())
    } else {
        Err("capture exceeds 16 props".into())
    }
}

/// # Errors
/// Refuses records larger than the fixed allocation bound.
pub fn check_record_size(bytes: usize) -> Result<(), String> {
    if bytes <= MAX_RECORD_BYTES {
        Ok(())
    } else {
        Err("construction record exceeds 64 KiB".into())
    }
}

/// # Errors
/// Refuses journal growth beyond its cap, including arithmetic overflow.
pub fn check_journal_size(existing: u64, incoming: usize) -> Result<(), String> {
    let total = u64::try_from(incoming)
        .ok()
        .and_then(|n| existing.checked_add(n));
    if total.is_some_and(|n| n <= JOURNAL_MAX_BYTES as u64) {
        Ok(())
    } else {
        Err("construction journal exceeds 8 MiB".into())
    }
}

/// # Errors
/// Refuses changed props, non-persistent sites and multi-cell postimage mismatches.
pub fn check_undo_context(facts: UndoFacts) -> Result<(), String> {
    if facts.planned_cells > 1 && !facts.expected_voxels_equal {
        return Err("multi-cell undo requires exact expected voxel postimage".into());
    }
    if !facts.props_unchanged {
        return Err("undo capture props changed".into());
    }
    if !facts.persists {
        return Err("undo requires persistent world".into());
    }
    Ok(())
}

/// Checks observed placement facts. Native build rights must additionally be
/// checked before commit and rechecked by `transaction_for`.
/// # Errors
/// Refuses non-building sources, occupied targets, temporary worlds and ambiguous actors.
pub fn check_context(facts: PlacementFacts) -> Result<(), String> {
    if facts.source.kind != CellKind::Building {
        return Err("source must be a building cell".into());
    }
    if facts.target.kind != CellKind::Air {
        return Err("target must be an air cell".into());
    }
    if !facts.persists {
        return Err("construction requires a persistent world".into());
    }
    check_actor_counts(facts.connected_players, facts.matching_actors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_replay_and_undo_context_fail_closed() {
        assert!(check_active(true).is_ok());
        assert!(check_active(false).is_err());
        assert!(check_epoch(7, 7).is_ok());
        assert!(check_epoch(7, 8).is_err());
        assert!(check_session("boot-1", "boot-1").is_ok());
        for (requested, current) in [("", ""), ("", "boot-1"), ("boot-1", "boot-2")] {
            assert!(check_session(requested, current).is_err());
        }
        assert!(same_request(&json!({"x":1}), &json!({"x":1})).is_ok());
        assert!(same_request(&json!({"x":1}), &json!({"x":2})).is_err());
        assert!(check_actor_counts(1, 1).is_ok());
        for counts in [(0, 1), (1, 0), (2, 1), (1, 2), (usize::MAX, usize::MAX)] {
            assert!(check_actor_counts(counts.0, counts.1).is_err());
        }
        for props_unchanged in [false, true] {
            for persists in [false, true] {
                assert_eq!(
                    check_undo_context(UndoFacts {
                        planned_cells: 1,
                        expected_voxels_equal: false,
                        props_unchanged,
                        persists
                    })
                    .is_ok(),
                    props_unchanged && persists
                );
            }
        }
    }

    #[test]
    fn multi_cell_undo_requires_exact_voxel_postimage() {
        let facts = UndoFacts {
            planned_cells: 1,
            expected_voxels_equal: false,
            props_unchanged: true,
            persists: true,
        };
        assert!(check_undo_context(facts).is_ok());
        assert!(
            check_undo_context(UndoFacts {
                planned_cells: 16,
                ..facts
            })
            .is_err()
        );
        assert!(
            check_undo_context(UndoFacts {
                planned_cells: 16,
                expected_voxels_equal: true,
                ..facts
            })
            .is_ok()
        );
        for planned_cells in [1, 16] {
            assert!(
                check_undo_context(UndoFacts {
                    planned_cells,
                    expected_voxels_equal: true,
                    props_unchanged: false,
                    ..facts
                })
                .is_err()
            );
            assert!(
                check_undo_context(UndoFacts {
                    planned_cells,
                    expected_voxels_equal: true,
                    persists: false,
                    ..facts
                })
                .is_err()
            );
        }
    }

    #[test]
    fn record_and_journal_exact_limits_and_overflow() {
        assert!(check_operation_capacity(MAX_OPERATIONS - 1, false).is_ok());
        assert!(check_operation_capacity(MAX_OPERATIONS, false).is_err());
        assert!(check_operation_capacity(usize::MAX, false).is_err());
        assert!(check_operation_capacity(0, true).is_err());
        assert!(check_capture_props(0).is_ok());
        assert!(check_capture_props(MAX_CAPTURE_PROPS).is_ok());
        assert!(check_capture_props(MAX_CAPTURE_PROPS + 1).is_err());
        assert!(check_capture_props(usize::MAX).is_err());
        assert!(check_record_size(0).is_ok());
        assert!(check_record_size(MAX_RECORD_BYTES).is_ok());
        assert!(check_record_size(MAX_RECORD_BYTES + 1).is_err());
        assert!(check_record_size(usize::MAX).is_err());
        assert!(check_journal_size(0, JOURNAL_MAX_BYTES).is_ok());
        assert!(check_journal_size(JOURNAL_MAX_BYTES as u64, 0).is_ok());
        assert!(check_journal_size(JOURNAL_MAX_BYTES as u64, 1).is_err());
        assert!(check_journal_size(u64::MAX, 1).is_err());
        assert!(check_journal_size(1, usize::MAX).is_err());
    }

    fn plan_request() -> Map<String, Value> {
        json!({"operation":"plan-1","source_x":0,"source_y":0,"source_z":0,"player":"Researcher",
            "plan":{"kind":"floor","x":4,"y":4,"z":4,"width":2,"height":0.5,"depth":2}})
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn plans_preserve_counts_keys_and_normalized_identity() {
        let mut args = plan_request();
        let floor = check_plan(&args).unwrap();
        assert_eq!(floor.targets.len(), 16);
        assert_eq!(floor.elements[0].0, "cell_0_0_0");
        args.get_mut("plan").unwrap()["yaw"] = json!(0);
        assert_eq!(
            check_plan(&args).unwrap().canonical_plan,
            floor.canonical_plan
        );
        let keys: Vec<_> = floor.elements.iter().map(|(key, _)| key.clone()).collect();
        for (yaw, x, z) in [(90, 6, 4), (180, 6, 6), (270, 4, 6)] {
            args.get_mut("plan").unwrap()["yaw"] = json!(yaw);
            args.get_mut("plan").unwrap()["x"] = json!(x);
            args.get_mut("plan").unwrap()["z"] = json!(z);
            assert_eq!(
                check_plan(&args)
                    .unwrap()
                    .elements
                    .iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>(),
                keys
            );
        }
        let plan = args.get_mut("plan").unwrap();
        *plan = json!({"kind":"wall","x":4,"y":4,"z":4,"width":4,"height":3,"depth":0.5});
        assert_eq!(check_plan(&args).unwrap().targets.len(), 48);
        let plan = args.get_mut("plan").unwrap();
        plan["kind"] = json!("doorway");
        plan["door_offset"] = json!(1);
        plan["door_width"] = json!(1.5);
        plan["door_height"] = json!(2);
        assert_eq!(check_plan(&args).unwrap().targets.len(), 36);
    }

    #[test]
    fn plan_cardinality_bricks_bounds_and_write_fields_fail_closed() {
        let mut args = plan_request();
        args.get_mut("plan").unwrap()["width"] = json!(4);
        args.get_mut("plan").unwrap()["depth"] = json!(4);
        assert_eq!(check_plan(&args).unwrap().targets.len(), 64);
        args.get_mut("plan").unwrap()["width"] = json!(6.5);
        args.get_mut("plan").unwrap()["depth"] = json!(2.5);
        assert!(check_plan(&args).unwrap_err().contains("64 solid"));
        let mut args = plan_request();
        args.get_mut("plan").unwrap()["x"] = json!(7.5);
        assert!(check_plan(&args).unwrap_err().contains("one aligned"));
        for (key, value) in [
            ("material", json!("stone")),
            ("x", json!(10240)),
            ("yaw", json!(45)),
        ] {
            let mut args = plan_request();
            args.get_mut("plan").unwrap()[key] = value;
            assert!(check_plan(&args).is_err());
        }
        let mut args = plan_request();
        args.insert("x".into(), json!(4));
        assert!(check_plan(&args).is_err());
        let mut args = plan_request();
        for (key, value) in [("source_x", 4), ("source_y", 4), ("source_z", 4)] {
            args.insert(key.into(), json!(value));
        }
        assert!(check_plan(&args).is_err());
        let mut args = plan_request();
        args.insert("dry".into(), json!(false));
        assert!(check_plan(&args).is_err());
        args.insert("confirm".into(), json!("server-write"));
        assert!(check_plan(&args).is_err());
        args.insert("session".into(), json!("boot-1"));
        assert!(check_plan(&args).is_ok());
        args.insert("session".into(), json!("invalid/session"));
        assert!(check_plan(&args).is_err());
    }

    fn request() -> Map<String, Value> {
        json!({"operation":"proof_1-A","source_x":0,"source_y":0,"source_z":0,
            "x":0.5,"y":0,"z":0,"player":"Researcher"})
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn dry_default_and_explicit_write_confirmation() {
        let mut args = request();
        let spec = check_place(&args).unwrap();
        assert!(spec.dry);
        assert_eq!(spec.confirm, None);
        assert_eq!(spec.target, [1, 0, 0]);
        args.insert("dry".into(), json!(false));
        assert!(check_place(&args).is_err());
        args.insert("confirm".into(), json!("server-write"));
        args.insert("session".into(), json!("bootnonce-1"));
        assert!(!check_place(&args).unwrap().dry);
        args.insert("confirm".into(), json!("wrong"));
        assert!(check_place(&args).is_err());
        let mut undo = json!({"operation":"proof_1-A"})
            .as_object()
            .unwrap()
            .clone();
        assert!(check_undo(&undo).unwrap().dry);
        undo.insert("dry".into(), json!(false));
        assert!(check_undo(&undo).is_err());
        undo.insert("confirm".into(), json!("server-write"));
        assert!(!check_undo(&undo).unwrap().dry);
    }

    #[test]
    fn apply_requires_bounded_preview_session_and_other_tools_refuse_it() {
        let mut args = request();
        assert_eq!(check_place(&args).unwrap().session, None);
        args.insert("dry".into(), json!(false));
        args.insert("confirm".into(), json!("server-write"));
        assert!(check_place(&args).is_err());
        for session in ["", "space here", "boot/1", "é", &"a".repeat(129)] {
            args.insert("session".into(), json!(session));
            assert!(check_place(&args).is_err());
            args.insert("dry".into(), json!(true));
            assert!(check_place(&args).is_err());
            args.insert("dry".into(), json!(false));
        }
        args.insert("session".into(), json!("a".repeat(128)));
        assert_eq!(check_place(&args).unwrap().session, Some("a".repeat(128)));
        args.insert("session".into(), json!(null));
        assert!(check_place(&args).is_err());
        let other = json!({"operation":"proof", "session":"boot-1"});
        assert!(check_status(other.as_object().unwrap()).is_err());
        assert!(check_undo(other.as_object().unwrap()).is_err());
    }

    #[test]
    fn cells_are_distinct_aligned_and_complete_at_world_edges() {
        let mut args = request();
        for (key, value) in [("x", 10239.5), ("y", 1023.5), ("z", 10239.5)] {
            args.insert(key.into(), json!(value));
        }
        assert_eq!(check_place(&args).unwrap().target, [20479, 2047, 20479]);
        for (key, value) in [
            ("x", 10240.0),
            ("y", 1024.0),
            ("z", -0.5),
            ("source_x", 0.25),
        ] {
            let mut bad = args.clone();
            bad.insert(key.into(), json!(value));
            assert!(check_place(&bad).is_err(), "{key}={value}");
        }
        let mut same = request();
        same.insert("x".into(), json!(0));
        assert!(check_place(&same).is_err());
    }

    #[test]
    fn identity_limits_and_unknown_arguments_are_enforced() {
        for id in ["", "a/b", "é", "space here", &"a".repeat(65)] {
            let mut args = request();
            args.insert("operation".into(), json!(id));
            assert!(check_place(&args).is_err());
            assert!(check_status(json!({"operation":id}).as_object().unwrap()).is_err());
        }
        assert!(check_status(json!({"operation":"a".repeat(64)}).as_object().unwrap()).is_ok());
        for player in ["", "  ", "a\nb", &"é".repeat(65)] {
            let mut args = request();
            args.insert("player".into(), json!(player));
            assert!(check_place(&args).is_err());
        }
        let mut args = request();
        args.insert("player".into(), json!("é".repeat(64)));
        assert!(check_place(&args).is_ok());
        for key in ["dry", "operation", "player", "source_x", "y"] {
            let mut bad = request();
            bad.insert(key.into(), Value::Null);
            assert!(check_place(&bad).is_err());
        }
        for key in [
            "operation",
            "source_x",
            "source_y",
            "source_z",
            "x",
            "y",
            "z",
            "player",
        ] {
            let mut bad = request();
            bad.remove(key);
            assert!(check_place(&bad).is_err());
        }
        let mut bad = request();
        bad.insert("force".into(), json!(true));
        assert!(check_place(&bad).is_err());
        assert!(
            check_status(
                json!({"operation":"proof", "dry":true})
                    .as_object()
                    .unwrap()
            )
            .is_err()
        );
        assert!(
            check_undo(
                json!({"operation":"proof", "force":true})
                    .as_object()
                    .unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn context_requires_exact_empty_destination_and_unambiguous_persistent_player() {
        let facts = PlacementFacts {
            source: Cell {
                kind: CellKind::Building,
                fill: 1,
            },
            target: Cell {
                kind: CellKind::Air,
                fill: 0,
            },
            persists: true,
            connected_players: 1,
            matching_actors: 1,
        };
        assert!(check_context(facts).is_ok());
        assert!(
            check_context(PlacementFacts {
                source: Cell {
                    kind: CellKind::Building,
                    fill: 0
                },
                target: Cell {
                    kind: CellKind::Air,
                    fill: 255
                },
                ..facts
            })
            .is_ok()
        );
        for source in [
            Cell {
                kind: CellKind::Air,
                fill: 1,
            },
            Cell {
                kind: CellKind::Other,
                fill: 1,
            },
            Cell {
                kind: CellKind::Other,
                fill: 0,
            },
        ] {
            assert!(check_context(PlacementFacts { source, ..facts }).is_err());
        }
        for target in [
            Cell {
                kind: CellKind::Building,
                fill: 0,
            },
            Cell {
                kind: CellKind::Other,
                fill: 0,
            },
            Cell {
                kind: CellKind::Building,
                fill: 1,
            },
        ] {
            assert!(check_context(PlacementFacts { target, ..facts }).is_err());
        }
        assert!(
            check_context(PlacementFacts {
                persists: false,
                ..facts
            })
            .is_err()
        );
        for count in [0, 2] {
            assert!(
                check_context(PlacementFacts {
                    connected_players: count,
                    ..facts
                })
                .is_err()
            );
            assert!(
                check_context(PlacementFacts {
                    matching_actors: count,
                    ..facts
                })
                .is_err()
            );
        }
    }
}
