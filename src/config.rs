use alloy::primitives::Address;

use crate::error::{Error, Result};

const DEPOSIT_WALLET_FACTORY: &str = "0x3DaBe8f032833CE42CC26d9149660E6f596759C5";
const DEPOSIT_WALLET_IMPLEMENTATION: &str = "0xFB2f5D822Ecb062dE63a7B830C5e83C994698851";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositWalletContractConfig {
    pub deposit_wallet_factory: Address,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractConfig {
    pub deposit_wallet_contracts: DepositWalletContractConfig,
}

#[must_use]
pub fn is_deposit_wallet_contract_config_valid(config: &DepositWalletContractConfig) -> bool {
    config.deposit_wallet_factory != Address::ZERO
        && config.deposit_wallet_implementation != Address::ZERO
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
