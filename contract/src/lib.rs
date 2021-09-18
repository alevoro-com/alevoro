use std::collections::HashMap;
use std::cmp::{min, max};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U64, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, StorageUsage,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::token::*;
pub use crate::enumerable::*;
use std::convert::TryFrom;
use std::str::FromStr;
use crate::locked_token::*;
use std::time::Duration;

mod internal;
mod metadata;
mod mint;
mod nft_core;
mod token;
mod enumerable;
mod locked_token;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    pub tokens_by_id: LookupMap<TokenId, Token>,

    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    pub owner_id: AccountId,

    /// The storage size in bytes for one account.
    pub extra_storage_in_bytes_per_token: StorageUsage,

    pub metadata: LazyOption<NFTMetadata>,

    pub tokens_stored_per_owner: UnorderedMap<AccountId, UnorderedSet<LockedToken>>,

    pub credit_tokens_per_creditor: UnorderedMap<AccountId, UnorderedSet<LockedToken>>,

    pub nft_locker_by_token_id: LookupMap<TokenId, AccountId>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NftMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    NFTsPerOwner,
    NFTsPerOwnerInner { account_id_hash: CryptoHash },
    CreditNFTsPerOwner,
    CreditNFTsPerOwnerInner { account_id_hash: CryptoHash },
    LockerByTokenId,
}

const CONTRACT_NAME: &str = "contract.pep.testnet";

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId) -> Self {
        let mut this = Self {
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            owner_id: owner_id.into(),
            extra_storage_in_bytes_per_token: 0,
            metadata: LazyOption::new(
                StorageKey::NftMetadata.try_to_vec().unwrap(),
                None,
            ),
            tokens_stored_per_owner: UnorderedMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap()),
            credit_tokens_per_creditor: UnorderedMap::new(StorageKey::CreditNFTsPerOwner.try_to_vec().unwrap()),
        };

        this.measure_min_token_storage_cost();

        this
    }

    pub fn get_all_locked_tokens(
        &self
    ) -> Vec<JsonLockedToken> {
        let mut all_locked_tokens = vec![];
        for account_id in self.tokens_stored_per_owner.keys_as_vector().iter() {
            all_locked_tokens.append(&mut self.get_locked_tokens(account_id, false))
        }
        all_locked_tokens
    }

    pub fn get_locked_tokens(
        &self,
        account_id: AccountId,
        need_all: bool
    ) -> Vec<JsonLockedToken> {
        let mut locked_tokens_jsons = vec![];
        let locked_tokens = self.get_locked_instances(account_id, need_all);
        for locked_token in locked_tokens.iter() {
            let json_token = self.nft_token(locked_token.token_id.clone()).unwrap().clone();
            locked_tokens_jsons.push(JsonLockedToken {
                json_token: json_token,
                locked_token: locked_token.clone()
            });
        }
        locked_tokens_jsons
    }

    pub fn get_debtors_tokens(
        &self,
        account_id: AccountId
    ) -> Vec<JsonLockedToken> {
        //let credit_tokens = self.get_tokens_for_borrowed_money(&&account_id);

        let mut result = vec![];
//        for locked_token in credit_tokens.iter() {
//            let token_id = locked_token.token_id.clone();
//            result.push(
//                JsonLockedToken {
//                    json_token: self.nft_token(token_id).unwrap(),
//                    locked_token: locked_token.clone(),
//                }
//            )
        //       }

        result
    }

    #[private]
    fn get_locked_instances(
        &self,
        account_id: AccountId,
        need_all: bool
    )-> Vec<LockedToken> {
        let mut tmp = vec![];
        let tokens_owner = self.tokens_stored_per_owner.get(&account_id);
        let tokens = if let Some(tokens_owner) = tokens_owner {
            tokens_owner
        } else {
            return vec![];
        };
        let keys = tokens.as_vector();
        let start = 0;
        let end = keys.len();
        for i in start..end {
            let cur_token: LockedToken = keys.get(i).unwrap();
            if need_all || !cur_token.is_confirmed {
                tmp.push(cur_token);
            }
        }
        tmp
    }

    #[payable]
    pub fn transfer_nft_to_contract(&mut self, token_id: TokenId, borrowed_money: String, apr: u64, borrow_duration: u64) {
        let account_id = &env::predecessor_account_id();
        let initial_storage_usage = env::storage_usage() as i128;
        let token_id_cloned = token_id.clone();

        self.nft_transfer(ValidAccountId::try_from(CONTRACT_NAME.to_string()).unwrap(), token_id, None, None);

        let mut locked_tokens = self.get_tokens_stored_per_owner(&account_id);
        locked_tokens.insert(&LockedToken {
            token_id: token_id_cloned.clone(),
            owner_id: account_id.clone(),
            duration: borrow_duration,
            borrowed_money,
            apr,
            creditor: None,
            start_time: None,
            is_confirmed: false,
        });
        self.tokens_stored_per_owner.insert(account_id, &locked_tokens);
        self.nft_locker_by_token_id.insert(&token_id_cloned, account_id);

        let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

        env::log(format!("Was {}. Now - was: {}.", initial_storage_usage, market_lock_size_in_bytes).as_bytes());

        let required_storage_in_bytes =
            ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

        env::log(format!("Extra storage now: {}.", self.extra_storage_in_bytes_per_token).as_bytes());
        env::log(format!("Required now: {}.", required_storage_in_bytes).as_bytes());

        refund_deposit(required_storage_in_bytes);

        env::log(format!("ADDED TOKEN ID: {}, SELLER: {}", token_id_cloned, self.nft_locker_by_token_id.get(&token_id_cloned).unwrap()).as_bytes())
    }

    #[payable]
    pub fn transfer_nft_back(&mut self, token_id: TokenId) {
        let owner_id = &env::predecessor_account_id();
        self.transfer_nft_from_contract_to_return_owner(&owner_id, &owner_id, token_id, false, false);
    }

    #[payable]
    fn transfer_nft_from_contract_to_return_owner(&mut self, init_owner: &&AccountId, return_owner: &&AccountId, token_id: TokenId, is_repaid: bool, is_delayed: bool) {
        let initial_storage_usage = env::storage_usage() as i128;

        let mut locked_tokens = self.get_tokens_stored_per_owner(init_owner);
        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert_eq!(token.owner_id.to_string(), init_owner.to_string());
            assert!(!token.is_confirmed || is_repaid || is_delayed);
            assert!(locked_tokens.remove(&token));

            self.tokens_stored_per_owner.insert(init_owner, &locked_tokens);
            self.nft_locker_by_token_id.remove(&token_id);
            self.internal_transfer(&CONTRACT_NAME.to_string(), return_owner, &token_id, None, None);

            if is_repaid || is_delayed {
                let creditor = token.creditor.unwrap();
                let mut creditor_tokens_saved = self.get_tokens_for_borrowed_money(&&creditor);

                let rm_token = creditor_tokens_saved
                    .iter()
                    .find(|x| x.token_id == token_id).unwrap();

                assert!(creditor_tokens_saved.remove(&rm_token));

                self.credit_tokens_per_creditor.insert(&creditor, &creditor_tokens_saved);
            }

            let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

            let required_storage_in_bytes =
                ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

            env::log(format!("Was {}. Now: {}. Required: {}.", initial_storage_usage, env::storage_usage(), required_storage_in_bytes).as_bytes());


            if !is_repaid.clone() && ! is_delayed.clone() {
                refund_deposit(required_storage_in_bytes);
            }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }

    #[payable]
    pub fn transfer_deposit_for_nft(&mut self, token_id: TokenId) {
        let lender_id = &env::predecessor_account_id();
        let seller_id = self.nft_locker_by_token_id.get(&token_id).unwrap();
        assert_ne!(lender_id.to_string(), seller_id.to_string());
        env::log(format!("Seller: {}", seller_id).as_bytes());
        env::log(format!("TokeID: {}", token_id).as_bytes());

        let mut contract_locked_tokens = self.get_tokens_stored_per_owner(&&seller_id);

        let token_exists_and_valid = contract_locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            let deposit = env::attached_deposit();
            env::log(format!("Is confirmed: {}", token.is_confirmed).as_bytes());

            if !token.is_confirmed {
                let borrowed_money = u128::from_str(&token.borrowed_money).expect("Failed to parse borrowed amount");
                assert_eq!(deposit, borrowed_money);
                let mut change_confirm_token = token.clone();
                change_confirm_token.is_confirmed = true;
                change_confirm_token.creditor = Some(AccountId::from(lender_id));
                change_confirm_token.start_time = Some(env::block_timestamp());

                assert!(contract_locked_tokens.remove(&token));
                contract_locked_tokens.insert(&change_confirm_token);

                self.tokens_stored_per_owner.insert(&seller_id, &contract_locked_tokens);

                let mut tokens_for_borrowed_money = self.get_tokens_for_borrowed_money(&lender_id);

                tokens_for_borrowed_money.insert(&change_confirm_token);

                self.credit_tokens_per_creditor.insert(lender_id, &tokens_for_borrowed_money);

                Promise::new(seller_id).transfer(deposit);
            } else {
                env::panic("Token has already been bought or owner canceled the order.".as_bytes())
            }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }

    fn get_tokens_for_borrowed_money(&self, lender_id: &&String) -> UnorderedSet<LockedToken> {
        let tokens_for_borrowed_money = self.credit_tokens_per_creditor
            .get(&lender_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::CreditNFTsPerOwnerInner {
                    account_id_hash: hash_account_id(&lender_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });
        tokens_for_borrowed_money
    }

    fn get_tokens_stored_per_owner(&self, account_id: &&String) -> UnorderedSet<LockedToken> {
        let locked_tokens = self.tokens_stored_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::NFTsPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });
        locked_tokens
    }

    #[payable]
    pub fn repaid_loan(&mut self, token_id: TokenId) {
        let deposit = env::attached_deposit();
        let owner_id = &env::predecessor_account_id();

        let contract_locked_tokens = self.get_tokens_stored_per_owner(&owner_id);

        env::log("GET LOCKED TOKENS".as_bytes());

        let token_exists_and_valid = contract_locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        env::log("CHECKED VALIDITY".as_bytes());

        if let Some(token) = token_exists_and_valid {
            env::log(format!("Is confirmed: {}", token.is_confirmed).as_bytes());

            if token.is_confirmed {
                let mut borrowed_money = u128::from_str(&token.borrowed_money).expect("Failed to parse borrowed amount");
                borrowed_money += borrowed_money * u128::from(token.apr) / 100;
                assert_eq!(deposit, borrowed_money);
                self.transfer_nft_from_contract_to_return_owner(&owner_id, &owner_id, token_id, true, false);
                if let Some(creditor) = token.creditor {
                    Promise::new(creditor).transfer(deposit);
                } else {
                    env::panic("Creditor does not exist".as_bytes())
                }
            } else {
                env::panic("Token isn't locked".as_bytes())
            }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }

    #[payable]
    pub fn check_transfer_overdue_nft_to_creditor(&mut self, token_id: TokenId) {
        let creditor_id = &env::predecessor_account_id();
        let locked_tokens = self.get_tokens_for_borrowed_money(&creditor_id);

        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert_eq!(token.creditor.clone().unwrap().to_string(), creditor_id.to_string());
            assert!(token.is_confirmed);

            let now = Duration::from_nanos(env::block_timestamp());
            let deal_time = Duration::from_nanos(token.start_time.unwrap());

            let diff = now - deal_time;
            let sec_diff = diff.as_secs();

            if sec_diff >= token.duration {
                self.transfer_nft_from_contract_to_return_owner(&&token.owner_id, &&token.creditor.unwrap(), token_id, false, true);
                env::log(format!("Successfully transferred NFT from {} to {} by creditor request.", token.owner_id, env::predecessor_account_id()).as_bytes());
            } else {
                env::panic(format!("It is still {} seconds left for lender to return money.", sec_diff).as_bytes());
            }

        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }


    fn measure_min_token_storage_cost(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = "a".repeat(64);
        let u = UnorderedSet::new(
            StorageKey::TokenPerOwnerInner {
                account_id_hash: hash_account_id(&tmp_account_id),
            }
                .try_to_vec()
                .unwrap(),
        );
        let locked_u = UnorderedSet::new(
            StorageKey::NFTsPerOwnerInner {
                account_id_hash: hash_account_id(&tmp_account_id),
            }
                .try_to_vec()
                .unwrap(),
        );
        self.tokens_per_owner.insert(&tmp_account_id, &u);
        self.tokens_stored_per_owner.insert(&tmp_account_id, &locked_u);
        self.credit_tokens_per_creditor.insert(&tmp_account_id, &locked_u);


        let tokens_per_owner_entry_in_bytes = env::storage_usage() - initial_storage_usage;
        let owner_id_extra_cost_in_bytes = (tmp_account_id.len() - self.owner_id.len()) as u64;

        self.extra_storage_in_bytes_per_token =
            tokens_per_owner_entry_in_bytes + owner_id_extra_cost_in_bytes;

        self.tokens_per_owner.remove(&tmp_account_id);
        self.tokens_stored_per_owner.remove(&tmp_account_id);
        self.credit_tokens_per_creditor.remove(&tmp_account_id);
    }


    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        // #[derive(BorshDeserialize)]
        // struct Old {
        //     pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

        //     pub tokens_by_id: LookupMap<TokenId, Token>,

        //     pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

        //     pub owner_id: AccountId,

        //     /// The storage size in bytes for one account.
        //     pub extra_storage_in_bytes_per_token: StorageUsage,

        //     pub metadata: LazyOption<NFTMetadata>,

        //     pub users_val: HashMap<AccountId, i8>,

        //     pub tokens_stored_per_owner: LookupMap<AccountId, UnorderedSet<LockedToken>>,

        //     pub nft_locker_by_token_id: LookupMap<TokenId, AccountId>
        // }
        // let state_1: Old = env::state_read().expect("Error");

        Self {
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(StorageKey::TokenMetadataById.try_to_vec().unwrap()),
            owner_id: CONTRACT_NAME.to_string(),
            extra_storage_in_bytes_per_token: 0,
            metadata: LazyOption::new(StorageKey::NftMetadata.try_to_vec().unwrap(), None),
            tokens_stored_per_owner: UnorderedMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
            credit_tokens_per_creditor: UnorderedMap::new(StorageKey::CreditNFTsPerOwner.try_to_vec().unwrap()),
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap()),
        }
    }
}

