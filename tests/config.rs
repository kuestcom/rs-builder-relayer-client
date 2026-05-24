mod common;

use kuest_builder_relayer_client::get_contract_config;

#[test]
fn get_contract_config_supports_expected_chains() {
    for chain_id in [137_u64, 80002_u64] {
        let config = get_contract_config(chain_id).expect("known chain");
        assert_eq!(
            config
                .deposit_wallet_contracts
                .deposit_wallet_factory
                .to_string(),
            "0x3DaBe8f032833CE42CC26d9149660E6f596759C5"
        );
        assert_eq!(
            config
                .deposit_wallet_contracts
                .deposit_wallet_implementation
                .to_string(),
            "0xFB2f5D822Ecb062dE63a7B830C5e83C994698851"
        );
    }
}

#[test]
fn get_contract_config_rejects_unknown_chain() {
    let error = get_contract_config(1).expect_err("chain should be rejected");
    assert_eq!(error.to_string(), "Invalid chainID: 1");
}
