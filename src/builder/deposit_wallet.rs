use alloy::primitives::{Address, Bytes, Signature, U256};
use alloy::signers::Signer as AlloySigner;
use alloy::sol;
use alloy::sol_types::{Eip712Domain, SolStruct as _, eip712_domain};
use std::str::FromStr as _;

use crate::config::DepositWalletContractConfig;
use crate::constants::{DEPOSIT_WALLET_DOMAIN_NAME, DEPOSIT_WALLET_DOMAIN_VERSION};
use crate::error::{Error, Result};
use crate::types::{
    DepositWalletBatchRequest, DepositWalletCall, DepositWalletCreateRequest, DepositWalletParams,
    DepositWalletTransactionArgs, TransactionType,
};

sol! {
    struct Call {
        address target;
        uint256 value;
        bytes data;
    }

    struct Batch {
        address wallet;
        uint256 nonce;
        uint256 deadline;
        Call[] calls;
    }
}

pub async fn build_deposit_wallet_batch_request(
    signer: &(dyn AlloySigner<Signature> + Send + Sync),
    args: &DepositWalletTransactionArgs,
    config: &DepositWalletContractConfig,
) -> Result<DepositWalletBatchRequest> {
    let wallet_address = parse_address("walletAddress", &args.wallet_address)?;
    let domain: Eip712Domain = eip712_domain! {
        name: DEPOSIT_WALLET_DOMAIN_NAME,
        version: DEPOSIT_WALLET_DOMAIN_VERSION,
        chain_id: args.chain_id,
        verifying_contract: wallet_address,
    };
    let batch = Batch {
        wallet: wallet_address,
        nonce: parse_u256("nonce", &args.nonce)?,
        deadline: parse_u256("deadline", &args.deadline)?,
        calls: args
            .calls
            .iter()
            .map(call_to_typed_data)
            .collect::<Result<Vec<_>>>()?,
    };
    let signature = signer
        .sign_hash(&batch.eip712_signing_hash(&domain))
        .await?;

    Ok(DepositWalletBatchRequest {
        tx_type: TransactionType::Wallet,
        from: args.from.clone(),
        to: config.factory_string(),
        nonce: args.nonce.clone(),
        signature: signature.to_string(),
        deposit_wallet_params: DepositWalletParams {
            deposit_wallet: args.wallet_address.clone(),
            deadline: args.deadline.clone(),
            calls: args.calls.clone(),
        },
    })
}

pub fn build_deposit_wallet_create_request(
    from: &str,
    config: &DepositWalletContractConfig,
) -> DepositWalletCreateRequest {
    DepositWalletCreateRequest {
        tx_type: TransactionType::WalletCreate,
        from: from.to_owned(),
        to: config.factory_string(),
    }
}

fn call_to_typed_data(call: &DepositWalletCall) -> Result<Call> {
    Ok(Call {
        target: parse_address("target", &call.target)?,
        value: parse_u256("value", &call.value)?,
        data: parse_bytes("data", &call.data)?,
    })
}

fn parse_address(field: &'static str, value: &str) -> Result<Address> {
    Address::from_str(value).map_err(|_| Error::InvalidAddress {
        field,
        value: value.to_owned(),
    })
}

fn parse_bytes(field: &'static str, value: &str) -> Result<Bytes> {
    Bytes::from_str(value).map_err(|_| Error::InvalidHex {
        field,
        value: value.to_owned(),
    })
}

fn parse_u256(field: &'static str, value: &str) -> Result<U256> {
    U256::from_str(value).map_err(|_| Error::InvalidInteger {
        field,
        value: value.to_owned(),
    })
}
