//! Game-free admission for the embedded research catalogue.

use serde_json::{Map, Value, json};

/// Evidence categories are not a universal confidence ranking.
pub const STATUSES: [&str; 7] = [
    "documented",
    "static_inspected",
    "live_observed",
    "cross_checked",
    "disputed",
    "stale",
    "hypothesis",
];

/// Maximum UTF-8 bytes in a search query.
pub const QUERY_MAX: usize = 256;
/// Maximum records in one response.
pub const LIMIT_MAX: usize = 20;

/// Validated search inputs. All terms must match, without case sensitivity.
#[derive(Debug, PartialEq, Eq)]
pub struct Query {
    pub terms: Vec<String>,
    pub status: Option<String>,
    pub limit: usize,
}

/// The tool's advertised input contract.
#[must_use]
pub fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "query": {"type": "string", "description": "All whitespace-separated terms must match, case-insensitively; at most 256 UTF-8 bytes. Omit to browse."},
            "status": {"type": "string", "enum": STATUSES},
            "limit": {"type": "integer", "minimum": 1, "maximum": LIMIT_MAX, "default": 5}
        }
    })
}

/// Validates arguments even when called outside MCP dispatch.
///
/// # Errors
/// Refuses unknown fields, wrong types, unsupported statuses, overlong queries,
/// and limits outside 1..=20.
pub fn check(args: &Map<String, Value>) -> Result<Query, String> {
    crate::args::check(&schema(), args)?;
    let query = args.get("query").and_then(Value::as_str).unwrap_or("");
    if query.len() > QUERY_MAX {
        return Err(format!("query must be at most {QUERY_MAX} UTF-8 bytes"));
    }
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(5);
    let limit = usize::try_from(limit)
        .ok()
        .filter(|limit| (1..=LIMIT_MAX).contains(limit))
        .ok_or_else(|| format!("limit must be between 1 and {LIMIT_MAX}"))?;
    Ok(Query {
        terms: query.split_whitespace().map(str::to_lowercase).collect(),
        status: args
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_owned),
        limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn defaults_and_term_normalization() {
        assert_eq!(
            check(&Map::new()).unwrap(),
            Query {
                terms: Vec::new(),
                status: None,
                limit: 5,
            }
        );
        let result = check(&obj(
            json!({"query": "  Altar\tBOUNDARIES ", "limit":20, "status":"hypothesis"}),
        ))
        .unwrap();
        assert_eq!(result.terms, ["altar", "boundaries"]);
        assert_eq!(result.limit, 20);
        assert_eq!(result.status.as_deref(), Some("hypothesis"));
    }

    #[test]
    fn limits_are_not_silently_coerced() {
        for limit in [
            json!(-1),
            json!(0),
            json!(21),
            json!(1.5),
            json!("5"),
            json!(null),
            json!(u64::MAX),
        ] {
            assert!(check(&obj(json!({"limit": limit}))).is_err(), "{limit}");
        }
        assert!(check(&obj(json!({"limit":1}))).is_ok());
    }

    #[test]
    fn byte_bound_applies_to_unicode_queries() {
        assert!(check(&obj(json!({"query":"é".repeat(128)}))).is_ok());
        assert!(check(&obj(json!({"query":"é".repeat(129)}))).is_err());
        assert!(check(&obj(json!({"query":"x".repeat(257)}))).is_err());
    }

    #[test]
    fn unknown_types_and_status_are_refused() {
        for value in [
            json!({"query":false}),
            json!({"status":"verified"}),
            json!({"status":null}),
            json!({"limti":5}),
        ] {
            assert!(check(&obj(value)).is_err());
        }
        for status in STATUSES {
            assert!(check(&obj(json!({"status":status}))).is_ok());
        }
    }
}
