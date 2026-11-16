//! Admission for vanilla workshop definition observations, never world selectors.
use crate::{client_recipe as gate, item_lookup::span};
use serde_json::{Map, Value, json};

pub const MAX_ENTRIES: usize = 4096;

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "workshop_id":{"type":"integer","minimum":0,"maximum":u32::MAX},
        "limit":{"type":"integer","minimum":1,"maximum":8}
    },"required":["workshop_id"]})
}
pub fn check(args: &Map<String, Value>) -> Result<(u32, usize), String> {
    crate::args::check(&schema(), args)?;
    let id = args["workshop_id"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("workshop_id must be a u32")?;
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(4) as usize;
    Ok((id, limit))
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub array: usize,
    pub stride: usize,
    pub guid: usize,
    pub id: usize,
    pub name: usize,
    pub item: usize,
    pub item_ref: usize,
    pub npc: usize,
}
impl Layout {
    pub fn check(self, bytes: usize, limit: usize) -> Result<(), String> {
        gate::request(bytes, limit)?;
        gate::stride(self.stride)?;
        span(self.array, 1, 8, bytes)?;
        for (at, size) in [
            (self.guid, 16),
            (self.id, 4),
            (self.name, 4),
            (self.item, 4),
            (self.item_ref, 16),
            (self.npc, 1),
        ] {
            gate::field(at, size, self.stride)?;
        }
        Ok(())
    }
    pub fn entries(self, start: usize, count: usize, bytes: usize) -> Result<(), String> {
        gate::count(count, MAX_ENTRIES)?;
        span(start, count, self.stride, bytes).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_is_a_raw_workshop_id_with_strict_limits_and_namespace() {
        assert_eq!(
            check(json!({"workshop_id":0,"limit":8}).as_object().unwrap()).unwrap(),
            (0, 8)
        );
        for id in [0, u32::MAX] {
            assert_eq!(
                check(json!({"workshop_id":id}).as_object().unwrap()).unwrap(),
                (id, 4)
            );
        }
        for value in [
            json!({}),
            json!({"item_id":7}),
            json!({"workshop_id":-1}),
            json!({"workshop_id":1.5}),
            json!({"workshop_id":true}),
            json!({"workshop_id":"7"}),
            json!({"workshop_id":4294967296_u64}),
            json!({"workshop_id":7,"limit":0}),
            json!({"workshop_id":7,"limit":9}),
            json!({"workshop_id":7,"confirm":"server-write"}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
    }
    #[test]
    fn layouts_and_complete_arrays_are_bounded_before_traversal() {
        let l = Layout {
            array: 0,
            stride: 56,
            guid: 0,
            id: 16,
            name: 20,
            item: 24,
            item_ref: 28,
            npc: 44,
        };
        assert!(l.check(64, 8).is_ok());
        assert!(l.entries(8, 1, 64).is_ok());
        assert!(l.entries(8, MAX_ENTRIES, 8 + MAX_ENTRIES * 56).is_ok());
        assert!(l.entries(8, 2, 64).is_err());
        assert!(l.entries(8, MAX_ENTRIES + 1, usize::MAX).is_err());
        assert!(l.entries(usize::MAX, 1, usize::MAX).is_err());
        for bad in [
            Layout { npc: 56, ..l },
            Layout { guid: 41, ..l },
            Layout { item: 53, ..l },
            Layout { item_ref: 41, ..l },
            Layout { array: 57, ..l },
            Layout { stride: 4097, ..l },
        ] {
            assert!(bad.check(64, 8).is_err());
        }
        assert!(l.check(crate::item_lookup::MAX_BYTES + 1, 8).is_err());
        assert!(l.check(64, 9).is_err());
    }
}
