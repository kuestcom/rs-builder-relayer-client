use alloy::primitives::{Address, Bytes, FixedBytes, keccak256};
use alloy::sol;
use alloy::sol_types::SolValue as _;
use std::str::FromStr as _;

use crate::error::{Error, Result};

const ERC1967_BEACON_CONST1: &str = "0x60195155f3363d3d373d3d363d602036600436635c60da";
const ERC1967_BEACON_CONST2: &str =
    "0x1b60e01b36527fa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6c";
const ERC1967_BEACON_CONST3: &str =
    "0xb3582b35133d50545afa5036515af43d6000803e604d573d6000fd5b3d6000f3";
const ERC1967_BEACON_PREFIX: u128 = 0x6100_523D_8160_233D_3973;

sol! {
    struct DepositWalletInitArgs {
        address factory;
        bytes32 walletId;
    }
}

pub fn derive_deposit_wallet(owner: &str, factory: &str, beacon: &str) -> Result<String> {
    let owner = parse_address("owner", owner)?;
    let factory = parse_address("factory", factory)?;
    let beacon = parse_address("beacon", beacon)?;

    let mut wallet_id = [0_u8; 32];
    wallet_id[12..].copy_from_slice(owner.as_slice());
    let args = DepositWalletInitArgs {
        factory,
        walletId: FixedBytes::from(wallet_id),
    }
    .abi_encode();
    let salt = keccak256(&args);
    let bytecode_hash = init_code_hash_erc1967_beacon_proxy(beacon, &args)?;

    Ok(factory.create2(salt, bytecode_hash).to_string())
}

fn init_code_hash_erc1967_beacon_proxy(beacon: Address, args: &[u8]) -> Result<FixedBytes<32>> {
    let args_len = u128::try_from(args.len()).expect("usize fits into u128");
    let combined = ERC1967_BEACON_PREFIX + (args_len << 56);

    let mut init_code = Vec::with_capacity(10 + 20 + 23 + 32 + 32 + args.len());
    init_code.extend_from_slice(&combined.to_be_bytes()[6..]);
    init_code.extend_from_slice(beacon.as_slice());
    init_code.extend_from_slice(
        Bytes::from_str(ERC1967_BEACON_CONST1)
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_BEACON_CONST1",
                value: ERC1967_BEACON_CONST1.to_owned(),
            })?
            .as_ref(),
    );
    init_code.extend_from_slice(
        Bytes::from_str(ERC1967_BEACON_CONST2)
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_BEACON_CONST2",
                value: ERC1967_BEACON_CONST2.to_owned(),
            })?
            .as_ref(),
    );
    init_code.extend_from_slice(
        Bytes::from_str(ERC1967_BEACON_CONST3)
            .map_err(|_| Error::InvalidHex {
                field: "ERC1967_BEACON_CONST3",
                value: ERC1967_BEACON_CONST3.to_owned(),
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
