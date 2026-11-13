//! Tool arguments against the tool's input schema. An argument the schema does
//! not name is refused, as in ValheimMCP's decision 0015: a misspelt `dry`
//! must not silently become a real write.

use serde_json::{Map, Value};

/// `Ok` when `args` names only properties of `schema`, has every required one,
/// and each has the schema's type (`number`, `integer`, `string`, `boolean`,
/// `array`, `object`). Also enforces enum and numeric bounds. Tool-specific
/// policy modules validate nested values, lengths and workload limits.
///
/// # Errors
/// The first problem found, worded for the caller.
pub fn check(schema: &Value, args: &Map<String, Value>) -> Result<(), String> {
    let empty = Map::new();
    let props = schema
        .get("properties")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    for (name, value) in args {
        let Some(prop) = props.get(name) else {
            let known: Vec<&str> = props.keys().map(String::as_str).collect();
            return Err(format!(
                "unknown argument '{name}' (known: {})",
                if known.is_empty() {
                    "none".into()
                } else {
                    known.join(", ")
                }
            ));
        };
        let ok = match prop.get("type").and_then(Value::as_str) {
            Some("number") => value.as_f64().is_some_and(f64::is_finite),
            Some("integer") => value.as_i64().is_some(),
            Some("string") => value.is_string(),
            Some("boolean") => value.is_boolean(),
            Some("array") => value.is_array(),
            Some("object") => value.is_object(),
            _ => true,
        };
        if !ok {
            return Err(format!(
                "argument '{name}' must be a {}",
                prop["type"].as_str().unwrap_or("value")
            ));
        }
        if let Some(allowed) = prop.get("enum").and_then(Value::as_array)
            && !allowed.contains(value)
        {
            let names: Vec<String> = allowed.iter().map(Value::to_string).collect();
            return Err(format!(
                "argument '{name}' must be one of {}",
                names.join(", ")
            ));
        }
        let max = prop.get("maximum").and_then(Value::as_f64);
        if let (Some(max), Some(v)) = (max, value.as_f64())
            && v > max
        {
            return Err(format!("argument '{name}' must be at most {max}"));
        }
        if let (Some(min), Some(v)) = (prop.get("minimum").and_then(Value::as_f64), value.as_f64())
            && v < min
        {
            return Err(format!("argument '{name}' must be at least {min}"));
        }
    }
    for req in schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !args.contains_key(req) {
            return Err(format!("missing argument '{req}'"));
        }
    }
    Ok(())
}

/// A number argument already checked by [`check`], or `default`.
#[must_use]
pub fn num(args: &Map<String, Value>, name: &str, default: f64) -> f64 {
    args.get(name).and_then(Value::as_f64).unwrap_or(default)
}

/// The point `[x, y, z]` from three number arguments.
#[must_use]
pub fn point(args: &Map<String, Value>, x: &str, y: &str, z: &str) -> [f64; 3] {
    [
        num(args, x, f64::NAN),
        num(args, y, f64::NAN),
        num(args, z, f64::NAN),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema() -> Value {
        json!({"type": "object", "properties": {"x": {"type": "number"}, "dry": {"type": "boolean"}, "n": {"type": "integer"}}, "required": ["x"]})
    }

    fn obj(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn known_arguments_pass() {
        assert_eq!(
            check(&schema(), &obj(json!({"x": 1.5, "dry": true, "n": 3}))),
            Ok(())
        );
    }

    #[test]
    fn an_unknown_argument_is_refused() {
        let err = check(&schema(), &obj(json!({"x": 1, "dyr": true}))).unwrap_err();
        assert!(err.starts_with("unknown argument 'dyr'"), "{err}");
        assert!(err.contains("dry"));
    }

    #[test]
    fn a_missing_required_argument_is_refused() {
        assert_eq!(
            check(&schema(), &obj(json!({}))),
            Err("missing argument 'x'".into())
        );
    }

    #[test]
    fn a_wrong_type_is_refused() {
        assert!(check(&schema(), &obj(json!({"x": "1"}))).is_err());
        assert!(check(&schema(), &obj(json!({"x": 1, "n": 1.5}))).is_err());
        assert!(check(&schema(), &obj(json!({"x": 1, "dry": "yes"}))).is_err());
    }

    #[test]
    fn a_schema_without_properties_takes_no_arguments() {
        let none = json!({"type": "object"});
        assert_eq!(check(&none, &Map::new()), Ok(()));
        assert!(
            check(&none, &obj(json!({"a": 1})))
                .unwrap_err()
                .contains("known: none")
        );
    }

    #[test]
    fn enum_and_maximum_are_enforced() {
        let s = json!({"type": "object", "properties": {
            "kind": {"type": "string", "enum": ["placed", "containers"]},
            "radius": {"type": "number", "maximum": 32},
        }});
        assert_eq!(
            check(&s, &obj(json!({"kind": "placed", "radius": 32}))),
            Ok(())
        );
        let err = check(&s, &obj(json!({"kind": "all"}))).unwrap_err();
        assert!(err.contains("one of \"placed\", \"containers\""), "{err}");
        assert!(
            check(&s, &obj(json!({"radius": 32.5})))
                .unwrap_err()
                .contains("at most 32")
        );
    }

    #[test]
    fn point_reads_three_numbers() {
        assert_eq!(
            point(&obj(json!({"x": 1, "y": 2.5, "z": 3})), "x", "y", "z"),
            [1.0, 2.5, 3.0]
        );
        assert!(point(&Map::new(), "x", "y", "z")[0].is_nan());
    }

    #[test]
    fn structured_types_and_minimum_are_enforced() {
        let s = json!({"properties": {
            "recipes": {"type": "array"},
            "stock": {"type": "object"},
            "limit": {"type": "integer", "minimum": 1, "maximum": 20}
        }});
        assert!(check(&s, &obj(json!({"recipes": [], "stock": {}, "limit": 1}))).is_ok());
        for bad in [
            json!({"recipes": {}}),
            json!({"stock": []}),
            json!({"limit": 0}),
        ] {
            assert!(check(&s, &obj(bad)).is_err());
        }
    }
}
