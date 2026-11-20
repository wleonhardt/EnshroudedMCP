//! Published any-thread cached sampler getter; no sampling request or commands.
use enshwick::input::Input;
use serde_json::Value;

pub(crate) fn read(input: Option<Input>) -> Result<Value, String> {
    let status = input
        .ok_or("native client input status API unavailable")?
        .local_status()
        .map_err(|error| error.to_string())?;
    Ok(policy::client_input::observation(
        policy::client_input::Status {
            requested: status.requested,
            layout_available: status.layout_available,
            link_available: status.link_available,
            read_enabled: status.read_enabled,
            sample_present: status.sample_present,
            fresh: status.fresh,
            age_ms: status.age_ms,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{
        Load, Mod, Off,
        abi::{Header, LocalInputInfo as Raw, Side, Status},
    };
    use std::sync::Mutex;
    static INPUT: Mutex<Option<Input>> = Mutex::new(None);
    struct Watcher;
    impl Mod for Watcher {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *INPUT.lock().unwrap() = Some(load.input()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Watcher,
        r#"id = "test.cached-input"
side = "both""#
    );
    fn start(side: Side) -> (enshwick::test::Installed, Option<Input>) {
        let loader = enshwick::test::FakeHost::new()
            .side(side)
            .player_input()
            .install();
        // SAFETY: these are this test module's generated exports on its own SDK fake.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let input = *INPUT.lock().unwrap();
        (loader, input)
    }
    #[test]
    fn cached_status_preserves_weak_flags_age_and_does_not_submit_commands() {
        let (loader, input) = start(Side::CLIENT);
        for (flags, age) in [
            (0, u64::MAX),
            (Raw::FRESH, 0),
            (Raw::REQUESTED | Raw::SAMPLE_PRESENT, 2000),
            (u32::MAX, u64::MAX - 1),
        ] {
            loader.set_local_input_status(Raw {
                header: Header::of::<Raw>(),
                flags,
                reserved: 0,
                age_ms: age,
            });
            let value = read(input).unwrap();
            for (key, mask) in [
                ("requested", Raw::REQUESTED),
                ("layout_available", Raw::LAYOUT_AVAILABLE),
                ("link_available", Raw::LINK_AVAILABLE),
                ("read_enabled", Raw::READ_ENABLED),
                ("sample_present", Raw::SAMPLE_PRESENT),
                ("fresh", Raw::FRESH),
            ] {
                assert_eq!(value[key], flags & mask != 0);
            }
            assert_eq!(
                value["age_ms"],
                serde_json::json!((age != u64::MAX).then_some(age))
            );
            assert_eq!(value["weakly_consistent"], true);
            assert_eq!(value["current_ui_verified"], false);
            assert_eq!(value["sampling_requested_by_this_read"], false);
            assert_eq!(read(input).unwrap(), value);
        }
        assert!(loader.client_commands().is_empty());
    }
    #[test]
    fn unavailable_handle_and_provider_refusal_do_not_invent_status() {
        assert!(read(None).unwrap_err().contains("unavailable"));
        let (loader, input) = start(Side::SERVER);
        assert!(read(input).is_err());
        assert!(loader.client_commands().is_empty());
    }
}
