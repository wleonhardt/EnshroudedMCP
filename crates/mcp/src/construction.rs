//! Bounded, session-bound construction proof. Call only on the game thread.
//! Placement has no native compare-and-swap: requires a quiescent isolated site.
//! Multi-cell and accepted-receipt undo require exact expected voxels; this does not guarantee
//! safe concurrent editing. Keep actors quiescent through placement and undo.
use enshwick::{
    Load, Off,
    voxels::{VoxelBox, VoxelMaterial, VoxelPos},
    world::{Capture, TransactionId, TransactionStatus},
};
use policy::{
    construction::{self, PlaceSpec, PlanSpec, UndoSpec},
    mcp::Tool,
    write_gate,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub(crate) struct Construction {
    shared: Arc<Shared>,
}
#[derive(Debug)]
struct Shared {
    dir: PathBuf,
    boot: String,
    epoch: AtomicU64,
    active: AtomicBool,
    events: AtomicU64,
    operations: Mutex<BTreeMap<String, Operation>>,
}
#[derive(Debug)]
struct Operation {
    request: Value,
    epoch: u64,
    bricks: Vec<Brick>,
    targets: Vec<[u32; 3]>,
    region_min: [f64; 3],
    region_max: [f64; 3],
    actor: u64,
    id: Option<TransactionId>,
    undo_requested: bool,
    error: Option<String>,
    accepted: bool,
    settled: bool,
}

#[derive(Debug)]
struct Brick {
    baseline: Capture,
    expected: VoxelBox,
    targets: Vec<[u32; 3]>,
}
fn expected_box(
    baseline: &Capture,
    targets: &[[u32; 3]],
    source: enshwick::voxels::Voxel,
) -> Result<VoxelBox, String> {
    let corner = baseline.origin().to_array();
    let mut expected = baseline.voxels().clone();
    for target in targets {
        let local = std::array::from_fn(|i| target[i] - corner[i]);
        if !expected.set(local.into(), source) {
            return Err("target outside baseline capture".into());
        }
    }
    Ok(expected)
}
// Compare every captured brick, not only bricks containing a sampled target.
fn compare(bricks: &[Brick], now: &[Capture]) -> (bool, bool, bool, bool) {
    let count_equal = bricks.len() == now.len();
    let baseline = count_equal && bricks.iter().zip(now).all(|(b, n)| n == &b.baseline);
    let props = count_equal
        && bricks.iter().zip(now).all(|(b, n)| {
            n.origin() == b.baseline.origin()
                && n.size() == b.baseline.size()
                && n.props() == b.baseline.props()
        });
    let expected = props
        && bricks
            .iter()
            .zip(now)
            .all(|(b, n)| n.voxels() == &b.expected);
    let cells = count_equal
        && bricks.iter().zip(now).all(|(b, n)| {
            b.targets.iter().all(|target| {
                let corner = b.baseline.origin().to_array();
                let local = std::array::from_fn(|i| target[i] - corner[i]);
                n.voxels().get(local.into()) == b.expected.get(local.into())
            })
        });
    (expected, baseline, props, cells)
}
fn capture_bricks(game: crate::tools::Game, bricks: &[Brick]) -> Result<Vec<Capture>, String> {
    bricks
        .iter()
        .map(|b| {
            let corner = b.baseline.origin().to_array();
            game.world
                .capture(corner.into(), corner.map(|n| n + 7).into())
                .map_err(|e| format!("verification unavailable: {e}"))
        })
        .collect()
}
pub(crate) fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "building_place_cell",
            description: "WRITE: Isolated quiescent-site proof: copy one exact building voxel into air, with durable replay refusal and native undo. Dry by default. No placement CAS; occupancy may change after preflight.",
            schema: construction::place_schema(),
        },
        Tool {
            name: "building_apply_plan",
            description: "WRITE: Copy one exact source voxel into a floor/wall/doorway or union of up to 8 named primitive parts, within 16 m per axis. At most 256 unique cells across 8 bricks, 64 cells per brick, one native transaction. Dry by default. No collision/cost/feasibility proof or placement CAS; requires quiescent isolated site.",
            schema: construction::plan_schema(),
        },
        Tool {
            name: "building_operation",
            description: "Poll a session-bound construction transaction and verify its whole brick and props. Never reconstruct native IDs after restart.",
            schema: construction::status_schema(),
        },
        Tool {
            name: "building_accept",
            description: "WRITE: Accept an exactly verified applied operation, allowing another bounded placement while retaining session-only native undo. Dry by default. Does not prove save/reload persistence.",
            schema: construction::accept_schema(),
        },
        Tool {
            name: "building_undo_cell",
            description: "WRITE: Undo one retained native transaction after actor, rights and whole-brick verification. Dry by default.",
            schema: construction::undo_schema(),
        },
    ]
}
fn loaded(active: &AtomicBool, epoch: &AtomicU64) {
    epoch.fetch_add(1, Ordering::SeqCst);
    active.store(true, Ordering::SeqCst);
}
fn shutdown(active: &AtomicBool, epoch: &AtomicU64) {
    active.store(false, Ordering::SeqCst);
    epoch.fetch_add(1, Ordering::SeqCst);
}
fn require_active(active: &AtomicBool) -> Result<(), String> {
    construction::check_active(active.load(Ordering::SeqCst))
}
struct Placement {
    spec: PlaceSpec,
    targets: Vec<[u32; 3]>,
    elements: Vec<(String, [u32; 3])>,
    geometry: Option<Value>,
    composition: Option<Value>,
    request: Value,
}
impl Placement {
    fn cell(spec: PlaceSpec) -> Self {
        Self {
            targets: vec![spec.target],
            elements: vec![("cell_0_0_0".into(), spec.target)],
            geometry: None,
            composition: None,
            request: canonical(&spec),
            spec,
        }
    }
    fn plan(plan: PlanSpec) -> Result<Self, String> {
        let target = *plan.targets.first().ok_or("plan has no target cells")?;
        let composition = plan.canonical_plan.get("parts").and_then(Value::as_array).map(|parts| json!({"part_count":parts.len(),"combination":"union","overlap_owner":"first lexical part ID","opening_behavior":"omitted cells are not subtraction; other parts may fill openings"}));
        let request = json!({"operation":plan.operation,"source":plan.source,"player":plan.player,"session":plan.session,"plan":plan.canonical_plan,"targets":plan.targets,"elements":plan.elements});
        Ok(Self {
            spec: PlaceSpec {
                operation: plan.operation,
                source: plan.source,
                target,
                player: plan.player,
                dry: plan.dry,
                confirm: plan.confirm,
                session: plan.session,
            },
            targets: plan.targets,
            elements: plan.elements,
            geometry: Some(plan.canonical_plan),
            composition,
            request,
        })
    }
}
fn region(targets: &[[u32; 3]]) -> Result<([f64; 3], [f64; 3]), String> {
    let first = *targets.first().ok_or("placement has no target cells")?;
    let min = std::array::from_fn(|axis| targets.iter().fold(first[axis], |a, p| a.min(p[axis])));
    let max: [u32; 3] =
        std::array::from_fn(|axis| targets.iter().fold(first[axis], |a, p| a.max(p[axis])) + 1);
    Ok((
        min.map(|n| f64::from(n) * 0.5),
        max.map(|n| f64::from(n) * 0.5),
    ))
}
fn canonical(s: &PlaceSpec) -> Value {
    json!({"operation":s.operation,"source":s.source,"target":s.target,"player":s.player,"session":s.session})
}
fn record(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    construction::check_record_size(bytes.len())?;
    Ok(bytes)
}
fn append(dir: &Path, value: &Value) -> Result<(), String> {
    let bytes = record(value)?;
    let path = dir.join("journal.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("journal open failed: {e}"))?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    construction::check_journal_size(size, bytes.len())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("journal durability failed: {e}"))
}
fn proof_headroom(dir: &Path, brick_count: usize) -> Result<(), String> {
    let size = match fs::metadata(dir.join("journal.jsonl")) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(_) => return Err("construction journal must be a regular file".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(format!("journal preflight failed: {error}")),
    };
    construction::check_journal_size(size, (2 * brick_count + 5) * construction::MAX_RECORD_BYTES)
}
fn reserve(dir: &Path, operation: &str, value: &Value) -> Result<(), String> {
    let bytes = record(value)?;
    fs::create_dir_all(dir.join("construction-operations")).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(
            dir.join("construction-operations")
                .join(format!("{operation}.json")),
        )
        .map_err(|e| {
            format!("operation reservation refused (never retry with this identity): {e}")
        })?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("operation intent durability failed: {e}"))
}
impl Construction {
    pub(crate) fn new(load: &mut Load<'_>) -> Result<Self, Off> {
        let this = Self {
            shared: Arc::new(Shared {
                dir: load.mod_dir()?,
                boot: format!(
                    "{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| Off::new(e.to_string()))?
                        .as_nanos()
                ),
                epoch: AtomicU64::new(0),
                active: AtomicBool::new(false),
                events: AtomicU64::new(0),
                operations: Mutex::new(BTreeMap::new()),
            }),
        };
        let state = this.shared.clone();
        load.on::<enshwick::events::WorldLoaded>(move |()| {
            loaded(&state.active, &state.epoch);
        })?;
        let state = this.shared.clone();
        load.on::<enshwick::events::ShutdownStarted>(move |()| {
            shutdown(&state.active, &state.epoch);
        })?;
        let state = this.shared.clone();
        load.on_voxels_changed(move |_| {
            state.events.fetch_add(1, Ordering::SeqCst);
        })?;
        Ok(this)
    }
    pub(crate) fn session(&self) -> String {
        format!(
            "{}-{}",
            self.shared.boot,
            self.shared.epoch.load(Ordering::SeqCst)
        )
    }
    fn gate(&self, dry: bool, confirm: Option<&str>) -> Result<(), String> {
        let age = write_gate::flag_age(&self.shared.dir.join(write_gate::FLAG));
        write_gate::check(dry, true, age, confirm)
    }
    pub(crate) fn place(&self, game: crate::tools::Game, spec: PlaceSpec) -> Result<Value, String> {
        self.place_request(game, Placement::cell(spec))
    }
    pub(crate) fn apply_plan(
        &self,
        game: crate::tools::Game,
        spec: PlanSpec,
    ) -> Result<Value, String> {
        self.place_request(game, Placement::plan(spec)?)
    }
    fn place_request(
        &self,
        game: crate::tools::Game,
        placement: Placement,
    ) -> Result<Value, String> {
        let Placement {
            spec,
            targets,
            elements,
            geometry,
            composition,
            request,
        } = placement;
        self.gate(spec.dry, spec.confirm.as_deref())?;
        require_active(&self.shared.active)?;
        let session = self.session();
        if !spec.dry {
            construction::check_session(spec.session.as_deref().unwrap_or(""), &session)?;
        }
        if !enshwick::host().is_some_and(|host| host.hosting()) {
            return Err("construction requires a hosted world".into());
        }
        let epoch = self.shared.epoch.load(Ordering::SeqCst);
        {
            let ops = self
                .shared
                .operations
                .lock()
                .map_err(|_| "construction state poisoned")?;
            if let Some(op) = ops.get(&spec.operation) {
                construction::same_request(&op.request, &request)?;
                drop(ops);
                return self.status(game, spec.operation);
            }
            construction::check_operation_capacity(ops.len(), ops.values().any(|o| !o.settled))?;
        }
        if self
            .shared
            .dir
            .join("construction-operations")
            .join(format!("{}.json", spec.operation))
            .exists()
        {
            return Err(
                "stale/unknown durable operation identity; native ID is not reconstructed".into(),
            );
        }
        let players = game.world.players();
        let actors: Vec<_> = players
            .iter()
            .filter(|p| p.is_named(&spec.player) || p.key.0.to_string() == spec.player)
            .collect();
        let source = game
            .voxels
            .sample(spec.source.into())
            .map_err(|e| e.to_string())?;
        let (min, max) = region(&targets)?;
        let cell = |v: enshwick::voxels::Voxel| construction::Cell {
            kind: match v.material() {
                VoxelMaterial::Building(_) => construction::CellKind::Building,
                VoxelMaterial::Air => construction::CellKind::Air,
                _ => construction::CellKind::Other,
            },
            fill: v.fill(),
        };
        construction::check_actor_counts(players.len(), actors.len())?;
        let actor = actors[0];
        game.world
            .can_build(actor, min, max)
            .map_err(|e| e.to_string())?;
        let groups = construction::target_bricks(&targets)?;
        let persists = game.world.persists(min, max).map_err(|e| e.to_string())?;
        let mut targets_before = Vec::with_capacity(targets.len());
        let mut bricks = Vec::with_capacity(groups.len());
        for group in groups {
            let corner = group.min;
            let baseline = game
                .world
                .capture(corner.into(), corner.map(|n| n + 7).into())
                .map_err(|e| e.to_string())?;
            construction::check_capture_props(baseline.props().len())?;
            for target in &group.targets {
                let local = std::array::from_fn(|i| target[i] - corner[i]);
                let before = baseline
                    .voxels()
                    .get(VoxelPos::from(local))
                    .ok_or("target outside baseline capture")?;
                construction::check_context(construction::PlacementFacts {
                    source: cell(source),
                    target: cell(before),
                    persists,
                    connected_players: players.len(),
                    matching_actors: actors.len(),
                })?;
                targets_before.push(json!({"cells":target,"material":format!("{:?}",before.material()),"fill":before.fill()}));
            }
            let expected = expected_box(&baseline, &group.targets, source)?;
            bricks.push(Brick {
                baseline,
                expected,
                targets: group.targets,
            });
        }
        // Admit every baseline record before creating the durable replay tombstone.
        for (index, brick) in bricks.iter().enumerate() {
            record(
                &json!({"kind":"construction_brick_baseline","operation":spec.operation,"index":index,"baseline":brick.baseline}),
            )?;
        }
        proof_headroom(&self.shared.dir, bricks.len())?;
        let brick_summary:Vec<_>=bricks.iter().map(|b|json!({"min_cells":b.baseline.origin().to_array(),"size_cells":b.baseline.size(),"props_count":b.baseline.props().len(),"planned_cells":b.targets.len()})).collect();
        if spec.dry {
            return Ok(
                json!({"state":"dry","session":session,"world_mutated":false,"event_counter":self.shared.events.load(Ordering::SeqCst),"whole_brick_captured":true,"journal_cycle_headroom_checked":true,"source_cells":spec.source,"target_cells":spec.target,"source":{"material":format!("{:?}",source.material()),"fill":source.fill()},"target_before":targets_before.iter().find(|cell| cell["cells"] == json!(spec.target)),"targets_before":targets_before,"targets_cells":targets,"geometry":geometry,"composition":composition,"elements":elements,"planned_cells":targets.len(),"limits":{"max_plan_cells":construction::PLAN_MAX_CELLS,"max_capture_props_per_brick":construction::MAX_CAPTURE_PROPS,"max_bricks":construction::PLAN_MAX_BRICKS,"max_cells_per_brick":construction::PLAN_MAX_CELLS_PER_BRICK},"keyed_cells":elements.iter().map(|(key,cells)|json!({"key":key,"cells":cells})).collect::<Vec<_>>(),"region_min_m":min,"region_max_exclusive_m":max,"brick_min_cells":bricks[0].baseline.origin().to_array(),"brick_size_cells":bricks[0].baseline.size(),"props_count":bricks.iter().map(|b|b.baseline.props().len()).sum::<usize>(),"brick_count":bricks.len(),"bricks":brick_summary,"build_rights_checked":true,"target_persists_checked":true,"requires_quiescent_isolated_site":true,"placement_cas":false,"zero_events_is_not_absolute_proof":true}),
            );
        }
        let intent = json!({"kind":"construction_intent","operation":spec.operation,"request":request,"session":self.shared.boot,"epoch":epoch,"actor_account_key":actor.key.0,"brick_count":bricks.len()});
        reserve(&self.shared.dir, &spec.operation, &intent)?;
        // From reservation onward even a precommit error conservatively blocks this proof.
        let mut ops = self
            .shared
            .operations
            .lock()
            .map_err(|_| "construction state poisoned")?;
        ops.insert(
            spec.operation.clone(),
            Operation {
                request,
                epoch,
                bricks,
                targets: targets.clone(),
                region_min: min,
                region_max: max,
                actor: actor.key.0,
                id: None,
                undo_requested: false,
                error: None,
                accepted: false,
                settled: false,
            },
        );
        let op = ops
            .get_mut(&spec.operation)
            .ok_or("operation disappeared")?;
        let preparation = (|| {
            append(&self.shared.dir, &intent)?;
            for (index, brick) in op.bricks.iter().enumerate() {
                append(
                    &self.shared.dir,
                    &json!({"kind":"construction_brick_baseline","operation":spec.operation,"index":index,"baseline":brick.baseline}),
                )?;
            }
            self.gate(false, spec.confirm.as_deref())
        })();
        if let Err(e) = preparation {
            op.error = Some(e.clone());
            return Err(e);
        }
        if construction::check_epoch(epoch, self.shared.epoch.load(Ordering::SeqCst)).is_err() {
            op.error = Some("world session changed before commit".into());
            return Err("world session changed before commit".into());
        }
        require_active(&self.shared.active)?;
        let mut transaction = game.world.transaction_for(actor);
        for target in &targets {
            let block =
                VoxelBox::new((*target).into(), [1; 3], vec![source]).ok_or("invalid cell box")?;
            transaction = transaction.write_voxels(&block);
        }
        self.gate(false, spec.confirm.as_deref())?;
        require_active(&self.shared.active)?;
        construction::check_epoch(epoch, self.shared.epoch.load(Ordering::SeqCst))?;
        match transaction.commit() {
            Ok(id) => op.id = Some(id),
            Err(e) => {
                op.error = Some(e.to_string());
                return Err(format!("commit refused; operation reserved: {e}"));
            }
        }
        let id = op.id.ok_or("missing committed transaction")?;
        if let Err(e) = append(
            &self.shared.dir,
            &json!({"kind":"construction_committed","operation":spec.operation,"transaction":id.get()}),
        ) {
            op.error = Some(e.clone());
            return Err(format!(
                "native transaction retained; journal error, outcome unknown; never retry placement: {e}"
            ));
        }
        drop(ops);
        self.status(game, spec.operation)
    }
    pub(crate) fn status(
        &self,
        game: crate::tools::Game,
        operation: String,
    ) -> Result<Value, String> {
        if require_active(&self.shared.active).is_err() {
            return Ok(
                json!({"operation":operation,"state":"inactive_session_unknown","undo_available":false}),
            );
        }
        let mut ops = self
            .shared
            .operations
            .lock()
            .map_err(|_| "construction state poisoned")?;
        let Some(op) = ops.get_mut(&operation) else {
            let path = self
                .shared
                .dir
                .join("construction-operations")
                .join(format!("{operation}.json"));
            return if path.exists() {
                Ok(
                    json!({"operation":operation,"state":"stale_unknown","native_id_reconstructed":false}),
                )
            } else {
                Err("unknown construction operation".into())
            };
        };
        if construction::check_epoch(op.epoch, self.shared.epoch.load(Ordering::SeqCst)).is_err() {
            return Ok(
                json!({"operation":operation,"state":"stale_session_unknown","undo_available":false}),
            );
        }
        let native = op.id.and_then(|id| game.world.transaction_status(id));
        let mut expected_equal = None;
        let mut baseline_equal = None;
        let mut cell_equal = None;
        if matches!(
            native,
            Some(
                TransactionStatus::Applied
                    | TransactionStatus::Undone
                    | TransactionStatus::Failed(_)
            )
        ) {
            let now = capture_bricks(game, &op.bricks)?;
            let (expected, baseline, _, cells) = compare(&op.bricks, &now);
            baseline_equal = Some(baseline);
            expected_equal = Some(expected);
            cell_equal = Some(cells);
            if matches!(native, Some(TransactionStatus::Undone)) && baseline_equal == Some(true) {
                op.settled = true;
            }
        }
        let state = match native {
            Some(TransactionStatus::Applied) if expected_equal == Some(true) => {
                "applied_capture_verified"
            }
            Some(TransactionStatus::Undone) if baseline_equal == Some(true) => {
                "undone_baseline_verified"
            }
            Some(TransactionStatus::Applied | TransactionStatus::Undone) => {
                "verification_conflict_unknown"
            }
            Some(TransactionStatus::Pending) => "pending",
            Some(TransactionStatus::Undoing) => "undoing",
            Some(TransactionStatus::Failed(_)) => "failed_unknown",
            Some(TransactionStatus::Expired) => "expired_unknown",
            Some(TransactionStatus::UndoRefused(_)) => "undo_refused_unknown",
            Some(TransactionStatus::NotAllowed(_)) => "not_allowed_unknown",
            _ => "unknown",
        };
        Ok(
            json!({"operation":operation,"state":state,"session":self.session(),"native_status":format!("{native:?}"),"transaction":op.id.map(TransactionId::get),"whole_capture_expected_equal":expected_equal,"whole_capture_baseline_equal":baseline_equal,"cell_expected_equal":cell_equal,"planned_cell_count":op.targets.len(),"brick_count":op.bricks.len(),"region_min_m":op.region_min,"region_max_exclusive_m":op.region_max,"settled":op.settled,"accepted":op.accepted,"journal_error":op.error,"undo_requested":op.undo_requested,"event_counter":self.shared.events.load(Ordering::SeqCst),"placement_cas":false,"requires_quiescent_isolated_site":true,"save_reload_verified":false}),
        )
    }
    pub(crate) fn accept(&self, game: crate::tools::Game, spec: UndoSpec) -> Result<Value, String> {
        self.gate(spec.dry, spec.confirm.as_deref())?;
        require_active(&self.shared.active)?;
        let mut ops = self
            .shared
            .operations
            .lock()
            .map_err(|_| "construction state poisoned")?;
        let op = ops
            .get_mut(&spec.operation)
            .ok_or("stale operation cannot be accepted")?;
        construction::check_epoch(op.epoch, self.shared.epoch.load(Ordering::SeqCst))?;
        let id = op
            .id
            .ok_or("no retained native transaction; outcome unknown")?;
        if op.accepted {
            drop(ops);
            return self.status(game, spec.operation);
        }
        let state = game.world.transaction_status(id);
        let now = capture_bricks(game, &op.bricks)?;
        let (expected, _, _, _) = compare(&op.bricks, &now);
        let persists = game
            .world
            .persists(op.region_min, op.region_max)
            .map_err(|e| e.to_string())?;
        construction::check_accept_context(construction::AcceptFacts {
            applied: matches!(state, Some(TransactionStatus::Applied)),
            expected_equal: expected,
            journal_ok: op.error.is_none(),
            undo_requested: op.undo_requested,
            persists,
        })?;
        let players = game.world.players();
        let actors: Vec<_> = players.iter().filter(|p| p.key.0 == op.actor).collect();
        construction::check_actor_counts(players.len(), actors.len())?;
        game.world
            .can_build(actors[0], op.region_min, op.region_max)
            .map_err(|e| e.to_string())?;
        if spec.dry {
            return Ok(
                json!({"state":"dry_accept","operation":spec.operation,"world_mutated":false,"whole_capture_expected_equal":expected,"accepted":false,"save_reload_verified":false}),
            );
        }
        self.gate(false, spec.confirm.as_deref())?;
        require_active(&self.shared.active)?;
        construction::check_epoch(op.epoch, self.shared.epoch.load(Ordering::SeqCst))?;
        // Release placement exclusivity only after the acceptance record is durable.
        if let Err(error) = append(
            &self.shared.dir,
            &json!({"kind":"construction_accepted","operation":spec.operation,"transaction":id.get(),"whole_capture_expected_equal":expected,"brick_count":now.len(),"save_reload_verified":false}),
        ) {
            op.error = Some(error.clone());
            return Err(error);
        }
        op.accepted = true;
        op.settled = true;
        drop(ops);
        self.status(game, spec.operation)
    }
    pub(crate) fn undo(&self, game: crate::tools::Game, spec: UndoSpec) -> Result<Value, String> {
        self.gate(spec.dry, spec.confirm.as_deref())?;
        require_active(&self.shared.active)?;
        let _ = self.status(game, spec.operation.clone())?;
        let mut ops = self
            .shared
            .operations
            .lock()
            .map_err(|_| "construction state poisoned")?;
        let other_unsettled = ops
            .iter()
            .any(|(name, op)| name != &spec.operation && !op.settled);
        let op = ops
            .get_mut(&spec.operation)
            .ok_or("stale operation cannot be undone")?;
        construction::check_epoch(op.epoch, self.shared.epoch.load(Ordering::SeqCst))?;
        let id = op
            .id
            .ok_or("no retained native transaction; outcome unknown")?;
        if op.undo_requested {
            // Never retry a requested undo, including native refusal, in this proof.
            drop(ops);
            return self.status(game, spec.operation);
        }
        let state = game.world.transaction_status(id);
        if matches!(
            state,
            Some(TransactionStatus::Undoing | TransactionStatus::Undone)
        ) {
            drop(ops);
            return self.status(game, spec.operation);
        }
        construction::check_undo_interlock(other_unsettled)?;
        if !matches!(
            state,
            Some(TransactionStatus::Applied | TransactionStatus::Failed(_))
        ) {
            return Err(format!("undo blocked by native status: {state:?}"));
        }
        let players = game.world.players();
        let actors: Vec<_> = players.iter().filter(|p| p.key.0 == op.actor).collect();
        construction::check_actor_counts(players.len(), actors.len())?;
        let min = op.region_min;
        let max = op.region_max;
        game.world
            .can_build(actors[0], min, max)
            .map_err(|e| e.to_string())?;
        let now = capture_bricks(game, &op.bricks)?;
        let (expected, baseline_equal, props_unchanged, _) = compare(&op.bricks, &now);
        let persists = game.world.persists(min, max).map_err(|e| e.to_string())?;
        construction::check_undo_context(construction::UndoFacts {
            accepted: op.accepted,
            props_unchanged,
            persists,
            planned_cells: op.targets.len(),
            expected_voxels_equal: expected,
        })?;
        if spec.dry {
            return Ok(
                json!({"state":"dry_undo","world_mutated":false,"application_expected_equal":expected,"whole_capture_baseline_equal":baseline_equal,"props_unchanged":props_unchanged,"native_conflict_check_required":true}),
            );
        }
        append(
            &self.shared.dir,
            &json!({"kind":"construction_undo_intent","operation":spec.operation,"transaction":id.get(),"application_expected_equal":expected,"whole_capture_baseline_equal":baseline_equal,"props_unchanged":props_unchanged,"brick_count":now.len(),"native_conflict_check_required":true}),
        )?;
        for (index, capture) in now.iter().enumerate() {
            append(
                &self.shared.dir,
                &json!({"kind":"construction_undo_brick","operation":spec.operation,"index":index,"actual_before_undo":capture}),
            )?;
        }
        self.gate(false, spec.confirm.as_deref())?;
        construction::check_epoch(op.epoch, self.shared.epoch.load(Ordering::SeqCst))?;
        require_active(&self.shared.active)?;
        // Accepted receipts become exclusive again before the native undo call.
        // Even a queue refusal is fail-stop; replay must never reissue this undo.
        op.settled = false;
        op.undo_requested = true;
        if let Err(error) = game.world.undo(id) {
            let error = format!("undo request failed; outcome unknown: {error}");
            op.error = Some(error.clone());
            return Err(error);
        }
        if let Err(e) = append(
            &self.shared.dir,
            &json!({"kind":"construction_undo_queued","operation":spec.operation,"transaction":id.get()}),
        ) {
            op.error = Some(e.clone());
            return Err(format!("undo queued; journal error, outcome unknown: {e}"));
        }
        drop(ops);
        let mut result = self.status(game, spec.operation)?;
        result["application_expected_equal"] = json!(expected);
        result["native_conflict_check_required"] = json!(true);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dir() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "mcp-construction-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn cell_conversion_keeps_existing_identity_and_single_cell_region() {
        let args = json!({"operation":"cell","source_x":0,"source_y":0,"source_z":0,"x":1,"y":2,"z":3,"player":"Researcher"});
        let spec = construction::check_place(args.as_object().unwrap()).unwrap();
        let original = canonical(&spec);
        let placement = Placement::cell(spec);
        assert_eq!(placement.request, original);
        assert_eq!(placement.targets, vec![[2, 4, 6]]);
        assert_eq!(
            region(&placement.targets).unwrap(),
            ([1.0, 2.0, 3.0], [1.5, 2.5, 3.5])
        );
    }
    #[test]
    fn plan_conversion_retains_all_keyed_cells_geometry_and_preview_session() {
        let args = json!({"operation":"plan","source_x":0,"source_y":0,"source_z":0,"player":"Researcher","session":"preview-1", "plan":{"kind":"floor","x":4,"y":4,"z":4,"width":4,"height":0.5,"depth":4}});
        let spec = construction::check_plan(args.as_object().unwrap()).unwrap();
        let targets = spec.targets.clone();
        let elements = spec.elements.clone();
        let geometry = spec.canonical_plan.clone();
        let placement = Placement::plan(spec).unwrap();
        assert_eq!(placement.targets, targets);
        assert_eq!(placement.targets.len(), 64);
        assert_eq!(placement.elements, elements);
        assert_eq!(placement.geometry, Some(geometry.clone()));
        assert_eq!(placement.composition, None);
        assert_eq!(placement.request["plan"], geometry);
        assert_eq!(placement.request["session"], "preview-1");
        assert_eq!(
            region(&placement.targets).unwrap(),
            ([4.0, 4.0, 4.0], [8.0, 4.5, 8.0])
        );
        assert_eq!(
            placement
                .elements
                .iter()
                .map(|(_, cell)| *cell)
                .collect::<Vec<_>>(),
            placement.targets
        );
    }
    #[test]
    fn composite_conversion_keeps_one_write_per_cell_and_replay_identity() {
        let args = json!({"operation":"union","source_x":0,"source_y":0,"source_z":0,"player":"Researcher","session":"preview-1","plan":{"kind":"composite","parts":[
            {"id":"z","kind":"floor","x":4.5,"y":4,"z":4,"width":1,"height":0.5,"depth":0.5},
            {"id":"a","kind":"floor","x":4,"y":4,"z":4,"width":1,"height":0.5,"depth":0.5}
        ]}});
        let plan = construction::check_plan(args.as_object().unwrap()).unwrap();
        let canonical = plan.canonical_plan.clone();
        let placement = Placement::plan(plan).unwrap();
        assert_eq!(placement.targets, vec![[8, 8, 8], [9, 8, 8], [10, 8, 8]]);
        assert_eq!(placement.elements.len(), placement.targets.len());
        assert_eq!(
            placement.elements[1],
            ("part_a_cell_1_0_0".into(), [9, 8, 8])
        );
        assert_eq!(placement.geometry, Some(canonical.clone()));
        assert_eq!(placement.request["plan"], canonical);
        assert_eq!(
            placement.composition,
            Some(
                json!({"part_count":2,"combination":"union","overlap_owner":"first lexical part ID","opening_behavior":"omitted cells are not subtraction; other parts may fill openings"})
            )
        );
        assert!(placement.request.get("composition").is_none());
        let mut reordered = args;
        reordered["plan"]["parts"].as_array_mut().unwrap().reverse();
        let replay =
            Placement::plan(construction::check_plan(reordered.as_object().unwrap()).unwrap())
                .unwrap();
        assert_eq!(placement.targets, replay.targets);
        assert_eq!(placement.elements, replay.elements);
        assert_eq!(placement.composition, replay.composition);
        construction::same_request(&placement.request, &replay.request).unwrap();
        reordered["plan"]["parts"][0]["id"] = json!("changed");
        let changed =
            Placement::plan(construction::check_plan(reordered.as_object().unwrap()).unwrap())
                .unwrap();
        assert!(construction::same_request(&placement.request, &changed.request).is_err());
    }
    fn synthetic_capture(corner: [u32; 3], voxels: VoxelBox) -> Capture {
        serde_json::from_value(json!({"origin":VoxelPos::from(corner),"voxels":voxels,"props":[]}))
            .unwrap()
    }
    #[test]
    fn multi_brick_expected_images_preserve_unrequested_cells_and_compare_every_brick() {
        use enshwick::voxels::Voxel;
        let air = Voxel::new(VoxelMaterial::Air, 0);
        let source = Voxel::new(VoxelMaterial::Building(7), 7);
        let targets = vec![[7, 0, 0], [8, 0, 0]];
        let bricks: Vec<_> = construction::target_bricks(&targets)
            .unwrap()
            .into_iter()
            .map(|group| {
                let baseline = synthetic_capture(
                    group.min,
                    VoxelBox::new([0; 3].into(), [8; 3], vec![air; 512]).unwrap(),
                );
                let expected = expected_box(&baseline, &group.targets, source).unwrap();
                assert_eq!(
                    expected.as_slice().iter().filter(|v| **v == source).count(),
                    1
                );
                Brick {
                    baseline,
                    expected,
                    targets: group.targets,
                }
            })
            .collect();
        let baseline: Vec<_> = bricks.iter().map(|b| b.baseline.clone()).collect();
        assert_eq!(compare(&bricks, &baseline), (false, true, true, false));
        let mut applied: Vec<_> = bricks
            .iter()
            .map(|b| synthetic_capture(b.baseline.origin().to_array(), b.expected.clone()))
            .collect();
        assert_eq!(compare(&bricks, &applied), (true, false, true, true));
        let mut drifted = bricks[1].expected.clone();
        assert!(drifted.set([7, 7, 7].into(), source));
        applied[1] = synthetic_capture([8, 0, 0], drifted);
        assert_eq!(compare(&bricks, &applied), (false, false, true, true));
        assert_eq!(
            compare(&bricks, &applied[..1]),
            (false, false, false, false)
        );
    }
    #[test]
    fn same_brick_layer_undo_requires_reverse_order_even_after_acceptance() {
        use enshwick::voxels::Voxel;
        let air = Voxel::new(VoxelMaterial::Air, 0);
        let stone = Voxel::new(VoxelMaterial::Building(157), 7);
        let baseline = synthetic_capture(
            [0; 3],
            VoxelBox::new([0; 3].into(), [8; 3], vec![air; 512]).unwrap(),
        );
        let frame = Brick {
            expected: expected_box(&baseline, &[[0, 0, 0]], stone).unwrap(),
            baseline,
            targets: vec![[0, 0, 0]],
        };
        let frame_image = synthetic_capture([0; 3], frame.expected.clone());
        let fill = Brick {
            expected: expected_box(&frame_image, &[[1, 0, 0]], stone).unwrap(),
            baseline: frame_image.clone(),
            targets: vec![[1, 0, 0]],
        };
        let filled = synthetic_capture([0; 3], fill.expected.clone());
        assert!(!compare(std::slice::from_ref(&frame), std::slice::from_ref(&filled)).0);
        assert!(compare(std::slice::from_ref(&fill), std::slice::from_ref(&filled)).0);
        // Undoing the fill restores exactly the accepted frame's expected image.
        assert!(
            compare(
                std::slice::from_ref(&fill),
                std::slice::from_ref(&frame_image)
            )
            .1
        );
        assert!(
            compare(
                std::slice::from_ref(&frame),
                std::slice::from_ref(&frame_image)
            )
            .0
        );
    }
    #[test]
    fn eight_brick_ledger_headroom_reserves_all_baseline_and_undo_records() {
        let d = dir();
        let file = fs::File::create(d.join("journal.jsonl")).unwrap();
        let boundary =
            (construction::JOURNAL_MAX_BYTES - 21 * construction::MAX_RECORD_BYTES) as u64;
        file.set_len(boundary).unwrap();
        assert!(proof_headroom(&d, 8).is_ok());
        file.set_len(boundary + 1).unwrap();
        assert!(proof_headroom(&d, 8).is_err());
        drop(file);
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn shutdown_blocks_new_tokens_until_a_new_world_loads() {
        let active = std::sync::atomic::AtomicBool::new(false);
        let epoch = AtomicU64::new(0);
        assert!(require_active(&active).is_err());
        loaded(&active, &epoch);
        let preview = epoch.load(Ordering::SeqCst);
        assert!(require_active(&active).is_ok());
        shutdown(&active, &epoch);
        let shutdown_token = epoch.load(Ordering::SeqCst);
        assert_ne!(preview, shutdown_token);
        assert!(
            require_active(&active).is_err(),
            "new shutdown token cannot authorize dry preview or write"
        );
        loaded(&active, &epoch);
        assert!(require_active(&active).is_ok());
        assert_ne!(shutdown_token, epoch.load(Ordering::SeqCst));
    }
    #[test]
    fn complete_cycle_headroom_is_required_before_reservation() {
        let d = dir();
        assert!(proof_headroom(&d, 1).is_ok());
        let file = fs::File::create(d.join("journal.jsonl")).unwrap();
        let boundary =
            (construction::JOURNAL_MAX_BYTES - 7 * construction::MAX_RECORD_BYTES) as u64;
        file.set_len(boundary).unwrap();
        assert!(proof_headroom(&d, 1).is_ok());
        file.set_len(boundary + 1).unwrap();
        assert!(proof_headroom(&d, 1).is_err());
        assert!(!d.join("construction-operations").exists());
        drop(file);
        fs::remove_file(d.join("journal.jsonl")).unwrap();
        fs::create_dir(d.join("journal.jsonl")).unwrap();
        assert!(proof_headroom(&d, 1).is_err());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn reservation_never_overwrites_prior_intent() {
        let d = dir();
        reserve(&d, "proof", &json!({"a":1})).unwrap();
        assert!(reserve(&d, "proof", &json!({"a":2})).is_err());
        assert!(
            fs::read_to_string(d.join("construction-operations/proof.json"))
                .unwrap()
                .contains("1")
        );
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn record_and_journal_caps_fail_before_append() {
        let d = dir();
        assert!(
            reserve(
                &d,
                "proof",
                &json!({"x":"a".repeat(construction::MAX_RECORD_BYTES)})
            )
            .is_err()
        );
        assert!(!d.join("construction-operations/proof.json").exists());
        let f = fs::File::create(d.join("journal.jsonl")).unwrap();
        f.set_len(construction::JOURNAL_MAX_BYTES as u64).unwrap();
        assert!(append(&d, &json!({"a":1})).is_err());
        assert_eq!(
            f.metadata().unwrap().len(),
            construction::JOURNAL_MAX_BYTES as u64
        );
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn failed_precommit_journal_preserves_tombstone() {
        let d = dir();
        reserve(&d, "proof", &json!({"a":1})).unwrap();
        fs::create_dir(d.join("journal.jsonl")).unwrap();
        assert!(append(&d, &json!({"a":1})).is_err());
        assert!(reserve(&d, "proof", &json!({"a":1})).is_err());
        fs::remove_dir_all(d).unwrap();
    }
}
