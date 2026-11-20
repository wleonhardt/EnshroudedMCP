//! Positive synthetic SDK voxel data survives lifecycle events independently.
use crate::tools::GameTools;
use enshwick::{
    Load, Mod, Off,
    abi::Status,
    schedule::Scheduler,
    voxels::{Voxel, VoxelMaterial, VoxelPos, Voxels},
};
use policy::mcp::Tools;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone)]
struct Context {
    tools: Arc<GameTools>,
    voxels: Voxels,
    scheduler: Scheduler,
}
static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
struct Reader;
impl Mod for Reader {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *CONTEXT.lock().unwrap() = Some(Context {
            tools: Arc::new(crate::Mcp::init(load)?.tools),
            voxels: load.voxels(),
            scheduler: load.scheduler()?,
        });
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Reader, "id = \"test.voxel-lifecycle\"\nside = \"server\"");
fn start() -> (enshwick::test::Installed, Context) {
    let loader = enshwick::test::FakeHost::new()
        .world()
        .voxels(
            VoxelPos::new(8, 8, 8),
            [1, 1, 1],
            &[Voxel::new(VoxelMaterial::Terrain(9), 255)],
        )
        .install();
    // SAFETY: generated exports belong to this isolated fake-only reader.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
    let c = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
    positive(&c);
    (loader, c)
}
fn positive(c: &Context) {
    assert!(enshwick::host().unwrap().hosting());
    let block = c.voxels.read(VoxelPos::new(8, 8, 8), [1, 1, 1]).unwrap();
    let (_, voxel) = block.iter().next().unwrap();
    assert!(matches!(voxel.material(), VoxelMaterial::Terrain(9)));
    assert_eq!(voxel.fill(), 255);
}
fn args(tool: &str) -> Value {
    if tool == "voxel_sample" {
        return json!({"x":4,"y":4,"z":4});
    }
    let mut value = json!({"x":4,"y":4,"z":4,"width":0.5,"height":0.5,"depth":0.5});
    if tool == "terrain_image" {
        value["pins"] = json!([{"label":"marker","position":[4,4,4]}]);
    }
    value
}
fn finish(
    loader: &enshwick::test::Installed,
    worker: thread::JoinHandle<Result<Value, String>>,
) -> Result<Value, String> {
    let until = Instant::now() + Duration::from_secs(2);
    while !worker.is_finished() && Instant::now() < until {
        let _ = loader.tick(Duration::from_millis(1));
        thread::yield_now();
    }
    assert!(worker.is_finished(), "voxel request did not complete");
    worker.join().unwrap()
}
fn queued(tool: &'static str) {
    for phase in ["unchanged", "reload", "shutdown"] {
        let (loader, c) = start();
        let _ = loader.tick(Duration::from_millis(1));
        let scheduler = c.scheduler;
        assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
        let tools = c.tools.clone();
        let request = args(tool);
        let worker = thread::spawn(move || tools.call(tool, request.as_object().unwrap()));
        // Pinned SDK Debug is only an enqueue barrier for this isolated queue.
        let until = Instant::now() + Duration::from_secs(2);
        while format!("{scheduler:?}") != "Scheduler { pending: true }" {
            assert!(!worker.is_finished(), "read did not queue");
            assert!(Instant::now() < until, "queue barrier");
            thread::yield_now();
        }
        match phase {
            "reload" => assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0),
            "shutdown" => assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0),
            _ => {}
        }
        positive(&c);
        let reads = loader.voxel_reads().len();
        let result = finish(&loader, worker);
        if phase == "unchanged" {
            let value = result.unwrap();
            if tool == "voxel_sample" {
                assert_eq!(value["fill"], 255);
                assert_eq!(value["material"], "Terrain(9)");
            } else {
                let survey = if tool == "terrain_image" {
                    &value["survey"]
                } else {
                    &value
                };
                assert_eq!(survey["voxels"]["material_counts"]["terrain"], 1);
                assert_eq!(survey["voxels"]["cells"][0]["fill"], 255);
            }
            if tool == "terrain_image" {
                assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
                let before = loader.voxel_reads().len();
                match c.tools.present(tool, value).unwrap() {
                    policy::presentation::Presentation::Png { metadata, .. } => {
                        assert_eq!(metadata["material_counts"]["terrain"], 1);
                        assert_eq!(metadata["pins"][0]["label"], "marker");
                        for key in [
                            "rendered_game_verified",
                            "surface_height_verified",
                            "clearance_verified",
                            "world_mutated",
                        ] {
                            assert_eq!(metadata[key], false);
                        }
                    }
                    _ => panic!("copied diagnostic PNG missing"),
                }
                assert_eq!(
                    loader.voxel_reads().len(),
                    before,
                    "encoding must use captured copies"
                );
            }
        } else {
            let error =
                result.expect_err("queued changed-context voxel read returned retained terrain");
            assert!(
                error.contains(if phase == "reload" {
                    "session"
                } else {
                    "active"
                }),
                "{tool}/{phase}: {error}"
            );
            assert_eq!(
                loader.voxel_reads().len(),
                reads,
                "refused capture must not read voxels"
            );
        }
    }
}
fn admission(tool: &'static str) {
    let (loader, c) = start();
    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
    positive(&c);
    let reads = loader.voxel_reads().len();
    let request = args(tool);
    let tools = c.tools.clone();
    let worker = thread::spawn(move || tools.call(tool, request.as_object().unwrap()));
    let error = finish(&loader, worker).unwrap_err();
    assert!(error.contains("active"), "{tool}: {error}");
    assert_eq!(loader.voxel_reads().len(), reads);
}
#[test]
fn queued_survey_refuses_retained_terrain_across_lifecycle() {
    queued("building_survey");
}
#[test]
fn queued_image_capture_refuses_retained_terrain_across_lifecycle() {
    queued("terrain_image");
}
#[test]
fn queued_sample_refuses_retained_terrain_across_lifecycle() {
    queued("voxel_sample");
}
#[test]
fn survey_refuses_shutdown_before_admission() {
    admission("building_survey");
}
#[test]
fn image_capture_refuses_shutdown_before_admission() {
    admission("terrain_image");
}
#[test]
fn sample_refuses_shutdown_before_admission() {
    admission("voxel_sample");
}
