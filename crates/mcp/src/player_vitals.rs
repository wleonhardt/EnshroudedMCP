//! Plain serialization of published SDK observations, without UI conversions.
use enshwick::world::{Meter, Vitals};
use serde_json::{Value, json};

fn meter(value: Meter) -> Value {
    json!({"current":value.current,"max":value.max})
}

pub(crate) fn encode(vitals: Option<Vitals>) -> Value {
    json!({
        "state":if vitals.is_some() {"available"} else {"unavailable"},
        "health":vitals.map(|v|meter(v.health)),
        "stamina":vitals.and_then(|v|v.stamina).map(meter),
        "mana":vitals.and_then(|v|v.mana).map(meter),
        "units":"SDK integer attribute values",
        "ui_semantics_verified":false,
        "null_semantics":"unavailable or absent; not zero"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_is_distinct_from_observed_zero_health() {
        let absent = encode(None);
        assert_eq!(absent["state"], "unavailable");
        for pool in ["health", "stamina", "mana"] {
            assert!(absent[pool].is_null());
        }
        let mut vitals = Vitals::default();
        vitals.health = Meter {
            current: 0,
            max: 150,
        };
        let observed = encode(Some(vitals));
        assert_eq!(observed["state"], "available");
        assert_eq!(observed["health"], json!({"current":0,"max":150}));
        assert!(observed["stamina"].is_null());
        assert!(observed["mana"].is_null());
        assert_eq!(absent["null_semantics"], "unavailable or absent; not zero");
    }

    #[test]
    fn signed_raw_values_and_optional_pools_are_not_rescaled_or_clamped() {
        let mut vitals = Vitals::default();
        vitals.health = Meter {
            current: -7,
            max: 0,
        };
        vitals.stamina = Some(Meter {
            current: 137,
            max: 111,
        });
        let stamina_only = encode(Some(vitals));
        assert_eq!(stamina_only["health"], json!({"current":-7,"max":0}));
        assert_eq!(stamina_only["stamina"], json!({"current":137,"max":111}));
        assert!(stamina_only["mana"].is_null());
        vitals.stamina = None;
        vitals.mana = Some(Meter {
            current: i32::MIN,
            max: i32::MAX,
        });
        let mana_only = encode(Some(vitals));
        assert!(mana_only["stamina"].is_null());
        assert_eq!(
            mana_only["mana"],
            json!({"current":i32::MIN,"max":i32::MAX})
        );
        assert_eq!(mana_only["units"], "SDK integer attribute values");
        assert_eq!(mana_only["ui_semantics_verified"], false);
        for pool in ["health", "mana"] {
            let object = mana_only[pool].as_object().unwrap();
            assert_eq!(object.len(), 2);
            assert!(object.contains_key("current") && object.contains_key("max"));
        }
    }
}
