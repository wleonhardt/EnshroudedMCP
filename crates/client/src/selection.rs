//! Published typed events observed in the client's world, not inferred UI state.
use crate::Shared;
use enshwick::{
    Load,
    typed::{
        EventType,
        ecs::{OpenHudScreenEvent, UiSelectCraftingStationRecipeEvent},
    },
};
use policy::client_selection::UiEvent;
use std::sync::Arc;

pub(crate) fn register(load: &mut Load<'_>, shared: &Arc<Shared>) -> [bool; 2] {
    let opened = shared.clone();
    let open = EventType::<OpenHudScreenEvent>::resolve(load).and_then(|ty| {
        ty.on_client(load, move |event| {
            if let Ok(mut evidence) = opened.selection.lock() {
                evidence.record(UiEvent::OpenScreen {
                    screen: format!("{:?}", event.screen),
                    host: event.interaction_host_id.id,
                });
            }
        })
    });
    if let Err(error) = &open {
        enshwick::warn!("client OpenHudScreenEvent unavailable: {error}");
    }
    let selected = shared.clone();
    let recipe = EventType::<UiSelectCraftingStationRecipeEvent>::resolve(load).and_then(|ty| {
        ty.on_client(load, move |event| {
            if let Ok(mut evidence) = selected.selection.lock() {
                evidence.record(UiEvent::StationRecipe {
                    operator: event.crafting_operator_id.id,
                    station: event.crafting_station_id.id,
                    recipe: event.recipe_id.value,
                    choices: event
                        .input_category_selections
                        .selections
                        .iter()
                        .take(policy::client_selection::CATEGORY_CHOICES)
                        .map(|choice| (choice.id.value, choice.selected_index))
                        .collect(),
                });
            }
        })
    });
    if let Err(error) = &recipe {
        enshwick::warn!("client recipe-selection event unavailable: {error}");
    }
    [open.is_ok(), recipe.is_ok()]
}
