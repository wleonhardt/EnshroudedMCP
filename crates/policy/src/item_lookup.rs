//! Admission and workload limits for vanilla registry identity reads.

use serde_json::{Map, Value, json};

pub const MAX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 65_536;
pub const MAX_NAME: usize = 128;
pub const MAX_MATCHES: usize = 8;

pub fn registry_limits(bytes: usize, entries: usize, names: usize) -> Result<(), String> {
    if bytes > MAX_BYTES || entries > MAX_ENTRIES || entries != names {
        return Err("registry exceeds limits or reference/name counts differ".into());
    }
    Ok(())
}

pub fn name_limit(bytes: usize) -> Result<(), String> {
    if bytes > MAX_NAME {
        return Err("registry debug name exceeds 128 bytes".into());
    }
    Ok(())
}

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{"ids":{"type":"array","minItems":1,"maxItems":8,"uniqueItems":true,"items":{"type":"integer","minimum":1,"maximum":u32::MAX}}},"required":["ids"]})
}

pub fn check(args: &Map<String, Value>) -> Result<Vec<u32>, String> {
    crate::args::check(&schema(), args)?;
    let values = args["ids"].as_array().ok_or("ids must be an array")?;
    if values.is_empty() || values.len() > 8 {
        return Err("ids must contain 1..8 unique nonzero u32 values".into());
    }
    let mut ids = Vec::new();
    for value in values {
        let id = value
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n != 0)
            .ok_or("ids must contain nonzero u32 values")?;
        if ids.contains(&id) {
            return Err("ids must be unique".into());
        }
        ids.push(id);
    }
    Ok(ids)
}

/// Validate multiplication, addition and complete spans before traversal.
pub fn span(
    start: usize,
    count: usize,
    stride: usize,
    len: usize,
) -> Result<std::ops::Range<usize>, String> {
    let end = count
        .checked_mul(stride)
        .and_then(|n| start.checked_add(n))
        .filter(|end| *end <= len)
        .ok_or("registry span is truncated or overflows")?;
    Ok(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn selectors_are_bounded_unique_nonzero_integers() {
        assert_eq!(
            check(json!({"ids":[1,4294967295_u64]}).as_object().unwrap()).unwrap(),
            vec![1, u32::MAX]
        );
        for value in [
            json!({}),
            json!({"ids":[]}),
            json!({"ids":[1,1]}),
            json!({"ids":[0]}),
            json!({"ids":[1.5]}),
            json!({"ids":[-1]}),
            json!({"ids":[4294967296_u64]}),
            json!({"ids":[1],"unknown":true}),
            json!({"ids":[1,2,3,4,5,6,7,8,9]}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err(), "{value}");
        }
    }

    #[test]
    fn spans_refuse_overflow_and_truncation() {
        assert_eq!(span(8, 2, 16, 40).unwrap(), 8..40);
        assert!(span(8, 2, 16, 39).is_err());
        assert!(span(usize::MAX, 1, 1, usize::MAX).is_err());
        assert!(span(0, usize::MAX, 16, usize::MAX).is_err());
    }

    #[test]
    fn registry_work_is_bounded() {
        assert!(registry_limits(MAX_BYTES, MAX_ENTRIES, MAX_ENTRIES).is_ok());
        assert!(registry_limits(MAX_BYTES + 1, 0, 0).is_err());
        assert!(registry_limits(1, MAX_ENTRIES + 1, MAX_ENTRIES + 1).is_err());
        assert!(registry_limits(1, 1, 0).is_err());
        assert!(name_limit(MAX_NAME).is_ok());
        assert!(name_limit(MAX_NAME + 1).is_err());
    }
}
