//! Published server weather observations; no weather mutation.
use enshwick::weather::Weather;
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "weather_read",
        description: "Read published server weather weights at one bounded world position in metres on the game thread. Unavailable differs from clear. Preserve finite clear/rain/snow/blizzard weights without normalization; report SDK strongest-state only for usable nonzero unit weights. Null within weights means a non-finite SDK value. No front enumeration, forecast change, rendered-weather, shelter or actor-safety proof.",
        schema: policy::weather_read::schema(),
    }
}

pub(crate) fn read(weather: Weather, position: [f64; 3]) -> Value {
    let sample = weather.at(position);
    let weights = sample.map(|s| [s.clear, s.rain, s.snow, s.blizzard]);
    let usable = weights.map(policy::weather_read::dominance_usable);
    json!({"source":"published_server_weather_at","position_m":position,
        "state":if sample.is_some(){"observed"}else{"unavailable"},
        "weights":weights.map(|[clear,rain,snow,blizzard]|json!({"clear":clear,"rain":rain,"snow":snow,"blizzard":blizzard})),
        "weight_sum":weights.map(|w|w.into_iter().map(f64::from).sum::<f64>()).filter(|sum|sum.is_finite()),
        "weights_usable_for_dominance":usable,
        "dominant_sdk_state":sample.filter(|_|usable==Some(true)).map(|s|format!("{:?}",s.state()).to_ascii_lowercase()),
        "weights_normalized_by_mcp":false,"world_mutated":false,
        "rendered_weather_verified":false,"shelter_verified":false,"actor_safety_verified":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Load, Mod, Off, abi::Status, schedule::Scheduler, weather::Weather};
    use std::{
        sync::{Mutex, mpsc},
        time::Duration,
    };
    static CONTEXT: Mutex<Option<(Weather, Scheduler)>> = Mutex::new(None);
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some((load.weather(), load.scheduler()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Probe, "id = \"test.weather-read\"\nside = \"server\"");

    #[test]
    fn deferred_samples_preserve_absence_weights_dominance_and_never_start_weather() {
        let loader = enshwick::test::FakeHost::new().install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let (weather, scheduler) = CONTEXT.lock().unwrap().unwrap();
        let sample = || {
            let (tx, rx) = mpsc::channel();
            scheduler.next_tick(move || {
                tx.send(read(weather, [1.25, 2.5, 3.75])).unwrap();
            });
            assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            let _ = loader.tick(Duration::from_millis(100));
            rx.try_recv().unwrap()
        };
        let absent = sample();
        assert_eq!(absent["state"], "unavailable");
        assert!(absent["weights"].is_null());
        assert!(absent["dominant_sdk_state"].is_null());
        for (weights, dominant) in [
            ([1., 0., 0., 0.], "clear"),
            ([0., 1., 0., 0.], "rain"),
            ([0., 0., 1., 0.], "snow"),
            ([0., 0., 0., 1.], "blizzard"),
            ([0.25; 4], "clear"),
            ([0.8, 0.8, 0., 0.], "clear"),
        ] {
            loader.set_weather(weights, &[]);
            let value = sample();
            assert_eq!(value["state"], "observed");
            assert_eq!(value["dominant_sdk_state"], dominant);
            assert_eq!(value["weights_usable_for_dominance"], true);
            for (name, expected) in ["clear", "rain", "snow", "blizzard"]
                .into_iter()
                .zip(weights)
            {
                assert_eq!(value["weights"][name].as_f64(), Some(f64::from(expected)));
            }
            assert_eq!(
                value["weight_sum"].as_f64(),
                Some(weights.into_iter().map(f64::from).sum())
            );
            for key in [
                "world_mutated",
                "rendered_weather_verified",
                "shelter_verified",
                "actor_safety_verified",
                "weights_normalized_by_mcp",
            ] {
                assert_eq!(value[key], false);
            }
        }
        for weights in [
            [0.; 4],
            [f32::NAN, 0., 0., 0.],
            [f32::INFINITY, 0., 0., 0.],
            [-0.1, 1., 0., 0.],
            [1.1, 0., 0., 0.],
        ] {
            loader.set_weather(weights, &[]);
            let value = sample();
            assert_eq!(value["state"], "observed");
            assert_eq!(value["weights_usable_for_dominance"], false);
            assert!(value["dominant_sdk_state"].is_null());
            assert!(value["weights"].is_object());
            for (name, expected) in ["clear", "rain", "snow", "blizzard"]
                .into_iter()
                .zip(weights)
            {
                if expected.is_finite() {
                    assert_eq!(value["weights"][name].as_f64(), Some(f64::from(expected)));
                } else {
                    assert!(value["weights"][name].is_null());
                }
            }
            if !weights[0].is_finite() {
                assert!(value["weights"]["clear"].is_null());
                assert!(value["weight_sum"].is_null());
            }
        }
        assert!(loader.forecasts().is_empty());
    }
}
