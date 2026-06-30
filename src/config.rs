use alloy::primitives::Address;

use crate::error::{Error, Result};

const DEPOSIT_WALLET_FACTORY: &str = "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c";
const DEPOSIT_WALLET_BEACON: &str = "0x74a618eBdd62Ff8579A8FE94f5B888d7623b9C35";
const DEPOSIT_WALLET_IMPLEMENTATION: &str = "0xf9dFAe108bF7d7aaa9E6D8c1aB281c6285BAF86c";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositWalletContractConfig {
    pub deposit_wallet_factory: Address,
    pub deposit_wallet_beacon: Address,
    pub deposit_wallet_implementation: Address,
}

impl DepositWalletContractConfig {
    #[must_use]
    pub fn factory_string(&self) -> String {
        self.deposit_wallet_factory.to_string()
    }

    #[must_use]
    pub fn implementation_string(&self) -> String {
        self.deposit_wallet_implementation.to_string()
    }

    #[must_use]
    pub fn beacon_string(&self) -> String {
        self.deposit_wallet_beacon.to_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractConfig {
    pub deposit_wallet_contracts: DepositWalletContractConfig,
}

#[must_use]
pub fn is_deposit_wallet_contract_config_valid(config: &DepositWalletContractConfig) -> bool {
    config.deposit_wallet_factory != Address::ZERO && config.deposit_wallet_beacon != Address::ZERO
}

pub fn get_contract_config(chain_id: u64) -> Result<ContractConfig> {
    let config = match chain_id {
        137 | 80002 => ContractConfig {
            deposit_wallet_contracts: DepositWalletContractConfig {
                deposit_wallet_factory: Address::parse_checksummed(DEPOSIT_WALLET_FACTORY, None)
                    .map_err(|_| Error::InvalidAddress {
                        field: "DepositWalletFactory",
                        value: DEPOSIT_WALLET_FACTORY.to_owned(),
                    })?,
                deposit_wallet_beacon: Address::parse_checksummed(DEPOSIT_WALLET_BEACON, None)
                    .map_err(|_| Error::InvalidAddress {
                        field: "DepositWalletBeacon",
                        value: DEPOSIT_WALLET_BEACON.to_owned(),
                    })?,
                deposit_wallet_implementation: Address::parse_checksummed(
                    DEPOSIT_WALLET_IMPLEMENTATION,
                    None,
                )
                .map_err(|_| Error::InvalidAddress {
                    field: "DepositWalletImplementation",
                    value: DEPOSIT_WALLET_IMPLEMENTATION.to_owned(),
                })?,
            },
        },
        _ => return Err(Error::InvalidChainId(chain_id)),
    };

    if !is_deposit_wallet_contract_config_valid(&config.deposit_wallet_contracts) {
        return Err(Error::UnsupportedContractConfig);
    }

    Ok(config)
}
