//! Admission for an inactive native lifecycle qualifier. No credentials or World.
use serde_json::{Map, Value, json};
use std::{fmt, net::SocketAddr, time::Duration};

pub const MAX_OPERATIONS: usize = 8;
pub const MAX_REQUESTS: u32 = 32;

pub fn new_operation<'a>(
    operation: &str,
    known: impl IntoIterator<Item = &'a str>,
) -> Result<(), String> {
    label(operation)?;
    for (index, identity) in known.into_iter().enumerate() {
        if identity == operation {
            return Err("connection operation is already retained; never replay".into());
        }
        if index + 1 >= MAX_OPERATIONS {
            return Err("connection operation capacity reached".into());
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Join(String),
    Leave,
    Cancel(String),
}
#[derive(Clone, Debug)]
pub struct Spec {
    pub operation: String,
    pub action: Action,
    pub dry: bool,
    pub preview: Option<String>,
    pub confirm: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    InGame,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Owned {
    Idle,
    Pending(String),
    Joined { operation: String, session: u64 },
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Context {
    pub screen: Screen,
    pub age_ms: Option<u64>,
    pub sequence: u64,
    pub native_session: Option<u64>,
    pub native_fresh: bool,
    pub native_age_ms: Option<u64>,
    pub peer_matches: Option<bool>,
    pub owned: Owned,
}

pub struct OperationObservation<'a> {
    pub operation: &'a str,
    pub action: &'a Action,
    pub pending: bool,
    pub joined_session: Option<u64>,
    pub left: bool,
}
pub fn ownership<'a>(
    screen: Screen,
    operations: impl IntoIterator<Item = OperationObservation<'a>>,
) -> Owned {
    let mut owned = Owned::Idle;
    for operation in operations {
        owned = match operation.action {
            Action::Join(_)
                if owned == Owned::Idle
                    && operation.pending
                    && !operation.left
                    && operation.joined_session.is_none() =>
            {
                Owned::Pending(operation.operation.into())
            }
            Action::Join(_)
                if owned == Owned::Idle
                    && !operation.pending
                    && !operation.left
                    && operation.joined_session.is_some_and(|session| session != 0) =>
            {
                Owned::Joined {
                    operation: operation.operation.into(),
                    session: operation.joined_session.unwrap(),
                }
            }
            Action::Leave
                if matches!(owned, Owned::Joined { .. })
                    && operation.left
                    && !operation.pending
                    && screen == Screen::Menu =>
            {
                Owned::Idle
            }
            _ => Owned::Unknown,
        };
    }
    owned
}
/// First matching copied observation only; this is not atomic peer authentication.
pub fn bind_join_session(existing: Option<u64>, joined: bool, context: &Context) -> Option<u64> {
    existing.or_else(|| {
        (joined
            && context.sequence != 0
            && context.screen == Screen::InGame
            && context.age_ms.is_some_and(|age| age <= 1000)
            && context.native_fresh
            && context.native_age_ms.is_some_and(|age| age <= 1000)
            && context.peer_matches == Some(true))
        .then_some(context.native_session)
        .flatten()
        .filter(|session| *session != 0)
    })
}

pub fn read_object(path: &std::path::Path) -> Result<Option<(Value, Duration)>, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("qualification file metadata unavailable".into()),
    };
    if !metadata.is_file() || metadata.len() > 4096 {
        return Err("qualification requires a bounded regular JSON file".into());
    }
    let modified = metadata
        .modified()
        .map_err(|_| "qualification request age unavailable")?;
    let file = std::fs::File::open(path).map_err(|_| "qualification file unavailable")?;
    let value = json_from_reader(file)?;
    let age = modified
        .elapsed()
        .map_err(|_| "qualification request age unavailable")?;
    Ok(Some((value, age)))
}

fn json_from_reader(reader: impl std::io::Read) -> Result<Value, String> {
    use std::io::Read as _;
    let mut bytes = Vec::new();
    reader
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| "qualification file unavailable")?;
    if bytes.len() > 4096 {
        return Err("qualification file grew past its bound".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "qualification JSON unavailable".into())
}

pub fn schema() -> Value {
    json!({"type":"object","properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "action":{"type":"string","enum":["join","leave","cancel"]},
        "profile":{"type":"string","minLength":1,"maxLength":64},
        "target_operation":{"type":"string","minLength":1,"maxLength":64},
        "dry":{"type":"boolean"},"preview":{"type":"string","minLength":1,"maxLength":2048},
        "confirm":{"type":"string"}},"required":["operation","action"],"additionalProperties":false})
}
fn label(value: &str) -> Result<String, String> {
    crate::client_control::operation(json!({"operation":value}).as_object().unwrap())
}
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let operation = crate::client_control::operation(args)?;
    let (dry, preview, confirm) = crate::client_control::mode(args)?;
    let profile = args.get("profile").and_then(Value::as_str);
    let target = args.get("target_operation").and_then(Value::as_str);
    let action = match args["action"].as_str().unwrap() {
        "join" if target.is_none() => Action::Join(label(
            profile.ok_or("join requires an allowlisted profile label")?,
        )?),
        "leave" if profile.is_none() && target.is_none() => Action::Leave,
        "cancel" if profile.is_none() => Action::Cancel(label(
            target.ok_or("cancel requires a retained target operation")?,
        )?),
        _ => return Err("connection action has unrelated profile or target arguments".into()),
    };
    Ok(Spec {
        operation,
        action,
        dry,
        preview,
        confirm,
    })
}
pub fn payload(action: &Action) -> Value {
    match action {
        Action::Join(profile) => json!({"action":"join","profile":profile}),
        Action::Leave => json!({"action":"leave"}),
        Action::Cancel(target) => json!({"action":"cancel","target_operation":target}),
    }
}
pub fn applicable(spec: &Spec, context: &Context, configured_profile: &str) -> Result<(), String> {
    if context.sequence == 0 || context.age_ms.is_none_or(|age| age > 1000) {
        return Err("fresh copied UI context is required".into());
    }
    match &spec.action {
        Action::Join(profile) => {
            if profile != configured_profile {
                return Err("connection profile is not configured".into());
            }
            if context.screen != Screen::Menu
                || context.owned != Owned::Idle
                || context.native_session.is_some_and(|session| session != 0)
            {
                return Err("join requires Menu, idle ownership and no observed world session; never leave implicitly".into());
            }
        }
        Action::Leave => {
            if context.screen != Screen::InGame
                || !matches!(&context.owned, Owned::Joined { session, .. } if *session != 0 && context.native_session == Some(*session))
                || !context.native_fresh
                || context.native_age_ms.is_none_or(|age| age > 1000)
                || context.peer_matches != Some(true)
            {
                return Err("leave requires this probe's observed join and fresh matching native world context".into());
            }
        }
        Action::Cancel(target) => {
            if context.owned != Owned::Pending(target.clone()) {
                return Err(
                    "cancel requires this probe's retained pending join; no implicit leave".into(),
                );
            }
        }
    }
    Ok(())
}
pub fn preview(boot: &str, spec: &Spec, context: &Context) -> String {
    json!({"boot":boot,"operation":spec.operation,"request":payload(&spec.action),
        "screen":format!("{:?}",context.screen),"sequence":context.sequence,
        "native_session":context.native_session,"native_fresh":context.native_fresh,
        "peer_matches":context.peer_matches,"owned":format!("{:?}",context.owned)})
    .to_string()
}
pub fn authorize(
    boot: &str,
    spec: &Spec,
    context: &Context,
    configured_profile: &str,
    flag_age: Option<Duration>,
    queued_age: Duration,
) -> Result<(), String> {
    applicable(spec, context, configured_profile)?;
    if spec.dry {
        return Ok(());
    }
    crate::write_gate::check(false, true, flag_age, spec.confirm.as_deref())?;
    crate::client_control::deadline(queued_age)?;
    crate::client_control::exact(spec.preview.as_deref(), &preview(boot, spec, context))
}

pub struct Profile {
    pub label: String,
    query: SocketAddr,
}
impl fmt::Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Profile")
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}
impl Profile {
    pub fn query(&self) -> String {
        self.query.to_string()
    }
    /// This is copied target-text agreement, never independent peer authentication.
    pub fn matches_peer(&self, server: Option<&str>) -> Option<bool> {
        server
            .and_then(|text| text.parse::<SocketAddr>().ok())
            .map(|address| address == self.query)
    }
}
pub fn profile(value: &Value) -> Result<Profile, String> {
    let args = value.as_object().ok_or("profile must be an object")?;
    crate::args::check(
        &json!({"properties":{"profile":{"type":"string"},"query":{"type":"string"}},"required":["profile","query"]}),
        args,
    )?;
    let name = label(args["profile"].as_str().unwrap())?;
    let query = args["query"].as_str().unwrap();
    if query.len() > 64 {
        return Err("profile query address is too long".into());
    }
    let query: SocketAddr = query
        .parse()
        .map_err(|_| "profile requires a loopback IPv4 query address")?;
    if !query.is_ipv4() || !query.ip().is_loopback() || query.port() == 0 {
        return Err("qualification profile requires a nonzero loopback IPv4 query port".into());
    }
    Ok(Profile { label: name, query })
}

pub fn provider(major: u32, minor: u32, bytes: u32, compiled_bytes: u32) -> Result<(), String> {
    if major == 12 && minor == 16 && bytes == compiled_bytes && bytes != 0 {
        Ok(())
    } else {
        Err("qualification requires the exact compiled ABI12.16 provider header".into())
    }
}
pub fn lease(age: Duration) -> Result<(), String> {
    if age < Duration::from_secs(180) {
        Ok(())
    } else {
        Err("native qualification session lease expired; no automatic action".into())
    }
}
pub fn start_session(dir: &std::path::Path, boot: &str) -> Result<(), String> {
    use std::io::Write;
    label(boot)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("session.json"))
        .map_err(|_| "fresh connection session unavailable; never replay")?;
    let bytes = serde_json::to_vec(&json!({"kind":"connection_probe_boot","boot":boot,
        "native_ids_reconstructed":false}))
    .map_err(|_| "connection session unavailable")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "connection session unconfirmed; never replay".into())
}

/// Caller serializes admissions on the client frame and rechecks authority after fsync.
/// An unconfirmed/partial reservation never authorizes a replay.
pub fn reserve(
    dir: &std::path::Path,
    boot: &str,
    spec: &Spec,
    context: &Context,
) -> Result<(), String> {
    use std::io::Write;
    label(&spec.operation)?;
    label(boot)?;
    if spec.dry {
        return Err("dry connection request cannot reserve an operation".into());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|_| "connection reservation directory unavailable")?;
    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|_| "connection reservations unavailable")?;
        if !entry
            .file_type()
            .map_err(|_| "connection reservation type unavailable")?
            .is_file()
        {
            return Err("unexpected connection reservation member".into());
        }
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or("unexpected connection reservation name")?;
        label(
            name.strip_prefix("connection-")
                .and_then(|n| n.strip_suffix(".json"))
                .ok_or("unexpected connection reservation name")?,
        )?;
        count += 1;
    }
    if count >= MAX_OPERATIONS {
        return Err("connection operation capacity reached".into());
    }
    let mut bytes=serde_json::to_vec(&json!({"kind":"connection_intent","boot":boot,"operation":spec.operation,
        "request":payload(&spec.action),"preview":preview(boot,spec,context),"automatic_retry":false})).map_err(|_|"connection intent unavailable")?;
    bytes.push(b'\n');
    crate::write_gate::journal_budget(0, bytes.len(), 0)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(format!("connection-{}.json", spec.operation)))
        .map_err(|_| "connection identity unavailable or already reserved; never replay")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "connection intent unconfirmed; never replay".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn reader_consumption_stays_bounded_when_input_exceeds_metadata_claim() {
        let mut reader = std::io::Cursor::new(vec![b'x'; 16384]);
        assert!(super::json_from_reader(&mut reader).is_err());
        assert!(
            reader.position() <= 4097,
            "reader consumed past the bounded sentinel byte"
        );
    }
}
