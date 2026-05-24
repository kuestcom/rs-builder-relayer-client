mod common;

use common::{ADDRESS, DERIVED_WALLET};
use kuest_builder_relayer_client::{derive_deposit_wallet, get_contract_config};

#[test]
fn derive_deposit_wallet_matches_current_sdk_vector_on_amoy() {
    let config = get_contract_config(80002).expect("known chain");
    let wallet = derive_deposit_wallet(
        ADDRESS,
        &config.deposit_wallet_contracts.factory_string(),
        &config.deposit_wallet_contracts.implementation_string(),
    )
    .expect("wallet derives");

    assert_eq!(wallet, DERIVED_WALLET);
}

#[test]
fn derive_deposit_wallet_matches_current_sdk_vector_on_polygon() {
    let config = get_contract_config(137).expect("known chain");
    let wallet = derive_deposit_wallet(
        ADDRESS,
        &config.deposit_wallet_contracts.factory_string(),
        &config.deposit_wallet_contracts.implementation_string(),
    )
    .expect("wallet derives");

    assert_eq!(wallet, DERIVED_WALLET);
}
