//! Production event adapter with published synthetic reflection and SDK delivery.
//! These observations do not prove rendered selection or successful crafting.
#[path = "../src/selection.rs"]
mod adapter;

use enshwick::abi::{Side, Status};
use enshwick::resources::{Constant, Field, Kind, Type, fnv1a32};
use enshwick::{Load, Mod, Off};
use policy::client_selection::Evidence;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Shared {
    selection: Mutex<Evidence>,
}
static SHARED: Mutex<Option<Arc<Shared>>> = Mutex::new(None);
struct Probe;
impl Mod for Probe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        let shared = Arc::new(Shared::default());
        assert_eq!(adapter::register(load, &shared), [true, true]);
        *SHARED.lock().unwrap() = Some(shared);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Probe, "id = \"test.selection-flow\"\nside = \"both\"");

// Synthetic reflection adapted from the pinned SDK client_game_events fixture.
// Copyright (c) 2026 Kyrylo Lytvynov; MIT, see patches/enshwick-LICENSE.
// Never offsets copied from a game binary.
fn types() -> Vec<Type<'static>> {
    let field = Field::new;
    let mut types = vec![
        Type::new("keen::uint8", Kind::Uint8, 1, 1),
        Type::new("keen::uint32", Kind::Uint32, 4, 4),
        Type::new("keen::sint64", Kind::Sint64, 8, 8),
        Type::new("keen::Time", Kind::Struct, 8, 8).with_fields(vec![field(
            "value",
            0,
            "keen::sint64",
        )]),
        Type::new("keen::ecs::Event", Kind::Struct, 1, 1),
        Type::new("keen::ecs::GameEvent", Kind::Struct, 8, 8)
            .with_inner("keen::ecs::Event")
            .with_fields(vec![field("timeStamp", 0, "keen::Time")]),
        Type::new("keen::HudScreenType", Kind::Enum, 1, 1)
            .with_inner("keen::uint8")
            .with_constants(vec![Constant::new("InGameMenu", 4)]),
        Type::new("keen::RecipeInputCategorySelection", Kind::Struct, 8, 4).with_fields(vec![
            field("id", 0, "keen::HashKey32"),
            field("selectedIndex", 4, "keen::uint8"),
        ]),
        Type::new(
            "keen::StaticArray<keen::RecipeInputCategorySelection,4>",
            Kind::StaticArray,
            32,
            4,
        )
        .with_inner("keen::RecipeInputCategorySelection")
        .with_count(4),
        Type::new("keen::RecipeInputCategorySelections", Kind::Struct, 32, 4).with_fields(vec![
            field(
                "selections",
                0,
                "keen::StaticArray<keen::RecipeInputCategorySelection,4>",
            ),
        ]),
        Type::new("keen::ecs::OpenHudScreenEvent", Kind::Struct, 24, 8)
            .with_inner("keen::ecs::GameEvent")
            .with_fields(vec![
                field("playerEntityId", 8, "keen::ecs::EntityId"),
                field("interactionHostId", 12, "keen::ecs::EntityId"),
                field("screen", 16, "keen::HudScreenType"),
            ]),
        Type::new(
            "keen::ecs::UiSelectCraftingStationRecipeEvent",
            Kind::Struct,
            56,
            8,
        )
        .with_inner("keen::ecs::GameEvent")
        .with_fields(vec![
            field("craftingOperatorId", 8, "keen::ecs::EntityId"),
            field("craftingStationId", 12, "keen::ecs::EntityId"),
            field("recipeId", 16, "keen::RecipeId"),
            field(
                "inputCategorySelections",
                20,
                "keen::RecipeInputCategorySelections",
            ),
        ]),
    ];
    for (name, member) in [
        ("keen::ecs::EntityId", "id"),
        ("keen::HashKey32", "value"),
        ("keen::RecipeId", "value"),
    ] {
        types.push(Type::new(name, Kind::Struct, 4, 4).with_fields(vec![field(
            member,
            0,
            "keen::uint32",
        )]));
    }
    types
}

#[test]
fn client_delivery_preserves_category_choices_without_server_or_current_state_authority() {
    let open = "keen::ecs::OpenHudScreenEvent";
    let recipe = "keen::ecs::UiSelectCraftingStationRecipeEvent";
    let hash = |name: &str| fnv1a32(name.as_bytes());
    for side in [Side::CLIENT, Side::SERVER] {
        let loader = enshwick::test::FakeHost::new()
            .side(side)
            .types(types())
            .game_event_type("ecs::OpenHudScreenEvent", hash(open), 24)
            .game_event_type("ecs::UiSelectCraftingStationRecipeEvent", hash(recipe), 56)
            .install();
        // SAFETY: these exports are generated for this test's mod.
        let status =
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) };
        assert_eq!(status, Status::OK, "{:?}", loader.logs());
        let shared = SHARED.lock().unwrap().as_ref().unwrap().clone();
        let mut opened = [0; 24];
        opened[12..16].copy_from_slice(&77u32.to_le_bytes());
        opened[16] = 4;
        let mut selected = [0; 56];
        for (at, value) in [(8, 71u32), (12, 77), (16, u32::MAX)] {
            selected[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        let choices = [(0, 0), (9, 1), (u32::MAX, 255), (9, 2)];
        for (index, (id, choice)) in choices.iter().enumerate() {
            let at = 20 + index * 8;
            selected[at..at + 4].copy_from_slice(&id.to_le_bytes());
            selected[at + 4] = *choice;
        }
        let _ = loader.raise_game_event(hash(open), &opened);
        let _ = loader.raise_game_event(hash(recipe), &selected);
        assert_eq!(
            shared.selection.lock().unwrap().report(0, 32)["events"],
            json!([])
        );
        let _ = loader.raise_client_game_event(hash(open), &opened);
        let _ = loader.raise_client_game_event(hash(recipe), &selected);
        let mut evidence = shared.selection.lock().unwrap();
        let report = evidence.report(0, 32);
        assert!(report["current_selected_tab"].is_null());
        assert!(report["current_selected_recipe"].is_null());
        if side == Side::SERVER {
            assert_eq!(report["events"], json!([]));
            continue;
        }
        assert_eq!(report["events"].as_array().unwrap().len(), 2);
        assert_eq!(report["events"][0]["event"]["host"], 77);
        assert_eq!(
            report["events"][0]["event"]["requested_screen"],
            "InGameMenu"
        );
        let event = &report["events"][1];
        assert_eq!(event["scope"], "client_world");
        assert_eq!(event["event"]["operator"], 71);
        assert_eq!(event["event"]["station"], 77);
        assert_eq!(event["event"]["recipe_id"], u32::MAX);
        assert_eq!(
            event["event"]["category_choices"],
            json!(choices.map(|(id, index)| json!({"category_id":id,"selected_index":index})))
        );
        assert_eq!(event["event"]["choices_truncated"], false);
        evidence.end_session();
        let cleared = evidence.report(0, 32);
        assert_eq!(cleared["events"], json!([]));
        assert_eq!(cleared["cursor_gap"], true);
        assert_eq!(cleared["session_generation"], 1);
    }
}
