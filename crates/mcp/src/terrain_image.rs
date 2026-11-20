//! Native capture only. Rasterization and PNG encoding run later on copied data.
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "terrain_image",
        description: "Read at most 4096 half-metre voxels in an 8 m box and return diagnostic top/section PNG plus exact JSON coordinates, raw materials/fill and optional caller reference pins. Native capture runs on the game thread; encoding uses copies. May load chunks. No game-frame capture, camera, props, collision, surface-height, support or movement proof. Air is not clearance; not an executor. Native read/capture requires active MCP lifecycle at admission and execution with unchanged epoch.",
        schema: policy::terrain_image::schema(),
    }
}

/// Called only by the server game-thread scheduler.
pub(crate) fn capture(
    game: crate::tools::Game,
    spec: policy::terrain_image::Spec,
    request: Value,
) -> Result<Value, String> {
    let survey = crate::building_survey::read(game, spec.survey)?;
    let captured = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|n| u64::try_from(n.as_millis()).ok());
    Ok(json!({"request":request,"survey":survey,"captured_unix_ms":captured}))
}
