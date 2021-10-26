use crate::*;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{AccountId, PromiseResult, ext_contract};
use crate::Contract;

#[ext_contract(ext_self_approve_receiver)]
pub trait NonFungibleTokenApprovalReceiver {
    fn nft_on_approve(&mut self,
                      token_id: TokenId,
                      borrowed_money: String,
                      apr: u64, borrow_duration: u64,
                      owner_id: AccountId) -> String;
}

impl Contract {
    fn nft_on_approve(&mut self, token_id: TokenId,
                      borrowed_money: String,
                      apr: u64, borrow_duration: u64,
                      owner_id: AccountId) -> String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Approve callback called"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let approval_id_test = near_sdk::serde_json::from_slice::<u64>(&result).unwrap().to_string();
                env::log(approval_id_test.as_bytes());
                let initial_storage_usage = env::storage_usage() as i128;

                let mut locked_tokens = self
                    .get_tokens_stored_per_owner(&&owner_id);
                locked_tokens.insert(&LockedToken {
                    token_id: token_id.clone(),
                    owner_id: owner_id.clone(),
                    duration: borrow_duration,
                    borrowed_money: borrowed_money,
                    apr: apr,
                    creditor: None,
                    start_time: None,
                    is_confirmed: false,
                    approval_id: 0
                });
                self.tokens_stored_per_owner.insert(&owner_id, &locked_tokens);
                self.nft_locker_by_token_id.insert(&token_id, &owner_id);

                let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

                let required_storage_in_bytes =
                    ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

                refund_deposit(required_storage_in_bytes);

                "Successfully received approve".to_string()
            },
        }
    }
}