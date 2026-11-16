//! Copied SDK state only. No world access or command submission.
use enshwick::input::{ClientSnapshot, Input, SnapshotPart};
use serde_json::{Value, json};

pub(crate) fn read(input: Option<Input>) -> Result<Value, String> {
    let snapshot = input
        .ok_or("native client snapshot API unavailable")?
        .client_snapshot()
        .map_err(|error| error.to_string())?;
    Ok(encode(snapshot))
}

fn encode(snapshot: ClientSnapshot) -> Value {
    let screens = snapshot.reads(SnapshotPart::Screens);
    let tab = snapshot.reads(SnapshotPart::MenuTab);
    let held = snapshot.reads(SnapshotPart::Held);
    let player = snapshot.reads(SnapshotPart::Player);
    let recipe = snapshot.reads(SnapshotPart::Recipe);
    json!({
        "source":"published_native_client_snapshot",
        "state":format!("{:?}",snapshot.state()).to_ascii_lowercase(),
        "session":snapshot.session(),"age_ms":snapshot.age_ms(),
        "reads":{"screens":screens,"menu_tab":tab,"held":held,"player":player,"recipe":recipe},
        "hud_screen":screens.then(||snapshot.hud_screen()).flatten(),
        "return_screen":screens.then(||snapshot.return_screen()).flatten(),
        "host":snapshot.host().map(|entity|entity.0),
        "menu_tab":tab.then(||snapshot.menu_tab()).flatten(),
        "held_actions":snapshot.held().map(|mask|mask.iter().map(|action|action.name()).collect::<Vec<_>>()),
        "player":snapshot.player().filter(|entity|entity.0!=0).map(|entity|entity.0),
        "highlighted_recipe_id":snapshot.highlighted_recipe().map(|id|id.0),
        "entity_world":"client",
        "identity_scope":"this client session only; never use client entity ids as server-world selectors",
        "null_semantics":"reads=false means unavailable; reads=true with null means none observed; recipe id zero is the SDK's no-highlight sentinel",
        "commands_submitted":false,"rendered_verified":false
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{ClientSnapshot as Raw, Side, Status, Str};
    use enshwick::{Entity, Load, Mod, Off, RecipeId};
    use std::sync::Mutex;

    static INPUT: Mutex<Option<Input>> = Mutex::new(None);
    struct Watcher;
    impl Mod for Watcher {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *INPUT.lock().unwrap() = Some(load.input()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Watcher, "id = \"test.native-snapshot\"\nside = \"both\"");

    #[test]
    fn copied_state_preserves_freshness_availability_and_client_identity_without_commands() {
        assert!(read(None).is_err());
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        let status =
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) };
        assert_eq!(status, Status::OK);
        let input = *INPUT.lock().unwrap();
        let never = read(input).unwrap();
        assert_eq!(never["state"], "never");
        assert_eq!(never["session"], 0);
        assert!(never["age_ms"].is_null());
        for (state, expected, age) in [
            (Raw::FRESH, "fresh", 16),
            (Raw::STALE, "stale", 4200),
            (Raw::OFF, "off", 4200),
        ] {
            loader.set_client_snapshot(Raw {
                state,
                age_ms: age,
                session: 3,
                available: Raw::SCREENS | Raw::MENU_TAB | Raw::HELD | Raw::PLAYER | Raw::RECIPE,
                hud_screen: Str::new("PlayerCrafting"),
                menu_tab: Str::new("Backpack"),
                player: Entity(512),
                highlighted_recipe: RecipeId(73),
                ..Raw::EMPTY
            });
            let value = read(input).unwrap();
            assert_eq!(value["state"], expected);
            assert_eq!(value["age_ms"], age);
            assert_eq!(value["session"], 3);
            assert_eq!(value["hud_screen"], "PlayerCrafting");
            assert_eq!(value["menu_tab"], "Backpack");
            assert_eq!(value["player"], 512);
            assert_eq!(value["highlighted_recipe_id"], 73);
            assert_eq!(value["entity_world"], "client");
            assert_eq!(value["commands_submitted"], false);
        }
        for available in [0, Raw::MENU_TAB | Raw::PLAYER | Raw::RECIPE] {
            loader.set_client_snapshot(Raw {
                state: Raw::FRESH,
                session: 4,
                age_ms: 0,
                available,
                ..Raw::EMPTY
            });
            let value = read(input).unwrap();
            assert_eq!(value["reads"]["menu_tab"], available != 0);
            assert!(value["menu_tab"].is_null());
            assert!(value["player"].is_null());
            assert!(value["highlighted_recipe_id"].is_null());
        }
        assert!(loader.client_commands().is_empty());
    }
}
