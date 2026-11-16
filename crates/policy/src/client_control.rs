//! Admission for supported native client commands. No server authority.
use serde_json::{Map, Value, json};
use std::time::Duration;

pub const MAX_OPERATIONS: usize = 64;
pub const WAIT: Duration = Duration::from_secs(2);
pub const SCREENS: &[&str] = &[
    "SystemMenu",
    "InGameMenu",
    "Map",
    "Journal",
    "Character",
    "PlayerCrafting",
    "Backpack",
    "SkillTree",
    "TextChat",
    "Social",
];
pub const TABS: &[&str] = &[
    "Social",
    "Map",
    "Journal",
    "PlayerCrafting",
    "Backpack",
    "Character",
    "SkillTree",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Open(String),
    Close,
    Tab(String),
    Look(f32, f32),
    Move([f32; 2], u32),
    Recipe(u32),
}
#[derive(Clone, Debug)]
pub struct Spec {
    pub operation: String,
    pub action: Action,
    pub dry: bool,
    pub preview: Option<String>,
    pub confirm: Option<String>,
}
#[derive(Clone, Debug)]
pub struct CancelSpec {
    pub operation: String,
    pub dry: bool,
    pub preview: Option<String>,
    pub confirm: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Context {
    pub fresh: bool,
    pub age_ms: Option<u64>,
    pub session: u64,
    pub screens_read: bool,
    pub screen: Option<String>,
    pub tab_read: bool,
    pub tab: Option<String>,
}
pub fn schema() -> Value {
    json!({"type":"object","properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "action":{"type":"string","enum":["open_screen","close_screen","choose_tab","look","move","highlight_recipe"]},
        "target":{"type":"string","maxLength":32},
        "yaw":{"type":"number"},"pitch":{"type":"number"},
        "dx":{"type":"number"},"dy":{"type":"number"},"hold_ms":{"type":"integer","minimum":1,"maximum":250},
        "recipe_id":{"type":"integer","minimum":1,"maximum":u32::MAX},
        "dry":{"type":"boolean","default":true},"preview":{"type":"string","minLength":1,"maxLength":2048},
        "confirm":{"type":"string"}},"required":["operation","action"]})
}
pub fn status_schema() -> Value {
    json!({"type":"object","properties":{"operation":{"type":"string","minLength":1,"maxLength":64}},"required":["operation"]})
}
pub fn cancel_schema() -> Value {
    let mut schema = status_schema();
    schema["properties"]["dry"] = json!({"type":"boolean","default":true});
    schema["properties"]["preview"] = json!({"type":"string","minLength":1,"maxLength":2048});
    schema["properties"]["confirm"] = json!({"type":"string"});
    schema
}
pub fn operation(args: &Map<String, Value>) -> Result<String, String> {
    let id = args.get("operation").and_then(Value::as_str).unwrap_or("");
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(
            "client operation requires 1..64 lowercase ASCII letters, digits, '-' or '_'".into(),
        );
    }
    Ok(id.into())
}
pub fn reserved_operation(filename: &str) -> Result<String, String> {
    let operation = filename
        .strip_prefix("command-")
        .and_then(|name| name.strip_suffix(".json"))
        .ok_or("unexpected entry in client command reservations")?;
    self::operation(json!({"operation":operation}).as_object().unwrap())
}
pub fn mode(args: &Map<String, Value>) -> Result<(bool, Option<String>, Option<String>), String> {
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let preview = args.get("preview").and_then(Value::as_str);
    if preview.is_some_and(|p| p.is_empty() || p.len() > 2048) || (!dry && preview.is_none()) {
        return Err("real client action requires its bounded current dry preview".into());
    }
    Ok((
        dry,
        preview.map(str::to_owned),
        args.get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
    ))
}
pub fn cancel(args: &Map<String, Value>) -> Result<CancelSpec, String> {
    crate::args::check(&cancel_schema(), args)?;
    let (dry, preview, confirm) = mode(args)?;
    Ok(CancelSpec {
        operation: operation(args)?,
        dry,
        preview,
        confirm,
    })
}
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let action = args["action"].as_str().unwrap_or("");
    let fields: &[&str] = match action {
        "open_screen" | "choose_tab" => &["target"],
        "close_screen" => &[],
        "look" => &["yaw", "pitch"],
        "move" => &["dx", "dy", "hold_ms"],
        "highlight_recipe" => &["recipe_id"],
        _ => return Err("unsupported native client action".into()),
    };
    if args.keys().any(|key| {
        !fields.contains(&key.as_str())
            && !["operation", "action", "dry", "preview", "confirm"].contains(&key.as_str())
    }) {
        return Err("argument does not belong to this client action".into());
    }
    let number = |key: &str| {
        args.get(key)
            .and_then(Value::as_f64)
            .filter(|n| n.is_finite())
            .ok_or_else(|| format!("{key} must be a finite number"))
    };
    let action = match action {
        "open_screen" | "choose_tab" => {
            let target = args.get("target").and_then(Value::as_str).unwrap_or("");
            let allowed = if action == "open_screen" {
                SCREENS
            } else {
                TABS
            };
            if !allowed.contains(&target) {
                return Err("unsupported native screen or tab".into());
            }
            if action == "open_screen" {
                Action::Open(target.into())
            } else {
                Action::Tab(target.into())
            }
        }
        "close_screen" => Action::Close,
        "look" => {
            let (yaw, pitch) = (number("yaw")?, number("pitch")?);
            if yaw.abs() > f64::from(std::f32::consts::FRAC_PI_2)
                || pitch.abs() > f64::from(std::f32::consts::FRAC_PI_2)
            {
                return Err("look is bounded to a quarter turn per axis".into());
            }
            Action::Look(yaw as f32, pitch as f32)
        }
        "move" => {
            let (x, y) = (number("dx")?, number("dy")?);
            let hold = args
                .get("hold_ms")
                .and_then(Value::as_u64)
                .filter(|n| (1..=250).contains(n))
                .ok_or("move hold_ms must be an integer from 1 to 250")?;
            let direction = [x as f32, y as f32];
            if x.hypot(y) > 1.0 || direction[0].hypot(direction[1]) == 0.0 {
                return Err("move direction must have nonzero length at most one".into());
            }
            Action::Move(direction, hold as u32)
        }
        "highlight_recipe" => Action::Recipe(
            args.get("recipe_id")
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n != 0)
                .ok_or("recipe_id must be a nonzero u32")?,
        ),
        _ => unreachable!(),
    };
    let (dry, preview, confirm) = mode(args)?;
    Ok(Spec {
        operation: operation(args)?,
        action,
        dry,
        preview,
        confirm,
    })
}
pub fn payload(action: &Action) -> Value {
    match action {
        Action::Open(target) => json!({"action":"open_screen","target":target}),
        Action::Close => json!({"action":"close_screen"}),
        Action::Tab(target) => json!({"action":"choose_tab","target":target}),
        Action::Look(yaw, pitch) => json!({"action":"look","yaw":yaw,"pitch":pitch}),
        Action::Move(direction, hold) => {
            json!({"action":"move","direction":direction,"hold_ms":hold})
        }
        Action::Recipe(id) => json!({"action":"highlight_recipe","recipe_id":id}),
    }
}
pub fn applicable(context: &Context, action: &Action) -> Result<(), String> {
    if !context.fresh
        || context.age_ms.is_none_or(|age| age > 1000)
        || context.session == 0
        || !context.screens_read
        || context.screen.as_deref().is_none_or(str::is_empty)
    {
        return Err(
            "client command requires a fresh nonzero session and readable HUD screen".into(),
        );
    }
    match action {
        Action::Look(..) | Action::Move(..) if context.screen.as_deref() != Some("InGame") => {
            Err("look/move require the InGame HUD screen".into())
        }
        Action::Tab(_)
            if !context.tab_read
                || context
                    .tab
                    .as_deref()
                    .is_none_or(|tab| !TABS.contains(&tab)) =>
        {
            Err("choose_tab requires an observed current menu tab".into())
        }
        Action::Recipe(_)
            if !context.tab_read || context.tab.as_deref() != Some("PlayerCrafting") =>
        {
            Err("recipe highlight requires the observed PlayerCrafting tab".into())
        }
        _ => Ok(()),
    }
}
pub fn preview(boot: &str, spec: &Spec, context: &Context) -> String {
    json!({"boot":boot,"operation":spec.operation,"request":payload(&spec.action),"session":context.session,"screen":context.screen,"tab_read":context.tab_read,"tab":context.tab}).to_string()
}
pub fn exact(expected: Option<&str>, current: &str) -> Result<(), String> {
    if expected == Some(current) {
        Ok(())
    } else {
        Err("client preview changed; inspect current session/screen and preview again".into())
    }
}
pub fn capacity(count: usize) -> Result<(), String> {
    if count < MAX_OPERATIONS {
        Ok(())
    } else {
        Err("client command operation capacity reached".into())
    }
}
pub fn deadline(age: Duration) -> Result<(), String> {
    if age < WAIT {
        Ok(())
    } else {
        Err("client command deadline expired before native submission".into())
    }
}
pub fn journal_headroom(current: u64, records: usize) -> Result<(), String> {
    crate::construction::check_journal_size(
        current,
        records * crate::write_gate::JOURNAL_RECORD_BYTES,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(value: Value) -> Result<Spec, String> {
        check(value.as_object().unwrap())
    }
    fn context() -> Context {
        Context {
            fresh: true,
            age_ms: Some(0),
            session: 1,
            screens_read: true,
            screen: Some("InGame".into()),
            tab_read: true,
            tab: None,
        }
    }
    #[test]
    fn reservation_names_recover_only_bounded_caller_operation_identities() {
        assert_eq!(reserved_operation("command-a_12-z.json").unwrap(), "a_12-z");
        for name in [
            "command-.json",
            "command-UPPER.json",
            "command-../other.json",
            "command-a.json.bak",
            "other.json",
        ] {
            assert!(reserved_operation(name).is_err(), "{name}");
        }
        assert!(reserved_operation(&format!("command-{}.json", "a".repeat(65))).is_err());
    }
    #[test]
    fn bounded_supported_actions_default_dry_and_refuse_cross_action_or_unpublished_fields() {
        for action in [
            json!({"action":"open_screen","target":"Map"}),
            json!({"action":"close_screen"}),
            json!({"action":"choose_tab","target":"Backpack"}),
            json!({"action":"look","yaw":0.1,"pitch":-0.2}),
            json!({"action":"move","dx":0,"dy":1,"hold_ms":250}),
            json!({"action":"highlight_recipe","recipe_id":u32::MAX}),
        ] {
            let mut value = action;
            value["operation"] = json!("fixture");
            assert!(parse(value).unwrap().dry);
        }
        for action in [
            json!({"action":"open_screen","target":"StorageChest"}),
            json!({"action":"join","target":"Map"}),
            json!({"action":"close_screen","target":"Map"}),
            json!({"action":"look","yaw":2,"pitch":0}),
            json!({"action":"look","yaw":0}),
            json!({"action":"move","dx":1,"dy":1,"hold_ms":1}),
            json!({"action":"move","dx":0,"dy":0,"hold_ms":1}),
            json!({"action":"move","dx":0,"dy":1,"hold_ms":251}),
            json!({"action":"highlight_recipe","recipe_id":0}),
            json!({"action":"highlight_recipe","recipe_id":4294967296u64}),
            json!({"action":"close_screen","dry":false}),
        ] {
            let mut value = action;
            value["operation"] = json!("fixture");
            assert!(parse(value).is_err());
        }
        for id in ["", "../other", "UPPER", "a\nb"] {
            assert!(parse(json!({"operation":id,"action":"close_screen"})).is_err());
        }
    }
    #[test]
    fn session_freshness_and_action_specific_screen_applicability_fail_closed() {
        let ctx = context();
        assert!(applicable(&ctx, &Action::Move([0.0, 1.0], 1)).is_ok());
        for ctx in [
            Context {
                fresh: false,
                ..context()
            },
            Context {
                age_ms: None,
                ..context()
            },
            Context {
                age_ms: Some(1001),
                ..context()
            },
            Context {
                session: 0,
                ..context()
            },
            Context {
                screens_read: false,
                ..context()
            },
            Context {
                screen: None,
                ..context()
            },
        ] {
            assert!(applicable(&ctx, &Action::Close).is_err());
        }
        let menu = Context {
            screen: Some("InGameMenu".into()),
            tab: Some("Backpack".into()),
            ..context()
        };
        assert!(applicable(&menu, &Action::Look(0.1, 0.0)).is_err());
        assert!(applicable(&menu, &Action::Tab("Map".into())).is_ok());
        assert!(applicable(&menu, &Action::Recipe(1)).is_err());
        assert!(
            applicable(
                &Context {
                    tab: Some("PlayerCrafting".into()),
                    ..menu
                },
                &Action::Recipe(1)
            )
            .is_ok()
        );
    }
    #[test]
    fn previews_bind_boot_payload_operation_session_and_tab_and_capacity_is_bounded() {
        let spec = parse(json!({"operation":"fixture","action":"close_screen"})).unwrap();
        let token = preview("boot", &spec, &context());
        assert!(exact(Some(&token), &token).is_ok());
        assert!(exact(Some(&token), &preview("next", &spec, &context())).is_err());
        let mut changed = spec.clone();
        changed.operation = "other".into();
        assert!(exact(Some(&token), &preview("boot", &changed, &context())).is_err());
        changed = spec.clone();
        changed.action = Action::Open("Map".into());
        assert!(exact(Some(&token), &preview("boot", &changed, &context())).is_err());
        assert!(
            exact(
                Some(&token),
                &preview(
                    "boot",
                    &spec,
                    &Context {
                        screen: Some("Map".into()),
                        ..context()
                    }
                )
            )
            .is_err()
        );
        assert!(
            exact(
                Some(&token),
                &preview(
                    "boot",
                    &spec,
                    &Context {
                        session: 2,
                        ..context()
                    }
                )
            )
            .is_err()
        );
        assert!(
            exact(
                Some(&token),
                &preview(
                    "boot",
                    &spec,
                    &Context {
                        tab: Some("Map".into()),
                        ..context()
                    }
                )
            )
            .is_err()
        );
        assert!(capacity(63).is_ok());
        assert!(capacity(64).is_err());
        assert!(deadline(WAIT).is_err());
    }
    #[test]
    fn move_direction_must_remain_nonzero_at_native_float_precision() {
        assert!(
            parse(json!({"operation":"tiny","action":"move","dx":1e-100,"dy":0,"hold_ms":1}))
                .is_err()
        );
    }
}
