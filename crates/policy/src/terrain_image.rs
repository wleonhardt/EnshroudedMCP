//! Bounded diagnostic raster from copied voxel observations; no game access.

use crate::{
    building_survey,
    presentation::{Png, Presentation},
};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

const WIDTH: usize = 448;
const HEIGHT: usize = 448;
const LEFT: usize = 64;
const TOP: usize = 112;
const TILE: usize = 16;
pub const MAX_PINS: usize = 16;

#[derive(Debug, Clone)]
pub struct Pin {
    label: String,
    position: [f64; 3],
}
#[derive(Debug, Clone)]
pub struct Spec {
    pub survey: building_survey::Spec,
    view: &'static str,
    slice: Option<u32>,
    pins: Vec<Pin>,
}

#[must_use]
pub fn schema() -> Value {
    let mut schema = building_survey::schema();
    let props = schema["properties"].as_object_mut().unwrap();
    props.remove("player");
    props.remove("foundation");
    props.insert("view".into(),json!({"type":"string","enum":["top","xy","zy"],"description":"Top X/Z, or vertical X/Y at Z slice, or Z/Y at X slice. Diagnostic occupancy only."}));
    props.insert("slice".into(),json!({"type":"number","minimum":0,"maximum":crate::bounds::EDGE,"multipleOf":0.5,"description":"Required for sections, forbidden for top; half-open half-metre interval minimum in metres."}));
    props.insert("pins".into(),json!({"type":"array","maxItems":MAX_PINS,"items":{"type":"object","additionalProperties":false,"properties":{"label":{"type":"string","minLength":1,"maxLength":24},"position":{"type":"array","minItems":3,"maxItems":3,"items":{"type":"number"}}},"required":["label","position"]},"description":"Caller-provided reference positions, never entity selectors or movement requests. Labels use ASCII letters/digits/spaces/_/-."}));
    schema
}

/// # Errors
/// Refuses unknown fields, unbounded/off-grid boxes, invalid sections or reference pins.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    let schema = schema();
    crate::args::check(&schema, args)?;
    let mut box_args = args.clone();
    for key in ["view", "slice", "pins"] {
        box_args.remove(key);
    }
    let survey = building_survey::check(&box_args)?;
    let view = match args.get("view").and_then(Value::as_str).unwrap_or("top") {
        "top" => "top",
        "xy" => "xy",
        "zy" => "zy",
        _ => return Err("unknown image view".into()),
    };
    let slice = if view == "top" {
        if args.contains_key("slice") {
            return Err("top view does not take a slice".into());
        }
        None
    } else {
        let n = args
            .get("slice")
            .and_then(Value::as_f64)
            .ok_or("section requires slice")?;
        if !n.is_finite() || n < 0.0 || (n * 2.0).fract() != 0.0 {
            return Err("slice must be on half-metre grid".into());
        }
        let axis = if view == "xy" { 2 } else { 0 };
        let n = (n * 2.0) as u32;
        if n < survey.min[axis] || n >= survey.max_exclusive[axis] {
            return Err("slice must lie inside capture".into());
        }
        Some(n)
    };
    let mut pins = Vec::new();
    let mut labels = BTreeSet::new();
    if let Some(values) = args.get("pins") {
        let values = values.as_array().ok_or("pins must be an array")?;
        if values.len() > MAX_PINS {
            return Err("at most 16 reference pins".into());
        }
        for value in values {
            let pin = value.as_object().ok_or("pin must be an object")?;
            crate::args::check(&schema["properties"]["pins"]["items"], pin)?;
            let label = pin["label"].as_str().ok_or("invalid pin label")?;
            if label.trim().is_empty()
                || label.len() > 24
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b" _-".contains(&b))
                || !labels.insert(label.to_owned())
            {
                return Err(
                    "pin labels must be unique, 1..24 ASCII letters/digits/spaces/_/-".into(),
                );
            }
            let values = pin["position"]
                .as_array()
                .filter(|v| v.len() == 3)
                .ok_or("pin position requires three metres")?;
            let mut position = [0.0; 3];
            for (i, v) in values.iter().enumerate() {
                position[i] = v
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or("invalid pin coordinate")?;
            }
            if !crate::bounds::in_world(position) {
                return Err("pin position outside world".into());
            }
            pins.push(Pin {
                label: label.to_owned(),
                position,
            });
        }
    }
    Ok(Spec {
        survey,
        view,
        slice,
        pins,
    })
}

#[derive(Clone)]
struct Cell {
    min: [u32; 3],
    kind: &'static str,
    id: Option<u8>,
    fill: u8,
}
fn coordinate(value: &Value) -> Result<[u32; 3], String> {
    let row = value
        .as_array()
        .filter(|v| v.len() == 3)
        .ok_or("invalid cell coordinate")?;
    let mut out = [0; 3];
    for (i, v) in row.iter().enumerate() {
        out[i] = v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid cell coordinate")?;
    }
    Ok(out)
}
fn capture(spec: &Spec, value: &Value) -> Result<(BTreeMap<[u32; 3], Cell>, Value), String> {
    let s = &spec.survey;
    if value["status"] != "read_only_observation"
        || value["grid"]["cell_size_m"].as_f64() != Some(0.5)
    {
        return Err("invalid native survey grid/status".into());
    }
    let region = &value["region"];
    if coordinate(&region["min_cells"])? != s.min
        || coordinate(&region["size_cells"])? != s.size
        || coordinate(&region["max_exclusive_cells"])? != s.max_exclusive
    {
        return Err("native region differs from request".into());
    }
    for (key, expected) in [("min_m", s.min), ("max_exclusive_m", s.max_exclusive)] {
        if region[key].as_array().is_none_or(|v| {
            v.len() != 3
                || v.iter()
                    .enumerate()
                    .any(|(i, n)| n.as_f64() != Some(f64::from(expected[i]) * 0.5))
        }) {
            return Err("native region metre/cell mismatch".into());
        }
    }
    let rows = value["voxels"]["cells"]
        .as_array()
        .ok_or("complete cells required")?;
    if rows.len() != s.cell_count
        || value["voxels"]["cell_count"].as_u64() != Some(s.cell_count as u64)
    {
        return Err("complete capture required".into());
    }
    let mut cells = BTreeMap::new();
    let mut counts = BTreeMap::from([
        ("air", 0usize),
        ("terrain", 0),
        ("building", 0),
        ("unknown", 0),
    ]);
    for row in rows {
        let min = coordinate(&row["min_cells"])?;
        if (0..3).any(|i| min[i] < s.min[i] || min[i] >= s.max_exclusive[i]) {
            return Err("cell outside capture".into());
        }
        let kind = match row["material"]["kind"].as_str() {
            Some("air") => "air",
            Some("terrain") => "terrain",
            Some("building") => "building",
            Some("unknown") => "unknown",
            _ => return Err("invalid material kind".into()),
        };
        let raw = &row["material"]["id"];
        let id = if raw.is_null() && matches!(kind, "air" | "unknown") {
            None
        } else {
            Some(
                raw.as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or("invalid material id")?,
            )
        };
        if kind == "air" && id.is_some() {
            return Err("Air material id must be null".into());
        }
        let fill = row["fill"]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .ok_or("invalid raw fill")?;
        if cells
            .insert(
                min,
                Cell {
                    min,
                    kind,
                    id,
                    fill,
                },
            )
            .is_some()
        {
            return Err("duplicate captured cell".into());
        }
        *counts.get_mut(kind).unwrap() += 1;
    }
    let counts = json!(counts);
    if value["voxels"]["material_counts"] != counts {
        return Err("material counts disagree".into());
    }
    Ok((cells, counts))
}

struct Raster(Vec<u8>);
impl Raster {
    fn new() -> Self {
        Self([24, 35, 50].repeat(WIDTH * HEIGHT))
    }
    fn put(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x < WIDTH && y < HEIGHT {
            self.0[(y * WIDTH + x) * 3..(y * WIDTH + x) * 3 + 3].copy_from_slice(&color);
        }
    }
    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 3]) {
        for b in y..y + h {
            for a in x..x + w {
                self.put(a, b, color);
            }
        }
    }
    fn text(&mut self, x: usize, y: usize, text: &str, color: [u8; 3]) {
        for (i, c) in text.chars().enumerate() {
            for (row, bits) in glyph(c.to_ascii_uppercase()).iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.put(x + i * 6 + col, y + row, color);
                    }
                }
            }
        }
    }
}
fn glyph(c: char) -> [u8; 7] {
    match c {
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 27, 17],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ',' => [0, 0, 0, 0, 0, 4, 8],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        ' ' => [0; 7],
        _ => [14, 17, 1, 2, 4, 0, 4],
    }
}

/// Render a complete copied native survey. No game handles, files or native commands.
/// # Errors
/// Refuses invalid requests/captures and encoding failure, never substitutes blank geometry.
pub fn present(value: Value) -> Result<Presentation, String> {
    let args = value["request"]
        .as_object()
        .ok_or("image request copy unavailable")?;
    let spec = check(args)?;
    let (cells, counts) = capture(&spec, &value["survey"])?;
    let mut selected: BTreeMap<(u32, u32), Cell> = BTreeMap::new();
    let (horizontal, vertical) = match spec.view {
        "top" => (0, 2),
        "xy" => (0, 1),
        _ => (2, 1),
    };
    for cell in cells.values() {
        if spec.view != "top"
            && cell.min[if spec.view == "xy" { 2 } else { 0 }] != spec.slice.unwrap()
        {
            continue;
        }
        let key = (cell.min[horizontal], cell.min[vertical]);
        if spec.view != "top"
            || selected.get(&key).is_none_or(|old| {
                if cell.kind == "air" {
                    old.kind == "air" && cell.min[1] > old.min[1]
                } else {
                    old.kind == "air" || cell.min[1] > old.min[1]
                }
            })
        {
            selected.insert(key, cell.clone());
        }
    }
    let mut raster = Raster::new();
    let mut displayed = Vec::new();
    let color = [225, 234, 244];
    let s = &spec.survey;
    raster.text(8, 8, &format!("VOXEL {} - DIAGNOSTIC", spec.view), color);
    raster.text(8, 20, "0.5 M CELLS - Y UP - NO ENGINE MESH", color);
    for (axis, y) in [(0, 32), (1, 44), (2, 56)] {
        raster.text(
            8,
            y,
            &format!(
                "{} [{},{}) M",
                "XYZ".chars().nth(axis).unwrap(),
                f64::from(s.min[axis]) / 2.0,
                f64::from(s.max_exclusive[axis]) / 2.0
            ),
            color,
        );
    }
    if let Some(slice) = spec.slice {
        raster.text(
            8,
            68,
            &format!(
                "SECTION {} [{},{}) M",
                if spec.view == "xy" { "Z" } else { "X" },
                f64::from(slice) / 2.0,
                f64::from(slice + 1) / 2.0
            ),
            color,
        );
    } else {
        raster.text(8, 68, "TOP - HIGHEST CLASSIFIED NON-AIR IN BOX", color);
    }
    raster.text(
        8,
        80,
        &format!("PINS {} - CALLER REFERENCES ONLY", spec.pins.len()),
        color,
    );
    raster.text(
        LEFT - 24,
        TOP - 12,
        &format!("+{}", "XYZ".chars().nth(vertical).unwrap()),
        color,
    );
    raster.text(
        LEFT + s.size[horizontal] as usize * TILE + 4,
        TOP + s.size[vertical] as usize * TILE - 8,
        &format!("+{}", "XYZ".chars().nth(horizontal).unwrap()),
        color,
    );
    for ((col, row), cell) in &selected {
        let x = LEFT + (*col - s.min[horizontal]) as usize * TILE;
        let y = TOP + (s.max_exclusive[vertical] - 1 - *row) as usize * TILE;
        let shade = match cell.kind {
            "air" => [37, 51, 71],
            "terrain" => [181, 152, 107],
            "building" => [99, 183, 208],
            _ => [190, 151, 232],
        };
        raster.rect(x, y, TILE, TILE, [62, 80, 103]);
        raster.rect(x + 1, y + 1, TILE - 1, TILE - 1, shade);
        displayed.push(json!({"min_cells":cell.min,"kind":cell.kind,"id":cell.id,"fill":cell.fill,"pixel_rect":[x,y,TILE,TILE]}));
    }
    let mut pins = Vec::new();
    for (index, pin) in spec.pins.iter().enumerate() {
        let inside = (0..3).all(|i| {
            pin.position[i] >= f64::from(s.min[i]) / 2.0
                && pin.position[i] < f64::from(s.max_exclusive[i]) / 2.0
        });
        let in_view = inside
            && spec.slice.is_none_or(|slice| {
                let n = pin.position[if spec.view == "xy" { 2 } else { 0 }];
                n >= f64::from(slice) / 2.0 && n < f64::from(slice + 1) / 2.0
            });
        let pixel = if in_view {
            let x = LEFT
                + ((pin.position[horizontal] - f64::from(s.min[horizontal]) / 2.0) * 32.0).round()
                    as usize;
            let y = TOP
                + ((f64::from(s.max_exclusive[vertical]) / 2.0 - pin.position[vertical]) * 32.0)
                    .round() as usize;
            raster.rect(x - 7, y - 5, 15, 11, [245, 198, 108]);
            raster.text(x - 5, y - 3, &(index + 1).to_string(), [19, 32, 52]);
            Some([x, y])
        } else {
            None
        };
        if index < 2 {
            raster.text(
                8,
                400 + index * 12,
                &format!(
                    "{} {} [{:.2},{:.2},{:.2}]{}",
                    index + 1,
                    pin.label,
                    pin.position[0],
                    pin.position[1],
                    pin.position[2],
                    if in_view { "" } else { " OFF VIEW" }
                ),
                color,
            );
        }
        pins.push(json!({"number":index+1,"label":pin.label,"position_m":pin.position,"in_capture":inside,"in_view":in_view,"pixel":pixel}));
    }
    raster.text(
        8,
        380,
        "AIR DARK - TERRAIN TAN - BUILDING CYAN - UNKNOWN VIOLET",
        color,
    );
    raster.text(
        8,
        432,
        "SAMPLED OCCUPANCY - CONTACT AND CLEARANCE UNVERIFIED",
        color,
    );
    let metadata = json!({"source":"published_server_building_survey","view":spec.view,"section_min_m":spec.slice.map(|n|f64::from(n)/2.0),"section_max_exclusive_m":spec.slice.map(|n|f64::from(n+1)/2.0),"region":value["survey"]["region"],"sampled_cells":s.cell_count,"material_counts":counts,"displayed_cells":displayed,"pins":pins,"pin_scope":"caller reference positions only; not entities, selection or movement","image":{"width":WIDTH,"height":HEIGHT,"cell_pixels":TILE,"plot_origin":[LEFT,TOP],"right_axis":horizontal,"up_axis":vertical},"captured_unix_ms":value["captured_unix_ms"],"capture_clock":"host clock after voxel read; not game time","native_world_session":"unavailable","atomic_snapshot":false,"rendered_game_verified":false,"surface_height_verified":false,"clearance_verified":false,"world_mutated":false,"limits":{"cells":building_survey::MAX_CELLS,"pins":MAX_PINS},"limitations":["diagnostic classified cells, not interpolated engine surface or camera","Air is not physical clearance; fill is preserved without a height equation","top view may conceal lower cells, caves and structures","no props, collision, support, persistence or traversal proof","reference positions are rounded only for pixel placement"]});
    Ok(Presentation::Png {
        metadata,
        image: Png::rgb(WIDTH, HEIGHT, &raster.0)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use zune_png::{PngDecoder, zune_core::bytestream::ZCursor};

    fn request() -> Value {
        json!({"x":10,"y":20,"z":30,"width":1,"height":1.5,"depth":2})
    }
    fn fixture(args: Value) -> Value {
        let mut cells = Vec::new();
        for z in 60..64 {
            for y in 40..43 {
                for x in 20..22 {
                    let (kind, id, fill) = if [x, y, z] == [20, 40, 60] {
                        ("terrain", Some(2), 128)
                    } else if [x, y, z] == [21, 42, 63] {
                        ("building", Some(157), 7)
                    } else {
                        ("air", None, 0)
                    };
                    cells.push(
                        json!({"min_cells":[x,y,z],"material":{"kind":kind,"id":id},"fill":fill}),
                    );
                }
            }
        }
        json!({"request":args,"survey":{"status":"read_only_observation","grid":{"cell_size_m":0.5},"region":{"min_cells":[20,40,60],"size_cells":[2,3,4],"max_exclusive_cells":[22,43,64],"min_m":[10,20,30],"max_exclusive_m":[11,21.5,32]},"voxels":{"cell_count":24,"material_counts":{"air":22,"terrain":1,"building":1,"unknown":0},"cells":cells}}})
    }
    fn decode(value: Value) -> (Value, Vec<u8>) {
        let result = present(value).unwrap().result();
        let metadata =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        let bytes = STANDARD
            .decode(result["content"][1]["data"].as_str().unwrap())
            .unwrap();
        let pixels = PngDecoder::new(ZCursor::new(&bytes)).decode_raw().unwrap();
        (metadata, pixels)
    }
    fn pixel(pixels: &[u8], x: usize, y: usize) -> &[u8] {
        &pixels[(y * 448 + x) * 3..(y * 448 + x) * 3 + 3]
    }

    #[test]
    fn top_and_sections_preserve_asymmetric_axes_and_raw_voxels() {
        for (view, slice, cell, point, color) in [
            ("top", None, [20, 40, 60], [72, 168], [181, 152, 107]),
            ("top", None, [21, 42, 63], [88, 120], [99, 183, 208]),
            ("xy", Some(30.0), [20, 40, 60], [72, 152], [181, 152, 107]),
            ("zy", Some(10.5), [21, 42, 63], [120, 120], [99, 183, 208]),
        ] {
            let mut args = request();
            args["view"] = json!(view);
            if let Some(s) = slice {
                args["slice"] = json!(s);
            }
            let (meta, pixels) = decode(fixture(args));
            assert_eq!(pixel(&pixels, point[0], point[1]), color);
            let row = meta["displayed_cells"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["min_cells"] == json!(cell))
                .unwrap();
            assert_eq!(row["fill"], if cell == [20, 40, 60] { 128 } else { 7 });
            assert_eq!(meta["rendered_game_verified"], false);
            assert_eq!(meta["surface_height_verified"], false);
        }
    }

    #[test]
    fn unknown_and_air_fill_do_not_become_clearance_or_height() {
        let mut value = fixture(request());
        value["survey"]["voxels"]["cells"][0]["material"] = json!({"kind":"unknown","id":254});
        value["survey"]["voxels"]["material_counts"] =
            json!({"air":22,"terrain":0,"building":1,"unknown":1});
        value["survey"]["voxels"]["cells"][5]["fill"] = json!(255);
        let (meta, pixels) = decode(value);
        assert_eq!(pixel(&pixels, 72, 168), [190, 151, 232]);
        assert_eq!(meta["material_counts"]["unknown"], 1);
        assert!(
            meta["displayed_cells"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["min_cells"] == json!([21, 42, 60]) && c["fill"] == 255)
        );
        assert_eq!(meta["clearance_verified"], false);
    }

    #[test]
    fn complete_capture_is_required_even_for_cells_outside_selected_section() {
        for (change, fragment) in [
            (0, "complete"),
            (1, "duplicate"),
            (2, "outside"),
            (3, "counts"),
            (4, "region"),
            (5, "fill"),
        ] {
            let mut value = fixture({
                let mut a = request();
                a["view"] = json!("xy");
                a["slice"] = json!(30);
                a
            });
            let cells = &mut value["survey"]["voxels"]["cells"];
            match change {
                0 => {
                    cells.as_array_mut().unwrap().pop();
                }
                1 => cells[23] = cells[22].clone(),
                2 => cells[23]["min_cells"] = json!([22, 42, 63]),
                3 => value["survey"]["voxels"]["material_counts"]["terrain"] = json!(2),
                4 => value["survey"]["region"]["size_cells"] = json!([2, 3, 5]),
                _ => cells[23]["fill"] = json!(256),
            }
            assert!(present(value).unwrap_err().contains(fragment), "{change}");
        }
    }

    #[test]
    fn pins_keep_exact_coordinates_and_half_open_capture_scope() {
        let mut args = request();
        args["pins"] = json!([{"label":"Point","position":[10.25,20.75,31.25]},{"label":"Outside","position":[11,20.75,31.25]}]);
        let (meta, _) = decode(fixture(args));
        assert_eq!(meta["pins"][0]["position_m"], json!([10.25, 20.75, 31.25]));
        assert_eq!(meta["pins"][0]["pixel"], json!([72, 136]));
        assert_eq!(meta["pins"][1]["in_capture"], false);
        assert!(meta["pins"][1]["pixel"].is_null());
    }

    #[test]
    fn admission_rejects_ignored_arguments_bad_sections_and_excess_pins() {
        for (key, value, fragment) in [
            ("confirm", json!("server-write"), "unknown argument"),
            ("player", json!("any"), "unknown argument"),
            ("slice", json!(30), "top"),
            ("x", json!(10.25), "grid"),
        ] {
            let mut a = request();
            a[key] = value;
            assert!(
                check(a.as_object().unwrap())
                    .unwrap_err()
                    .contains(fragment)
            );
        }
        let mut a = request();
        a["view"] = json!("xy");
        assert!(check(a.as_object().unwrap()).unwrap_err().contains("slice"));
        a["slice"] = json!(32);
        assert!(
            check(a.as_object().unwrap())
                .unwrap_err()
                .contains("inside")
        );
        a["slice"] = json!(30);
        a["pins"] = json!(vec![json!({"label":"P","position":[10,20,30]}); 17]);
        assert!(check(a.as_object().unwrap()).unwrap_err().contains("16"));
    }

    #[test]
    fn pin_labels_nested_fields_world_bounds_and_aliases_are_checked() {
        let pin = json!({"label":"P","position":[10,20,30]});
        for value in [
            json!([pin.clone(), pin.clone()]),
            json!([{"label":" ","position":[10,20,30]}]),
            json!([{"label":"P\n","position":[10,20,30]}]),
            json!([{"label":"é","position":[10,20,30]}]),
            json!([{"label":"P","position":[10,1024.5,30]}]),
            json!([{"label":"P","position":[10,20]}]),
            json!([{"label":"P","position":[10,20,30],"entity":1}]),
        ] {
            let mut a = request();
            a["pins"] = value;
            assert!(check(a.as_object().unwrap()).is_err());
        }
    }

    #[test]
    fn section_pins_on_another_plane_are_listed_without_projection() {
        let mut args = request();
        args["view"] = json!("xy");
        args["slice"] = json!(30);
        args["pins"] = json!([{"label":"In","position":[10.25,20.75,30.25]},{"label":"Other","position":[10.25,20.75,30.5]}]);
        let (meta, _) = decode(fixture(args));
        assert_eq!(meta["pins"][0]["pixel"], json!([72, 136]));
        assert_eq!(meta["pins"][1]["in_capture"], true);
        assert_eq!(meta["pins"][1]["in_view"], false);
        assert!(meta["pins"][1]["pixel"].is_null());
        assert_eq!(meta["section_min_m"], 30.0);
        assert_eq!(meta["section_max_exclusive_m"], 30.5);
    }

    #[test]
    fn maximum_complete_capture_keeps_air_unknown_and_response_bounds_truthful() {
        let args = json!({"x":0,"y":0,"z":0,"width":8,"height":8,"depth":8});
        let mut cells = Vec::new();
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    cells.push(
                        json!({"min_cells":[x,y,z],"material":{"kind":"air","id":null},"fill":255}),
                    );
                }
            }
        }
        let value = json!({"request":args,"survey":{"status":"read_only_observation","grid":{"cell_size_m":0.5},"region":{"min_cells":[0,0,0],"size_cells":[16,16,16],"max_exclusive_cells":[16,16,16],"min_m":[0,0,0],"max_exclusive_m":[8,8,8]},"voxels":{"cell_count":4096,"cells":cells,"material_counts":{"air":4096,"terrain":0,"building":0,"unknown":0}}}});
        let result = present(value).unwrap().result();
        assert!(result.to_string().len() < 2 * 1024 * 1024);
        let meta: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(meta["sampled_cells"], 4096);
        assert_eq!(meta["displayed_cells"].as_array().unwrap().len(), 256);
        assert!(
            meta["displayed_cells"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["min_cells"][1] == 15 && c["fill"] == 255 && c["kind"] == "air")
        );
        assert_eq!(meta["surface_height_verified"], false);
    }
}
