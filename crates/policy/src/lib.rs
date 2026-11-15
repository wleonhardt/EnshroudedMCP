//! Game-free rules of the MCP endpoint. Nothing here touches the game, so
//! `cargo test` checks every gate on any computer.

pub mod args;
pub mod bounds;
pub mod building_access;
pub mod building_plan;
pub mod building_survey;
pub mod client;
pub mod client_markers;
pub mod client_recipe;
pub mod client_selection;
pub mod clock_control;
pub mod construction;
pub mod health_recovery;
pub mod http;
pub mod inspection;
pub mod interaction_target;
pub mod item_lookup;
pub mod knowledge;
pub mod knowledge_read;
pub mod mcp;
pub mod pacification;
pub mod recipe_plan;
pub mod reposition;
pub mod request;
pub mod write_gate;
