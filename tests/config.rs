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
            "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c"
        );
        assert_eq!(
            config
                .deposit_wallet_contracts
                .deposit_wallet_beacon
                .to_string(),
            "0x74a618eBdd62Ff8579A8FE94f5B888d7623b9C35"
        );
        assert_eq!(
            config
                .deposit_wallet_contracts
                .deposit_wallet_implementation
                .to_string(),
            "0xf9dFAe108bF7d7aaa9E6D8c1aB281c6285BAF86c"
        );
    }
}

#[test]
fn get_contract_config_rejects_unknown_chain() {
    let error = get_contract_config(1).expect_err("chain should be rejected");
    assert_eq!(error.to_string(), "Invalid chainID: 1");
}
