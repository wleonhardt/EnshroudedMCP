//! Published server clock observations for experiment timing.

use enshwick::daytime::TimeOfDay;
use serde_json::{Value, json};

pub(crate) fn encode(value: Option<TimeOfDay>) -> Value {
    json!({"state":if value.is_some() {"observed"} else {"unavailable"},
        "source":"published_server_clock",
        "observation":value.map(|now| json!({"hours":now.hours,"is_day":now.is_day,
            "day_starts_hours":now.day_starts,"night_starts_hours":now.night_starts,
            "day_duration_seconds":now.day_time_duration.as_secs_f64(),
            "night_duration_seconds":now.night_time_duration.as_secs_f64(),"speed":now.speed})),
        "rendered_lighting_verified":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{DaytimeInfo, Header, Status};
    use enshwick::{Load, Mod, Off};
    use std::sync::Mutex;

    static CLOCK: Mutex<Option<enshwick::daytime::Clock>> = Mutex::new(None);
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CLOCK.lock().unwrap() = Some(load.clock());
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.clock"
side = "server""#
    );

    #[test]
    fn unavailable_differs_from_observed_midnight_and_sdk_units_are_explicit() {
        let loader = enshwick::test::FakeHost::new().install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let clock = CLOCK.lock().unwrap().unwrap();
        let missing = encode(clock.now());
        assert_eq!(missing["state"], "unavailable");
        assert!(missing["observation"].is_null());
        let hour = 3_600_000_000_000;
        loader.set_daytime(DaytimeInfo {
            header: Header::of::<DaytimeInfo>(),
            time: 0,
            day_start: 6 * hour,
            day_end: 22 * hour,
            day_length: 1_800_500_000_000,
            night_length: 720_000_000_000,
            speed: 2.0,
            flags: 0,
        });
        let value = encode(clock.now());
        assert_eq!(value["state"], "observed");
        assert_eq!(value["observation"]["hours"], 0.0);
        assert_eq!(value["observation"]["is_day"], false);
        assert_eq!(value["observation"]["day_starts_hours"], 6.0);
        assert_eq!(value["observation"]["night_starts_hours"], 22.0);
        assert_eq!(value["observation"]["day_duration_seconds"], 1800.5);
        assert_eq!(value["observation"]["night_duration_seconds"], 720.0);
        assert_eq!(value["observation"]["speed"], 2.0);
        assert_eq!(value["rendered_lighting_verified"], false);
    }
}
