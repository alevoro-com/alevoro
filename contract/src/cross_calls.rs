use near_sdk::json_types::ValidAccountId;
use near_sdk::{ext_contract, Promise, AccountId, PromiseOrValue, env, StorageUsage};
use near_contract_standards::non_fungible_token::{TokenId, refund_deposit};
use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApprovalReceiver;
use crate::{Contract, CONTRACT_NAME};
use crate::locked_token::{LockedToken, LockedTokenState};
use std::cmp::max;
use std::convert::TryFrom;

#[ext_contract(marketplace)]
pub trait TokenTransfer {
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<String>,
        memo: Option<String>
    );
}