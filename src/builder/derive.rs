use alloy::primitives::{Address, Bytes, FixedBytes, keccak256};
use alloy::sol;
use alloy::sol_types::SolValue as _;
use std::str::FromStr as _;

use crate::error::{Error, Result};

const ERC1967_CONST1: &str = "0xcc3735a920a3ca505d382bbc545af43d6000803e6038573d6000fd5b3d6000f3";
const ERC1967_CONST2: &str = "0x5155f3363d3d373d3d363d7f360894a13ba1a3210667c828492db98dca3e2076";
const ERC1967_PREFIX: u128 = 0x6100_3D3D_8160_233D_3973;

sol! {
    struct DepositWalletInitArgs {
        address factory;
        bytes32 walletId;
    }
}

pub fn derive_deposit_wallet(owner: &str, factory: &str, implementation: &str) -> Result<String> {
    let owner = parse_address("owner", owner)?;
    let factory = parse_address("factory", factory)?;
    let implementation = parse_address("implementation", implementation)?;

    let mut wallet_id = [0_u8; 32];
    wallet_id[12..].copy_from_slice(owner.as_slice());
    let args = DepositWalletInitArgs {
        factory,
        walletId: FixedBytes::from(wallet_id),
    }
    .abi_encode();
    let salt = keccak256(&args);
    let bytecode_hash = init_code_hash_erc1967(implementation, &args)?;

    Ok(factory.create2(salt, bytecode_hash).to_string())
}

fn init_code_hash_erc1967(implementation: Address, args: &[u8]) -> Result<FixedBytes<32>> {
    let args_len = u128::try_from(args.len()).expect("usize fits into u128");
    let combined = ERC1967_PREFIX + (args_len << 56);

    let mut init_code = Vec::with_capacity(10 + 20 + 2 + 32 + 32 + args.len());
    init_code.extend_from_slice(&combined.to_be_bytes()[6..]);
    init_code.extend_from_slice(implementation.as_slice());
    init_code.extend_from_slice(
        Bytes::from_str("0x6009")
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_6009",
                value: "0x6009".to_owned(),
            })?
            .as_ref(),
    );
    init_code.extend_from_slice(
        Bytes::from_str(ERC1967_CONST2)
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_CONST2",
                value: ERC1967_CONST2.to_owned(),
            })?
            .as_ref(),
    );
    init_code.extend_from_slice(
        Bytes::from_str(ERC1967_CONST1)
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_CONST1",
                value: ERC1967_CONST1.to_owned(),
            })?
            .as_ref(),
    );
    init_code.extend_from_slice(args);

    Ok(keccak256(init_code))
}

fn parse_address(field: &'static str, value: &str) -> Result<Address> {
    Address::from_str(value).map_err(|_| Error::InvalidAddress {
        field,
        value: value.to_owned(),
    })
}
