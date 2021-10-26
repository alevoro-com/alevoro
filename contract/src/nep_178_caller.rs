use crate::*;
use near_sdk::{ext_contract, PromiseOrValue};
use near_contract_standards::non_fungible_token::Token;

#[ext_contract(marketplace_transferer)]
pub trait NonFungibleTokenMarketplace {
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>
    );

    fn nft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String
    ) -> PromiseOrValue<bool>;

    fn nft_token(self, token_id: TokenId) -> Option<Token>;
}

#[ext_contract(marketplace_approver)]
pub trait NonFungibleTokenApproval {
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: ValidAccountId,
        msg: Option<String>
    ) -> Option<Promise>;

    fn nft_revoke(&mut self, token_id: TokenId, account_id: ValidAccountId);

    fn nft_revoke_all(&mut self, token_id: TokenId);

    fn nft_is_approved(
        self,
        token_id: TokenId,
        approved_account_id: ValidAccountId,
        approval_id: Option<u64>
    ) -> bool;
}