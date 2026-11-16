//! Optional native client endpoint. Published client commands; no OS input or World access.
mod categories;
mod control;
mod markers;
mod native_snapshot;
mod recipe_types;
mod recipes;
mod requirements;
mod selection;

use policy::write_gate::PendingWrite;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use enshwick::content::{items, recipes::RecipeLayout, types, vanilla};
use enshwick::{Host, Load, Mod, Off};
use policy::mcp::{Tool, Tools};
use serde_json::{Map, Value, json};

const WAIT: Duration = Duration::from_secs(2);

fn prompt_value(title: &str, verb: u32, language: &str, text: Option<&str>) -> Value {
    json!({"title":policy::client::text(title),"title_truncated":title.len()>policy::client::TEXT,
        "verb":verb,"verb_text":text.map(policy::client::text),
        "verb_text_state":if text.is_some(){"observed"}else{"unavailable"},
        "verb_text_truncated":text.is_some_and(|s|s.len()>policy::client::TEXT),
        "language":language,"verb_text_source":"client_archive_localization_with_English_fallback"})
}

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
    Command(policy::client_control::Spec, Instant),
    Cancel(policy::client_control::CancelSpec, Instant),
    Item(String),
    Recipe(policy::client::RecipeSelector, usize),
    Markers(policy::client_markers::Spec),
}

struct Pending {
    request: Request,
    cancel: Arc<PendingWrite>,
    reply: mpsc::SyncSender<Result<Value, String>>,
}

#[derive(Default)]
struct Shared {
    control: Mutex<control::State>,
    observation: Mutex<Observation>,
    pending: Mutex<Option<Pending>>,
    selection: Mutex<policy::client_selection::Evidence>,
    markers: Mutex<markers::State>,
}

struct ClientTools {
    shared: Arc<Shared>,
    list: Vec<Tool>,
    input: bool,
    recipes: bool,
    selection_sources: [bool; 2],
    mod_dir: Option<std::path::PathBuf>,
    marker_ui: bool,
    native_input: Option<enshwick::input::Input>,
}

impl ClientTools {
    fn new(shared: Arc<Shared>, input: bool, recipes: bool) -> Self {
        Self {
            shared,
            input,
            recipes,
            selection_sources: [false; 2],
            mod_dir: None,
            marker_ui: false,
            native_input: None,
            list: vec![
                Tool {
                    name: "client_command",
                    description: "Deliberate native client action: open/close in-game screen, choose tab, bounded look/move or highlight recipe (never craft). Caller operation identity required; dry default. Real requests require exact current session/screen/tab preview, companion ALLOW_WRITES and confirm: server-write. Once-only durable identity: inspect client_command_status after lost replies; never retry a new identity blindly. Frame queues to SDK's later client execution phase, no server World access. SDK observed movement means hold completed, not arrival; look acceptance is not rendered proof. No join, credentials, logout, arbitrary input or offsets.",
                    schema: policy::client_control::schema(),
                },
                Tool {
                    name: "client_command_status",
                    description: "Read retained client command outcome by caller operation identity. No dispatch, ID reconstruction or retry. Prior-boot durable identities report unavailable. Native pending/applied/observed/cancelled is distinct from rendering, arrival, craftability or server authority.",
                    schema: policy::client_control::status_schema(),
                },
                Tool {
                    name: "client_command_cancel",
                    description: "Cancel only this companion's retained native command. Dry default returns cancellation preview. Real request needs exact preview, companion write flag and confirm: server-write; at most one native cancellation attempt per operation. SDK outcome must be observed separately. Never reconstruct an ID or cancel another mod's command.",
                    schema: policy::client_control::cancel_schema(),
                },
                Tool {
                    name: "client_snapshot",
                    description: "Copied native client state from the SDK, safe on the listener thread: session, freshness, per-part availability, HUD screen, actual inner menu tab, held controls, client player and highlighted recipe. Null with reads=false is unavailable; null with reads=true means none observed. Zero recipe is the SDK no-highlight sentinel. Client IDs never select server entities. Does not submit commands or prove rendering, connection, craftability or world authority.",
                    schema: json!({"type":"object"}),
                },
                Tool {
                    name: "client_map_markers",
                    description: "Transient mod-owned map annotation. No args reads submitted cache, not renderer state. Supply markers (at most one; empty clears), dry defaults true. Real requests require exact preview, companion-folder ALLOW_WRITES and confirm: server-write. Frame-queued only, fresh client epoch/revision; pending timeout cancels before start. SDK acceptance is not rendered proof. Leave cleanup removes only this mod's annotation without requiring a new flag; audit failures do not block cleanup. No saved marker edits, fast travel, scene raycast, world mutation or OS input.",
                    schema: policy::client_markers::schema(),
                },
                Tool {
                    name: "client_capabilities",
                    description: "Optional native research companion capabilities. Copied native client snapshot availability is separate from per-part runtime reads. Published bounded native client commands are separate from runtime applicability and live verification. No screenshots, OS input, crafting execution, join/logout or authoritative world inspection.",
                    schema: json!({"type":"object"}),
                },
                Tool {
                    name: "client_state",
                    description: "Latest native UI-frame snapshot: screen, interaction prompt with bounded localized verb text (English fallback; missing stays unavailable), screen host/template and local held gameplay controls. A null template may mean no host, first-frame/unresolved metadata or unavailable reader. Stale/unavailable observations explicit; no world authority or current inner selection inferred.",
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
                    description: "Vanilla recipe metadata for exactly one output item debug name or raw recipe_id (0..4294967295), at most 8 recipes. ID lookup counts all matching records; collisions/truncation are explicit and never imply a unique UI selection. Raw recipe/workshop definition IDs from archive records, input/output stack item IDs and GUIDs, typed ingredients, at most 32 members per category and declared prop IDs/shelter flag. Missing categories/truncation explicit; raw scaling and full requirement/eligibility semantics unverified. Workshop and prop type IDs are not item or entity selectors. Not current UI selection or craftability. Whole archive allocation precedes parsing caps.",
                    schema: policy::client::recipe_schema(),
                },
            ],
        }
    }

    fn request(&self, request: Request) -> Result<Value, String> {
        let (tx, rx) = mpsc::sync_channel(1);
        let cancel = Arc::new(PendingWrite::default());
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
                let cancelled = cancel.cancel();
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
                Err(if cancelled {
                    "client UI frame did not answer within 2 seconds; queued request cancelled before execution".into()
                } else {
                    "client UI request started but did not answer within 2 seconds; outcome unknown; inspect status/journal and do not blindly retry".into()
                })
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
            "client_command" => {
                let spec = policy::client_control::check(args)?;
                control::admission(spec.dry, spec.confirm.as_deref(), self.mod_dir.as_deref())?;
                if self.native_input.is_none() {
                    return Err("native client command API unavailable".into());
                }
                self.request(Request::Command(spec, Instant::now()))
            }
            "client_command_status" => {
                let operation = policy::client_control::operation(args)?;
                self.shared
                    .control
                    .lock()
                    .map_err(|_| "client command state poisoned")?
                    .report(&operation, self.mod_dir.as_deref())
            }
            "client_command_cancel" => {
                let spec = policy::client_control::cancel(args)?;
                control::admission(spec.dry, spec.confirm.as_deref(), self.mod_dir.as_deref())?;
                self.request(Request::Cancel(spec, Instant::now()))
            }
            "client_snapshot" => native_snapshot::read(self.native_input),
            "client_map_markers" => {
                let spec = policy::client_markers::check(args)?;
                if spec.markers.is_none() {
                    return Ok(self
                        .shared
                        .markers
                        .lock()
                        .map_err(|_| "marker state poisoned")?
                        .report());
                }
                if !self.marker_ui {
                    return Err("native client marker UI unavailable".into());
                }
                markers::admission(&spec, self.mod_dir.as_deref())?;
                self.request(Request::Markers(spec))
            }
            "client_capabilities" => Ok(
                json!({"role":"optional_client_companion","version":env!("CARGO_PKG_VERSION"),"enshwick":"1.0.0-alpha.17","compiled_framework_revision":"6a151a9550ed7a6ade99c23fa817a91e25b71bc9","compiled_framework_abi":format!("{}.{}",enshwick::abi::ABI_MAJOR,enshwick::abi::ABI_MINOR),"framework_extension":"clean official alpha.17 SDK; compiled provenance is not live loader verification","screen_host_template_observation":true,"ui_observation":true,"local_input":self.input,"native_client_snapshot":self.native_input.is_some(),"native_client_commands":self.native_input.is_some(),"native_command_live_verified":false,"native_command_availability":"published API adapter; runtime applicability and availability checked per request","recipe_layout":self.recipes,"client_event_sources":{"open_hud_screen":self.selection_sources[0],"station_recipe_selection":self.selection_sources[1]},"native_map_marker_ui":self.marker_ui,"native_map_marker_rendered_verified":false,"semantic_ui_actions":([self.marker_ui.then_some("client_map_markers"),self.native_input.map(|_|"client_command")].into_iter().flatten().collect::<Vec<_>>()),"semantic_gameplay_actions":if self.native_input.is_some(){vec!["client_command.look","client_command.move"]}else{vec![]},"unavailable":["screenshots","keyboard/mouse injection","server join/credentials/logout","rendered evidence","authoritative world/player state"],"limits":{"history":policy::client::HISTORY,"pending_requests":1,"wait_seconds":2,"text_bytes":policy::client::TEXT,"selection_history":policy::client_selection::HISTORY,"map_markers":1,"client_command_operations":policy::client_control::MAX_OPERATIONS,"move_hold_ms":250,"look_radians_per_axis":std::f32::consts::FRAC_PI_2},"live_verified":false}),
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
                let (item, limit) = policy::client::recipe(args)?;
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
    match request {
        Request::Item(name) => {
            let guid = vanilla::item(&name).map_err(|error| error.to_string())?;
            let language = host.language();
            let text = items::vanilla_name(guid, language);
            Ok(
                json!({"debug_name":name,"guid":guid.to_string(),"item_id":enshwick::ItemId::from_guid(guid).0,"language":language.code(),"localized_name":text.map(policy::client::text),"localized_name_truncated":text.is_some_and(|s|s.len()>policy::client::TEXT),"source":"vanilla_archive_and_client_localization","effective_definition_verified":false}),
            )
        }
        Request::Recipe(selector, limit) => {
            let selection = match selector {
                policy::client::RecipeSelector::Output(name) => recipes::Selection::Output(
                    vanilla::item(&name).map_err(|error| error.to_string())?,
                ),
                policy::client::RecipeSelector::Id(id) => recipes::Selection::Id(id),
            };
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
                selection,
                limit,
                input_type.ok_or("recipe input type unavailable")?,
                key.type_hash,
            )
        }
        Request::Markers(_) | Request::Command(..) | Request::Cancel(..) => {
            Err("UI request requires the client frame dispatcher".into())
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
        let marker_ui = load.ui().ok();
        let marker_available = marker_ui.is_some();
        let mod_dir = load.mod_dir().ok();
        let frame_dir = mod_dir.clone();
        let selection_sources = selection::register(load, &shared);
        let layout = RecipeLayout::resolve(load).ok();
        let input_type =
            enshwick::typed::ResourceType::<enshwick::typed::RecipeInputResource>::resolve(load)
                .ok()
                .map(|ty| ty.key(enshwick::content::Guid([0; 16])).type_hash);
        let host = load.host();
        let frames = shared.clone();
        load.on_frame(move |frame|{
            if let Ok(mut state)=frames.control.lock(){state.observe(input,frame_dir.as_deref());}
            let prompt=frame.prompt().map(|prompt| {
                let language=host.language();
                prompt_value(&prompt.title,prompt.verb,language.code(),host.game_text(prompt.verb,language))
            });
            let held=input.and_then(|input|input.local_held()).map(|mask|mask.iter().map(|action|action.name()).collect::<Vec<_>>());
            let value=json!({"screen":format!("{:?}",frame.screen()),"hud_screen":frame.open_screen().map(|screen|screen.name()),"prompt":prompt,"host":frame.screen_host().map(|entity|entity.0),"host_position":frame.screen_host_position(),"host_template":frame.screen_host_template().map(|guid|guid.to_string()),"held_actions":held});
            let left_game=frames.observation.lock().map(|mut observation|observation.record(value)).unwrap_or(false);
            if left_game && let Ok(mut evidence)=frames.selection.lock(){evidence.end_session();}
            if let Ok(mut state)=frames.markers.lock() {
                state.observe(frame.screen()==enshwick::ui::Screen::InGame,frame_dir.as_deref(),|list|markers::submit(marker_ui.as_ref(),list));
            }
            let job=frames.pending.lock().ok().and_then(|mut job|job.take());
            if let Some(job)=job && job.cancel.begin() {
                let result=match job.request {
                    Request::Command(spec,queued)=>frames.control.lock().map_err(|_|"client command state poisoned".to_owned()).and_then(|mut state|state.run(spec,input,frame_dir.as_deref(),queued)),
                    Request::Cancel(spec,queued)=>frames.control.lock().map_err(|_|"client command state poisoned".to_owned()).and_then(|mut state|state.cancel(spec,input,frame_dir.as_deref(),queued)),
                    Request::Markers(spec)=>frames.markers.lock().map_err(|_|"marker state poisoned".to_owned()).and_then(|mut state|state.run(spec,frame_dir.as_deref(),|list|markers::submit(marker_ui.as_ref(),list))),
                    request=>resolve(host,layout,input_type,request),
                };
                let _=job.reply.send(result);
            }
        })?;
        let mut tools = ClientTools::new(
            shared,
            input.is_some(),
            layout.is_some() && input_type.is_some(),
        );
        tools.selection_sources = selection_sources;
        tools.mod_dir = mod_dir;
        tools.marker_ui = marker_available;
        tools.native_input = input;
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

    #[test]
    fn prompt_localization_preserves_missing_empty_and_bounded_utf8() {
        let missing = prompt_value("", 7, "en-US", None);
        assert_eq!(missing["verb"], 7);
        assert!(missing["verb_text"].is_null());
        assert_eq!(missing["verb_text_state"], "unavailable");
        let empty = prompt_value("", 7, "en-US", Some(""));
        assert_eq!(empty["verb_text"], "");
        assert_eq!(empty["verb_text_state"], "observed");
        let long = "界".repeat(100);
        let bounded = prompt_value(&long, 8, "en-US", Some(&long));
        assert_eq!(bounded["verb_text"].as_str().unwrap().len(), 255);
        assert_eq!(bounded["verb_text_truncated"], true);
        assert_eq!(bounded["title_truncated"], true);
    }

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
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .game_text(7, enshwick::i18n::Language::EN_US, "Open")
            .install();
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
        assert_eq!(state["observation"]["prompt"]["verb_text"], "Open");
        assert_eq!(
            state["observation"]["prompt"]["verb_text_state"],
            "observed"
        );
        assert!(
            state["observation"]
                .get("host_template")
                .is_some_and(Value::is_null)
        );
        loader.show_prompt(Some(("Test chest", 8)));
        let _ = loader.draw_frame();
        let missing = shared.observation.lock().unwrap().snapshot();
        assert_eq!(
            missing["observation"]["prompt"]["verb_text_state"],
            "unavailable"
        );
        assert!(missing["observation"]["prompt"]["verb_text"].is_null());
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
        loader.set_client_snapshot(enshwick::abi::ClientSnapshot {
            state: enshwick::abi::ClientSnapshot::FRESH,
            session: 1,
            age_ms: 0,
            available: enshwick::abi::ClientSnapshot::SCREENS,
            hud_screen: enshwick::abi::Str::new("InGame"),
            ..enshwick::abi::ClientSnapshot::EMPTY
        });
        for cancelled in [false, true] {
            let spec = policy::client_control::check(
                json!({"operation":"test-frame-dry","action":"open_screen","target":"Map"})
                    .as_object()
                    .unwrap(),
            )
            .unwrap();
            let cancel = Arc::new(PendingWrite::default());
            if cancelled {
                assert!(cancel.cancel());
            }
            let (tx, rx) = mpsc::sync_channel(1);
            *shared.pending.lock().unwrap() = Some(Pending {
                request: Request::Command(spec, Instant::now()),
                cancel,
                reply: tx,
            });
            assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            let _ = loader.draw_frame();
            if cancelled {
                assert!(matches!(
                    rx.try_recv(),
                    Err(mpsc::TryRecvError::Disconnected)
                ));
            } else {
                assert_eq!(rx.try_recv().unwrap().unwrap()["state"], "dry");
            }
            assert!(loader.client_commands().is_empty());
        }
    }

    #[test]
    fn compiled_framework_provenance_is_separate_from_live_verification() {
        let tools = ClientTools::new(Arc::default(), false, false);
        let value = tools.call("client_capabilities", &Map::new()).unwrap();
        assert_eq!(value["enshwick"], "1.0.0-alpha.17");
        assert_eq!(
            value["compiled_framework_revision"],
            "6a151a9550ed7a6ade99c23fa817a91e25b71bc9"
        );
        assert_eq!(value["compiled_framework_abi"], "12.1");
        assert_eq!(value["screen_host_template_observation"], true);
        assert_eq!(value["live_verified"], false);
        assert_eq!(value["semantic_gameplay_actions"], json!([]));
        assert_eq!(value["native_map_marker_ui"], false);
        assert_eq!(value["semantic_ui_actions"], json!([]));
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
    fn unavailable_marker_control_and_invalid_selectors_never_enqueue() {
        let tools = ClientTools::new(Arc::default(), false, false);
        assert_eq!(tools.list().len(), 11);
        for (name, args) in [
            ("client_item", json!({"item":""})),
            ("client_recipes", json!({"item":"wood","limit":9})),
            ("client_recipes", json!({"item":"wood","recipe_id":0})),
            ("client_recipes", json!({"recipe_id":4294967296u64})),
            ("client_state", json!({"key":"Escape"})),
            ("client_snapshot", json!({"key":"Escape"})),
            ("client_command", json!({"operation":"bad","action":"join"})),
            ("client_command_status", json!({"operation":"../other"})),
            (
                "client_command_status",
                json!({"operation":"bad","dry":true}),
            ),
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
