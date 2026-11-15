//! Optional native client research endpoint. No OS control or gameplay writes.
mod categories;
mod recipe_types;
mod recipes;
mod requirements;
mod selection;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use enshwick::content::{items, recipes::RecipeLayout, types, vanilla};
use enshwick::{Host, Load, Mod, Off};
use policy::mcp::{Tool, Tools};
use serde_json::{Map, Value, json};

const WAIT: Duration = Duration::from_secs(2);

#[derive(Default, Debug)]
struct Observation {
    at: Option<Instant>,
    value: Value,
    sequence: u64,
    history: VecDeque<Value>,
    dropped: u64,
}

impl Observation {
    fn record(&mut self, value: Value) -> bool {
        let left_game = self.value["screen"] == "InGame" && value["screen"] != "InGame";
        let changed = self.at.is_none()
            || self.value["screen"] != value["screen"]
            || self.value["host"] != value["host"]
            || self.value["hud_screen"] != value["hud_screen"];
        if changed {
            self.sequence = self.sequence.saturating_add(1);
            if self.history.len() == policy::client::HISTORY {
                self.history.pop_front();
                self.dropped = self.dropped.saturating_add(1);
            }
            self.history.push_back(json!({"sequence":self.sequence,"screen":value["screen"],"hud_screen":value["hud_screen"],"host":value["host"]}));
        }
        self.value = value;
        self.at = Some(Instant::now());
        left_game
    }

    fn snapshot(&self) -> Value {
        let age = self.at.map(|at| at.elapsed().as_millis());
        json!({"observation":self.value,"age_ms":age,"stale":age.is_none_or(|age|age>1000),"sequence":self.sequence,"entity_world":"client","identity_scope":"client UI observation only; never use these ids as server-world selectors; reacquire after screen/session changes"})
    }

    fn events(&self, after: u64, limit: usize) -> Value {
        let events: Vec<_> = self
            .history
            .iter()
            .filter(|e| e["sequence"].as_u64().is_some_and(|n| n > after))
            .take(limit)
            .cloned()
            .collect();
        let next = events
            .last()
            .and_then(|e| e["sequence"].as_u64())
            .unwrap_or(after);
        let oldest = self.history.front().and_then(|e| e["sequence"].as_u64());
        json!({"events":events,"next_after":next,"latest_sequence":self.sequence,"dropped_total":self.dropped,"cursor_gap":oldest.is_some_and(|n|after<n.saturating_sub(1)),"scope":"screen/host changes only, not all gameplay events"})
    }
}

#[derive(Debug)]
enum Request {
    Item(String),
    Recipe(String, usize),
}

struct Pending {
    request: Request,
    cancel: Arc<AtomicBool>,
    reply: mpsc::SyncSender<Result<Value, String>>,
}

#[derive(Default)]
struct Shared {
    observation: Mutex<Observation>,
    pending: Mutex<Option<Pending>>,
    selection: Mutex<policy::client_selection::Evidence>,
}

struct ClientTools {
    shared: Arc<Shared>,
    list: Vec<Tool>,
    input: bool,
    recipes: bool,
    selection_sources: [bool; 2],
}

impl ClientTools {
    fn new(shared: Arc<Shared>, input: bool, recipes: bool) -> Self {
        Self {
            shared,
            input,
            recipes,
            selection_sources: [false; 2],
            list: vec![
                Tool {
                    name: "client_capabilities",
                    description: "Optional native research companion. No screenshots, OS input, camera, gameplay mutations, selected recipe or local-player world inspection API.",
                    schema: json!({"type":"object"}),
                },
                Tool {
                    name: "client_state",
                    description: "Latest native UI-frame snapshot: screen, interaction prompt, screen host/template and local held gameplay controls. A null template may mean no host, first-frame/unresolved metadata or unavailable reader. Stale/unavailable observations explicit; no world authority or current inner selection inferred.",
                    schema: json!({"type":"object"}),
                },
                Tool {
                    name: "client_events",
                    description: "Bounded screen/host change history. Cursor and overwritten-event count; not a complete gameplay event stream.",
                    schema: policy::client::events_schema(),
                },
                Tool {
                    name: "client_selection_events",
                    description: "Bounded observations of named client-world UI events. Requested HUD screens and station recipe-selection events are not verified current tabs, passive recipe highlights or applied production changes. Client entity IDs are never server-world selectors.",
                    schema: policy::client::events_schema(),
                },
                Tool {
                    name: "client_item",
                    description: "One vanilla item debug name resolved with localized client text. Reads vanilla archive; no effective mod/instance semantics or craftability inferred.",
                    schema: policy::client::item_schema(false),
                },
                Tool {
                    name: "client_recipes",
                    description: "Vanilla recipe metadata for one output item's debug name, at most 8 recipes. Raw stacks, typed ingredients, at most 32 members per category and declared prop IDs/shelter flag. Missing categories/truncation explicit; raw scaling and full requirement/eligibility semantics unverified. Prop type IDs are not item or entity selectors. Not current UI selection or craftability. Whole archive allocation precedes parsing caps.",
                    schema: policy::client::item_schema(true),
                },
            ],
        }
    }

    fn request(&self, request: Request) -> Result<Value, String> {
        let (tx, rx) = mpsc::sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut pending = self
                .shared
                .pending
                .lock()
                .map_err(|_| "client queue poisoned")?;
            if pending.is_some() {
                return Err("client request queue busy".into());
            }
            *pending = Some(Pending {
                request,
                cancel: cancel.clone(),
                reply: tx,
            });
        }
        match rx.recv_timeout(WAIT) {
            Ok(value) => value,
            Err(_) => {
                cancel.store(true, Ordering::Release);
                let mut pending = self
                    .shared
                    .pending
                    .lock()
                    .map_err(|_| "client queue poisoned")?;
                if pending
                    .as_ref()
                    .is_some_and(|job| Arc::ptr_eq(&job.cancel, &cancel))
                {
                    *pending = None;
                }
                Err("client UI frame did not answer within 2 seconds; queued read cancelled; an already running read may finish".into())
            }
        }
    }
}

impl Tools for ClientTools {
    fn list(&self) -> &[Tool] {
        &self.list
    }
    fn call(&self, name: &str, args: &Map<String, Value>) -> Result<Value, String> {
        let tool = self
            .list
            .iter()
            .find(|tool| tool.name == name)
            .ok_or("unknown client tool")?;
        policy::args::check(&tool.schema, args)?;
        match name {
            "client_capabilities" => Ok(
                json!({"role":"optional_client_companion","version":env!("CARGO_PKG_VERSION"),"enshwick":"1.0.0-alpha.15","compiled_framework_revision":"acd5550c607a38c983433f9807cc95564244fe0b","compiled_framework_abi":format!("{}.{}",enshwick::abi::ABI_MAJOR,enshwick::abi::ABI_MINOR),"framework_extension":"pinned nightly SDK; compiled provenance is not live loader verification","screen_host_template_observation":true,"ui_observation":true,"local_input":self.input,"recipe_layout":self.recipes,"client_event_sources":{"open_hud_screen":self.selection_sources[0],"station_recipe_selection":self.selection_sources[1]},"semantic_gameplay_actions":[],"unavailable":["screenshots","keyboard/mouse injection","camera control","current selected tab","passively highlighted crafting recipe","authoritative world/player state"],"limits":{"history":policy::client::HISTORY,"pending_requests":1,"wait_seconds":2,"text_bytes":policy::client::TEXT,"selection_history":policy::client_selection::HISTORY},"live_verified":false}),
            ),
            "client_state" => Ok(self
                .shared
                .observation
                .lock()
                .map_err(|_| "client observation poisoned")?
                .snapshot()),
            "client_events" => {
                let (after, limit) = policy::client::events(args)?;
                Ok(self
                    .shared
                    .observation
                    .lock()
                    .map_err(|_| "client observation poisoned")?
                    .events(after, limit))
            }
            "client_selection_events" => {
                let (after, limit) = policy::client::events(args)?;
                Ok(self
                    .shared
                    .selection
                    .lock()
                    .map_err(|_| "client selection evidence poisoned")?
                    .report(after, limit))
            }
            "client_item" => {
                let (item, _) = policy::client::item(args, false)?;
                self.request(Request::Item(item))
            }
            "client_recipes" => {
                let (item, limit) = policy::client::item(args, true)?;
                self.request(Request::Recipe(item, limit))
            }
            _ => Err("unknown client tool".into()),
        }
    }
}

fn resolve(
    host: Host,
    layout: Option<RecipeLayout>,
    input_type: Option<u32>,
    request: Request,
) -> Result<Value, String> {
    let name = match &request {
        Request::Item(name) | Request::Recipe(name, _) => name,
    };
    let guid = vanilla::item(name).map_err(|error| error.to_string())?;
    match request {
        Request::Item(name) => {
            let language = host.language();
            let text = items::vanilla_name(guid, language);
            Ok(
                json!({"debug_name":name,"guid":guid.to_string(),"item_id":enshwick::ItemId::from_guid(guid).0,"language":language.code(),"localized_name":text.map(policy::client::text),"localized_name_truncated":text.is_some_and(|s|s.len()>policy::client::TEXT),"source":"vanilla_archive_and_client_localization","effective_definition_verified":false}),
            )
        }
        Request::Recipe(_, limit) => {
            let layout = layout.ok_or("recipe layout unavailable")?;
            let key = types::RECIPE_REGISTRY_RESOURCE
                .key(enshwick::content::recipes::RECIPE_REGISTRY)
                .map_err(|e| e.to_string())?;
            let bytes = host
                .game_resource(key)
                .ok_or("vanilla recipe registry unavailable")?;
            recipes::parse(
                layout,
                &bytes,
                guid,
                limit,
                input_type.ok_or("recipe input type unavailable")?,
                key.type_hash,
            )
        }
    }
}

pub struct ResearchClient {
    tools: ClientTools,
}

impl Mod for ResearchClient {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let shared = Arc::new(Shared::default());
        if load.host().build().side != enshwick::abi::Side::CLIENT {
            return Ok(Self {
                tools: ClientTools::new(shared, false, false),
            });
        }
        let input = load.input().ok();
        let selection_sources = selection::register(load, &shared);
        let layout = RecipeLayout::resolve(load).ok();
        let input_type =
            enshwick::typed::ResourceType::<enshwick::typed::RecipeInputResource>::resolve(load)
                .ok()
                .map(|ty| ty.key(enshwick::content::Guid([0; 16])).type_hash);
        let host = load.host();
        let frames = shared.clone();
        load.on_frame(move |frame|{
            let prompt=frame.prompt().map(|prompt|json!({"title":policy::client::text(&prompt.title),"title_truncated":prompt.title.len()>policy::client::TEXT,"verb":prompt.verb}));
            let held=input.and_then(|input|input.local_held()).map(|mask|mask.iter().map(|action|action.name()).collect::<Vec<_>>());
            let value=json!({"screen":format!("{:?}",frame.screen()),"hud_screen":frame.open_screen().map(|screen|screen.name()),"prompt":prompt,"host":frame.screen_host().map(|entity|entity.0),"host_position":frame.screen_host_position(),"host_template":frame.screen_host_template().map(|guid|guid.to_string()),"held_actions":held});
            let left_game=frames.observation.lock().map(|mut observation|observation.record(value)).unwrap_or(false);
            if left_game && let Ok(mut evidence)=frames.selection.lock(){evidence.end_session();}
            let job=frames.pending.lock().ok().and_then(|mut job|job.take());
            if let Some(job)=job && !job.cancel.load(Ordering::Acquire) {
                let _=job.reply.send(resolve(host,layout,input_type,job.request));
            }
        })?;
        let mut tools = ClientTools::new(
            shared,
            input.is_some(),
            layout.is_some() && input_type.is_some(),
        );
        tools.selection_sources = selection_sources;
        Ok(Self { tools })
    }

    fn ready(&'static self, host: Host) {
        if host.build().side != enshwick::abi::Side::CLIENT {
            return;
        }
        let config = mcp_transport::Config::from_env(
            "ENSHROUDED_MCP_CLIENT_PORT",
            "ENSHROUDED_MCP_CLIENT_TOKEN",
            8754,
        );
        if let Err(error) = std::thread::Builder::new()
            .name("enshrouded-mcp-client".into())
            .spawn(move || mcp_transport::run(&config, &self.tools, env!("CARGO_PKG_VERSION")))
        {
            enshwick::error!("could not start client listener: {error}");
        }
    }
}

#[cfg(not(test))]
enshwick::declare_mod!(ResearchClient);

#[cfg(test)]
mod tests {
    use super::*;

    static FRAME_STATE: Mutex<Option<Arc<Shared>>> = Mutex::new(None);
    struct Recorder;
    impl Mod for Recorder {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let companion = ResearchClient::init(load)?;
            *FRAME_STATE.lock().unwrap() = Some(companion.tools.shared);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Recorder,
        r#"id = "test.client-research"
side = "both""#
    );

    #[test]
    fn native_client_frames_capture_screen_prompt_and_clear_host_on_close() {
        use enshwick::abi::{Side, Status};
        use enshwick::ui::HudScreenType;
        let loader = enshwick::test::FakeHost::new().side(Side::CLIENT).install();
        // SAFETY: these are this test module's framework-generated exports.
        let status =
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) };
        assert_eq!(status, Status::OK);
        loader.open_screen(Some(HudScreenType::StorageChest));
        loader.host_screen(Some(enshwick::Entity(42)));
        loader.show_prompt(Some(("Test chest", 7)));
        let _ = loader.draw_frame();
        let shared = FRAME_STATE.lock().unwrap().as_ref().unwrap().clone();
        let state = shared.observation.lock().unwrap().snapshot();
        assert_eq!(state["observation"]["hud_screen"], "StorageChest");
        assert_eq!(state["observation"]["host"], 42);
        assert_eq!(state["observation"]["prompt"]["title"], "Test chest");
        assert!(
            state["observation"]
                .get("host_template")
                .is_some_and(Value::is_null)
        );
        let template = enshwick::content::Guid([17; 16]);
        loader.read_screen_host(template, [1.0, 2.0, 3.0]);
        let _ = loader.draw_frame();
        let state = shared.observation.lock().unwrap().snapshot();
        assert_eq!(state["observation"]["host_template"], template.to_string());
        assert_eq!(state["sequence"], 1); // Metadata freshness is not a new screen/host event.

        loader.open_screen(Some(HudScreenType::InGame));
        loader.host_screen(None);
        loader.show_prompt(None);
        let _ = loader.draw_frame();
        let state = shared.observation.lock().unwrap().snapshot();
        assert!(state["observation"]["host"].is_null());
        assert!(state["observation"]["host_template"].is_null());
        assert!(state["observation"]["prompt"].is_null());
        assert_eq!(state["sequence"], 2);
    }

    #[test]
    fn compiled_framework_provenance_is_separate_from_live_verification() {
        let tools = ClientTools::new(Arc::default(), false, false);
        let value = tools.call("client_capabilities", &Map::new()).unwrap();
        assert_eq!(value["enshwick"], "1.0.0-alpha.15");
        assert_eq!(
            value["compiled_framework_revision"],
            "acd5550c607a38c983433f9807cc95564244fe0b"
        );
        assert_eq!(value["compiled_framework_abi"], "11.30");
        assert_eq!(value["screen_host_template_observation"], true);
        assert_eq!(value["live_verified"], false);
        assert_eq!(value["semantic_gameplay_actions"], json!([]));
    }

    #[test]
    fn unanswered_request_is_cancelled_and_does_not_occupy_queue() {
        let tools = ClientTools::new(Arc::default(), false, false);
        assert!(
            tools
                .request(Request::Item("test".into()))
                .unwrap_err()
                .contains("cancelled")
        );
        assert!(tools.shared.pending.lock().unwrap().is_none());
    }

    #[test]
    fn snapshots_are_unavailable_before_first_frame_and_history_reports_overwrite() {
        let mut observation = Observation::default();
        assert_eq!(observation.snapshot()["stale"], true);
        for i in 0..70 {
            observation.record(json!({"screen":"InGame","hud_screen":"Backpack","host":i}));
        }
        assert_eq!(observation.history.len(), 64);
        let events = observation.events(0, 2);
        assert_eq!(events["dropped_total"], 6);
        assert_eq!(events["cursor_gap"], true);
        assert_eq!(events["events"].as_array().unwrap().len(), 2);
        assert_eq!(events["next_after"], 8);
        assert_eq!(observation.snapshot()["stale"], false);
    }

    #[test]
    fn no_action_tools_and_invalid_selectors_never_enqueue() {
        let tools = ClientTools::new(Arc::default(), false, false);
        assert_eq!(tools.list().len(), 6);
        for (name, args) in [
            ("client_item", json!({"item":""})),
            ("client_recipes", json!({"item":"wood","limit":9})),
            ("client_state", json!({"key":"Escape"})),
            ("client_events", json!({"limit":0})),
        ] {
            assert!(tools.call(name, args.as_object().unwrap()).is_err());
        }
        assert!(tools.shared.pending.lock().unwrap().is_none());
        assert!(tools.call("client_press", &Map::new()).is_err());
    }

    #[test]
    fn stale_snapshot_does_not_claim_current_ui_and_identical_frames_make_no_events() {
        let mut observation = Observation::default();
        let value = json!({"screen":"InGame","hud_screen":"Backpack","host":1});
        observation.record(value.clone());
        observation.record(value);
        assert_eq!(observation.sequence, 1);
        observation.at = Some(Instant::now() - Duration::from_secs(3));
        assert_eq!(observation.snapshot()["stale"], true);
    }
}
