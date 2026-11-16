//! Admission for one read-only published server weather sample.
use serde_json::{Map, Value, json};

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "x":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE},
        "y":{"type":"number","minimum":0,"maximum":crate::bounds::HEIGHT},
        "z":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE}},"required":["x","y","z"]})
}
pub fn check(args: &Map<String, Value>) -> Result<[f64; 3], String> {
    crate::args::check(&schema(), args)?;
    let point = crate::args::point(args, "x", "y", "z");
    if !crate::bounds::in_world(point) {
        return Err("weather sample point is outside world bounds".into());
    }
    Ok(point)
}
pub fn dominance_usable(weights: [f32; 4]) -> bool {
    weights
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        && weights.iter().any(|v| *v > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn one_finite_world_point_is_required_without_write_or_front_selectors() {
        for value in [
            json!({"x":0,"y":0,"z":0}),
            json!({"x":10240,"y":1024,"z":10240}),
            json!({"x":1.25,"y":2.5,"z":3.75}),
        ] {
            assert!(check(value.as_object().unwrap()).is_ok());
        }
        for value in [
            json!({}),
            json!({"x":1,"y":2}),
            json!({"x":true,"y":2,"z":3}),
            json!({"x":-1,"y":2,"z":3}),
            json!({"x":1,"y":1024.5,"z":3}),
            json!({"x":10240.5,"y":2,"z":3}),
            json!({"x":null,"y":2,"z":3}),
            json!({"x":1,"y":2,"z":3,"confirm":"server-write"}),
            json!({"x":1,"y":2,"z":3,"fronts":true}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
    }

    #[test]
    fn dominance_requires_finite_unit_weights_with_a_positive_signal() {
        for weights in [
            [1., 0., 0., 0.],
            [0.25, 0.25, 0.25, 0.25],
            [0.8, 0.8, 0., 0.],
        ] {
            assert!(dominance_usable(weights));
        }
        for weights in [
            [0.; 4],
            [f32::NAN, 0., 0., 0.],
            [f32::INFINITY, 0., 0., 0.],
            [-0.1, 1., 0., 0.],
            [1.1, 0., 0., 0.],
        ] {
            assert!(!dominance_usable(weights));
        }
    }
}
