//! Inactive damage-hook observation; no product tool or native installation.
use enshwick::{Off, damage::DamageCause};
use serde_json::{Value, json};

/// Observe causes absent from damage-filter hooks after loader initialization.
///
/// # Errors
/// Preserves published unavailability, including no loader and unarmed hooks.
pub fn read() -> Result<Value, Off> {
    Ok(snapshot(&enshwick::damage::causes_off()?))
}

fn snapshot(causes: &[DamageCause]) -> Value {
    let mut causes = causes.to_vec();
    causes.sort_by_key(|cause| cause.to_abi());
    let rows: Vec<_> = causes
        .into_iter()
        .map(|cause| {
            let kind = match cause {
                DamageCause::Impact => "impact",
                DamageCause::FallDamage => "fall_damage",
                DamageCause::Fog => "fog",
                DamageCause::DeadlyFog => "deadly_fog",
                DamageCause::Suffocated => "suffocated",
                DamageCause::DrowningDeath => "drowning_death",
                DamageCause::Freezing => "freezing",
                DamageCause::AdminKill => "admin_kill",
                _ => "other",
            };
            json!({"abi_code":cause.to_abi(),"kind":kind})
        })
        .collect();
    json!({"source":"published_damage_causes_off","state":"observed",
        "missing_filter_causes":rows,
        "scope":"causes no registered damage filter receives; not current damage or prevention",
        "damage_prevention_verified":false,"quiet_conditions_verified":false,
        "write_authorized":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::Status;
    use enshwick::damage::DamageCause;
    use enshwick::test::FakeHost;
    use enshwick::{Load, Mod, Off};
    use std::cell::Cell;

    thread_local! {
        static FILTER: Cell<bool> = const { Cell::new(false) };
        static INIT_REFUSED: Cell<bool> = const { Cell::new(false) };
    }

    struct Observer;
    impl Mod for Observer {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            INIT_REFUSED.set(read().is_err());
            if FILTER.get() {
                // Test-only hook passes every hit unchanged; not damage protection.
                load.on_damage(|_| {})?;
            }
            Ok(Self)
        }
    }

    enshwick::test::declare_mod!(Observer, "id = \"test.damage-coverage\"\nside = \"server\"");

    fn start(host: FakeHost, filtering: bool) -> enshwick::test::Installed {
        FILTER.set(filtering);
        INIT_REFUSED.set(false);
        let loader = host.install();
        // SAFETY: this file's generated export runs only against the published fake host.
        assert_eq!(unsafe { loader.init(enshwick_mod_init) }, Status::OK);
        assert!(
            INIT_REFUSED.get(),
            "init cannot become an observed empty list"
        );
        loader
    }

    fn codes(value: &serde_json::Value) -> Vec<u32> {
        value["missing_filter_causes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|cause| u32::try_from(cause["abi_code"].as_u64().unwrap()).unwrap())
            .collect()
    }

    #[test]
    fn no_loader_is_typed_unavailability_not_empty_coverage() {
        assert!(read().is_err());
    }

    #[test]
    fn observer_without_a_filter_reports_every_named_cause_off() {
        let _loader = start(FakeHost::new(), false);
        let value = read().unwrap();
        let mut expected: Vec<_> = DamageCause::NAMED.iter().map(|c| c.to_abi()).collect();
        expected.sort_unstable();
        assert_eq!(codes(&value), expected);
        assert_eq!(value["source"], "published_damage_causes_off");
        assert_eq!(value["damage_prevention_verified"], false);
        assert_eq!(value["quiet_conditions_verified"], false);
    }

    #[test]
    fn available_hooks_and_empty_off_list_do_not_prove_protection() {
        let _loader = start(FakeHost::new(), true);
        let value = read().unwrap();
        assert!(codes(&value).is_empty());
        assert_eq!(value["damage_prevention_verified"], false);
        assert_eq!(value["quiet_conditions_verified"], false);
    }

    #[test]
    fn missing_fog_site_keeps_both_unobservable_fog_causes() {
        let _loader = start(
            FakeHost::new().without_damage_sites(&[DamageCause::Fog]),
            true,
        );
        let mut expected = vec![DamageCause::Fog.to_abi(), DamageCause::DeadlyFog.to_abi()];
        expected.sort_unstable();
        assert_eq!(codes(&read().unwrap()), expected);
    }

    #[test]
    fn missing_breathing_site_keeps_both_unobservable_breathing_causes() {
        let _loader = start(
            FakeHost::new().without_damage_sites(&[DamageCause::DrowningDeath]),
            true,
        );
        let mut expected = vec![
            DamageCause::Suffocated.to_abi(),
            DamageCause::DrowningDeath.to_abi(),
        ];
        expected.sort_unstable();
        assert_eq!(codes(&read().unwrap()), expected);
    }

    #[test]
    fn every_named_code_and_unknown_extremes_survive_without_float_conversion() {
        let mut causes = DamageCause::NAMED.to_vec();
        causes.extend([DamageCause::from_abi(99), DamageCause::from_abi(u32::MAX)]);
        let before = causes.clone();
        let value = snapshot(&causes);
        assert_eq!(causes, before);
        let mut expected: Vec<_> = causes.iter().map(|c| c.to_abi()).collect();
        expected.sort_unstable();
        assert_eq!(codes(&value), expected);
        let rows = value["missing_filter_causes"].as_array().unwrap();
        assert_eq!(rows[rows.len() - 1]["kind"], "other");
        assert_eq!(rows[rows.len() - 1]["abi_code"], u32::MAX);
    }

    #[test]
    fn unordered_duplicate_observations_are_sorted_without_silent_loss() {
        let a = [DamageCause::Fog, DamageCause::Impact, DamageCause::Fog];
        let b = [DamageCause::Fog, DamageCause::Fog, DamageCause::Impact];
        assert_eq!(snapshot(&a), snapshot(&b));
        assert_eq!(codes(&snapshot(&a)).len(), 3);
    }
}
