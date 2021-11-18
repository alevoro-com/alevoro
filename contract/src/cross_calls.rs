use near_sdk::json_types::ValidAccountId;
use near_sdk::{ext_contract};
use near_contract_standards::non_fungible_token::{TokenId};

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