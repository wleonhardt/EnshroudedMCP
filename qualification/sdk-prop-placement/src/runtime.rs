//! Opt-in exact-SDK native qualification; absent from the product workspace.
mod metadata;
mod native;

#[cfg(test)]
#[path = "lib.rs"]
mod consumer_contracts;
