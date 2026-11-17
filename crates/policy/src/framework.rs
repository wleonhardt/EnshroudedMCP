//! Immutable provider metadata, separate from compiled SDK provenance.
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub api_table_bytes: u32,
}

#[must_use]
pub fn observation(header: Option<Header>) -> Value {
    json!({"source":"published_host_api_immutable_header",
        "state":if header.is_some(){"observed"}else{"unavailable"},
        "abi_major":header.map(|h|h.abi_major),"abi_minor":header.map(|h|h.abi_minor),
        "api_table_bytes":header.map(|h|h.api_table_bytes),
        "scope":"header metadata only; signatures, API-call fidelity and gameplay are not verified"})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_provider_remains_unknown_without_compiled_defaults() {
        let value = observation(None);
        assert_eq!(value["state"], "unavailable");
        for field in ["abi_major", "abi_minor", "api_table_bytes"] {
            assert!(value[field].is_null());
        }
    }

    #[test]
    fn supplied_provider_values_survive_without_compatibility_or_trust_claims() {
        for (major, minor, bytes) in [(42, 99, 4096), (0, 0, 0), (u16::MAX, u16::MAX, u32::MAX)] {
            let value = observation(Some(Header {
                abi_major: major,
                abi_minor: minor,
                api_table_bytes: bytes,
            }));
            assert_eq!(value["state"], "observed");
            assert_eq!(value["abi_major"], major);
            assert_eq!(value["abi_minor"], minor);
            assert_eq!(value["api_table_bytes"], bytes);
            for field in ["compatible", "signature_verified", "gameplay_verified"] {
                assert!(value.get(field).is_none());
            }
        }
    }
}
