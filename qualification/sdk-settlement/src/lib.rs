//! Source-only contract probe. No host, game or transaction is opened.
#![cfg(test)]
use enshwick::world::{TransactionId, TransactionStatus, World};
use enshwick::{Entity, WorldError};
use std::time::Duration;

fn observe_settlement(world: World, id: TransactionId) -> Result<(), WorldError> {
    world.when_settled(id, Duration::from_secs(1), |settled| {
        let _: Option<Entity> = settled.standing();
        let _: &Option<TransactionStatus> = &settled.status;
        let _: &[Entity] = &settled.props;
    })
}

#[test]
fn supported_signature_compiles_and_numeric_abi_remains_12_1() {
    let _: fn(World, TransactionId) -> Result<(), WorldError> = observe_settlement;
    assert_eq!(
        (enshwick::abi::ABI_MAJOR, enshwick::abi::ABI_MINOR),
        (12, 1)
    );
}
