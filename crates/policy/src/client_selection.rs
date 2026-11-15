//! Bounded evidence from named client UI events, never inferred current state.
use serde_json::{Value, json};
use std::collections::VecDeque;

pub const HISTORY: usize = 64;
pub const CATEGORY_CHOICES: usize = 4;

#[derive(Debug)]
pub enum UiEvent {
    OpenScreen {
        screen: String,
        host: u32,
    },
    StationRecipe {
        operator: u32,
        station: u32,
        recipe: u32,
        choices: Vec<(u32, u8)>,
    },
}

#[derive(Default, Debug)]
pub struct Evidence {
    sequence: u64,
    dropped: u64,
    session_generation: u64,
    cleared_through: u64,
    events: VecDeque<Value>,
}

impl Evidence {
    pub fn end_session(&mut self) {
        self.cleared_through = self.sequence;
        self.events.clear();
        self.session_generation = self.session_generation.saturating_add(1);
    }
    pub fn record(&mut self, event: UiEvent) {
        let payload = match event {
            UiEvent::OpenScreen { screen, host } => {
                json!({"kind":"open_hud_screen_event","requested_screen":super::client::text(&screen),"host":host})
            }
            UiEvent::StationRecipe {
                operator,
                station,
                recipe,
                choices,
            } => {
                json!({"kind":"station_recipe_selection_event","operator":operator,"station":station,"recipe_id":recipe,"category_choices":choices.iter().take(CATEGORY_CHOICES).map(|(id,index)|json!({"category_id":id,"selected_index":index})).collect::<Vec<_>>(),"choices_truncated":choices.len()>CATEGORY_CHOICES})
            }
        };
        self.sequence = self.sequence.saturating_add(1);
        if self.events.len() == HISTORY {
            self.events.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.events
            .push_back(json!({"sequence":self.sequence,"scope":"client_world","event":payload}));
    }

    pub fn report(&self, after: u64, limit: usize) -> Value {
        let events: Vec<_> = self
            .events
            .iter()
            .filter(|event| event["sequence"].as_u64().is_some_and(|n| n > after))
            .take(limit.min(32))
            .cloned()
            .collect();
        let next = events
            .last()
            .and_then(|event| event["sequence"].as_u64())
            .unwrap_or(after);
        let first = self
            .events
            .front()
            .and_then(|event| event["sequence"].as_u64());
        json!({"events":events,"next_after":next,"latest_sequence":self.sequence,"session_generation":self.session_generation,"dropped_history":self.dropped,"cursor_gap":after<self.cleared_through || first.is_some_and(|n|after<n.saturating_sub(1)),"loader_drop_counts":null,"loader_drop_status":"not exported; loader logs a bounded copy/scan warning","current_selected_tab":null,"current_selected_recipe":null,"state_status":"no verified selected-tab or passive-recipe state getter","evidence_scope":"observed client events, not confirmation that UI or production applied the request; not current selection","identity_scope":"client event observation only; cleared on observed session exit; never use these ids as server-world ids"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_events_do_not_become_current_selection_or_server_authority() {
        let mut evidence = Evidence::default();
        evidence.record(UiEvent::OpenScreen {
            screen: "PlayerCrafting".into(),
            host: 4,
        });
        evidence.record(UiEvent::StationRecipe {
            operator: 3,
            station: 4,
            recipe: 55,
            choices: vec![(9, 1)],
        });
        let report = evidence.report(0, 32);
        assert!(report["current_selected_tab"].is_null());
        assert!(report["current_selected_recipe"].is_null());
        assert_eq!(report["events"][0]["scope"], "client_world");
        assert_eq!(report["events"][1]["event"]["recipe_id"], 55);
        assert_eq!(report["next_after"], 2);
    }
    #[test]
    fn history_choices_and_query_output_are_capped_with_gap_evidence() {
        let mut evidence = Evidence::default();
        for _ in 0..70 {
            evidence.record(UiEvent::StationRecipe {
                operator: 1,
                station: 2,
                recipe: 3,
                choices: vec![(1, 0); 8],
            });
        }
        let report = evidence.report(0, 100);
        assert_eq!(report["events"].as_array().unwrap().len(), 32);
        assert_eq!(report["dropped_history"], 6);
        assert_eq!(report["cursor_gap"], true);
        assert_eq!(
            report["events"][0]["event"]["category_choices"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(report["events"][0]["event"]["choices_truncated"], true);
    }
    #[test]
    fn leaving_game_clears_old_client_entity_ids_without_reusing_cursors() {
        let mut evidence = Evidence::default();
        evidence.record(UiEvent::OpenScreen {
            screen: "Backpack".into(),
            host: 77,
        });
        evidence.end_session();
        let report = evidence.report(0, 32);
        assert_eq!(report["events"], json!([]));
        assert_eq!(report["session_generation"], 1);
        assert_eq!(report["latest_sequence"], 1);
        evidence.record(UiEvent::OpenScreen {
            screen: "PlayerCrafting".into(),
            host: 77,
        });
        assert_eq!(evidence.report(1, 32)["events"][0]["sequence"], 2);
    }

    #[test]
    fn session_clear_reports_unread_history_loss_even_before_new_events() {
        let mut evidence = Evidence::default();
        evidence.end_session();
        assert_eq!(evidence.report(0, 32)["cursor_gap"], false);
        evidence.record(UiEvent::OpenScreen {
            screen: "Backpack".into(),
            host: 77,
        });
        evidence.end_session();
        assert_eq!(evidence.report(0, 32)["cursor_gap"], true);
        assert_eq!(evidence.report(1, 32)["cursor_gap"], false);
        evidence.end_session();
        assert_eq!(evidence.report(0, 32)["cursor_gap"], true);
        assert_eq!(evidence.report(1, 32)["cursor_gap"], false);
        evidence.record(UiEvent::OpenScreen {
            screen: "PlayerCrafting".into(),
            host: 77,
        });
        assert_eq!(evidence.report(0, 32)["cursor_gap"], true);
        assert_eq!(evidence.report(1, 32)["cursor_gap"], false);
        assert_eq!(evidence.report(1, 32)["events"][0]["sequence"], 2);
    }
}
