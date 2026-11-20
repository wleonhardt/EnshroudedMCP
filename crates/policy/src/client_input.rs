//! Cached sampler metadata only; weakly consistent fields grant no UI authority.
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub requested: bool,
    pub layout_available: bool,
    pub link_available: bool,
    pub read_enabled: bool,
    pub sample_present: bool,
    pub fresh: bool,
    pub age_ms: Option<u64>,
}

pub fn schema() -> Value {
    json!({"type":"object"})
}

pub fn observation(status: Status) -> Value {
    json!({"source":"published_cached_local_input_status",
        "requested":status.requested,"layout_available":status.layout_available,
        "link_available":status.link_available,"read_enabled":status.read_enabled,
        "sample_present":status.sample_present,"fresh":status.fresh,"age_ms":status.age_ms,
        "weakly_consistent":true,"sampling_requested_by_this_read":false,
        "current_ui_verified":false,"commands_submitted":false,"rendered_verified":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_argument_can_request_sampling_or_mutation() {
        assert!(crate::args::check(&schema(), json!({}).as_object().unwrap()).is_ok());
        for args in [
            json!({"requested":true}),
            json!({"confirm":"server-write"}),
            json!({"dry":false}),
        ] {
            assert!(crate::args::check(&schema(), args.as_object().unwrap()).is_err());
        }
    }
    #[test]
    fn independent_cached_flags_and_unknown_age_never_become_current_ui() {
        for bits in 0..64 {
            for age in [None, Some(0), Some(1001), Some(u64::MAX - 1)] {
                let value = observation(Status {
                    requested: bits & 1 != 0,
                    layout_available: bits & 2 != 0,
                    link_available: bits & 4 != 0,
                    read_enabled: bits & 8 != 0,
                    sample_present: bits & 16 != 0,
                    fresh: bits & 32 != 0,
                    age_ms: age,
                });
                for (i, key) in [
                    "requested",
                    "layout_available",
                    "link_available",
                    "read_enabled",
                    "sample_present",
                    "fresh",
                ]
                .iter()
                .enumerate()
                {
                    assert_eq!(value[key], bits & (1 << i) != 0);
                }
                assert_eq!(value["age_ms"], json!(age));
                assert_eq!(value["weakly_consistent"], true);
                for key in [
                    "sampling_requested_by_this_read",
                    "current_ui_verified",
                    "commands_submitted",
                    "rendered_verified",
                ] {
                    assert_eq!(value[key], false);
                }
                assert!(value.get("session").is_none());
                assert!(value.get("player").is_none());
            }
        }
    }
}
