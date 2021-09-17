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

    pub users_val: HashMap<AccountId, i8>,

    pub tokens_stored_per_owner: UnorderedMap<AccountId, UnorderedSet<LockedToken>>,

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
    LockerByTokenId
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTMetadata) -> Self {
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
                Some(&metadata),
            ),
            users_val: HashMap::new(),
            tokens_stored_per_owner: UnorderedMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap()),
        };

        this.measure_min_token_storage_cost();

        this
    }

    pub fn get_num(&self, account_id: AccountId) -> i8 {
        let val = self.users_val.get(&account_id).cloned();
        if val.is_none() {
            return 0;
        }
        return val.unwrap();
    }

    pub fn get_all_locked_tokens(
        &self
    ) -> Vec<JsonLockedToken> {
        let mut all_locked_tokens = vec![];
        for account_id in self.tokens_stored_per_owner.keys_as_vector().iter() {
            let locked_tokens = self.get_locked_instances(account_id, false);
            for locked_token in locked_tokens.iter() {
                let json_token = self.nft_token(locked_token.token_id.clone()).unwrap().clone();
                all_locked_tokens.push(JsonLockedToken {
                    json_token: json_token,
                    locked_token: locked_token.clone()
                })
            }
        }
        all_locked_tokens
    }

    pub fn get_locked_tokens(
        &self,
        account_id: AccountId,
        need_all: bool
    ) -> Vec<JsonToken> {
        let mut locked_tokens_jsons = vec![];
        let locked_tokens = self.get_locked_instances(account_id, need_all);
        for locked_token in locked_tokens.iter() {
            locked_tokens_jsons.push(self.nft_token(locked_token.token_id.clone()).unwrap());
        }
        locked_tokens_jsons
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


    pub fn increment(&mut self, account_id: AccountId) {
        let val = self.users_val.get(&account_id).cloned();
        let mut new_val: i8 = 0;
        if val.is_some() {
            new_val = val.unwrap();
        }
        new_val += 1;
        self.users_val.insert(account_id, new_val);
        let log_message = format!("Increased number to {}", new_val);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn transfer_nft_to_contract(&mut self, token_id: TokenId, borrowed_money: String, apr: u64, borrow_duration: u64) {
        let account_id = &env::predecessor_account_id();
        let initial_storage_usage = env::storage_usage() as i128;
        let token_id_cloned = token_id.clone();

        self.nft_transfer(ValidAccountId::try_from("contract.ze.testnet".to_string()).unwrap(), token_id, None, None);

        let mut locked_tokens = self.tokens_stored_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::NFTsPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });
        locked_tokens.insert(&LockedToken {
            token_id: token_id_cloned.clone(),
            owner_id: account_id.clone(),
            duration: borrow_duration,
            borrowed_money: borrowed_money,
            apr: apr,
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
        let account_id = &env::predecessor_account_id();
        let initial_storage_usage = env::storage_usage() as i128;
        // let token_id_cloned = token_id.clone();

        let mut locked_tokens = self.tokens_stored_per_owner
            .get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::NFTsPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });

        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert!(token.owner_id.to_string() == account_id.to_string());
            assert!(!token.is_confirmed);
            assert!(locked_tokens.remove(&token));

            self.tokens_stored_per_owner.insert(account_id, &locked_tokens);
            self.nft_locker_by_token_id.remove(&token_id);
            self.internal_transfer(&"contract.ze.testnet".to_string(), account_id, &token_id, None, None);

            let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

            let required_storage_in_bytes =
                ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

            env::log(format!("Was {}. Now: {}. Required: {}.", initial_storage_usage, env::storage_usage(), required_storage_in_bytes).as_bytes());


            refund_deposit(required_storage_in_bytes);
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

        let mut contract_locked_tokens = self.tokens_stored_per_owner
            .get(&seller_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::NFTsPerOwnerInner {
                    account_id_hash: hash_account_id(&lender_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });

        env::log("GET LOCKED TOKENS".as_bytes());

        let token_exists_and_valid = contract_locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        env::log("CHECKED VALIDITY".as_bytes());

        for x in contract_locked_tokens.iter() {
            env::log(format!("Token_id: {}", x.token_id).as_bytes());
        }

        if let Some(token) = token_exists_and_valid {
            let deposit = env::attached_deposit();
            env::log(format!("Is confirmed: {}", token.is_confirmed).as_bytes());

            if !token.is_confirmed {
                let borrowed_money = u128::from_str(&token.borrowed_money).expect("Failed to parse borrowed amount");
                assert_eq!(deposit, borrowed_money);
                let mut change_confirm_token = token.clone();
                change_confirm_token.is_confirmed = true;

                assert!(contract_locked_tokens.remove(&token));
                contract_locked_tokens.insert(&change_confirm_token);

                self.tokens_stored_per_owner.insert(&seller_id, &contract_locked_tokens);

                Promise::new(seller_id).transfer(deposit);
            } else {
                env::panic("Token has already been bought or owner canceled the order.".as_bytes())
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


        let tokens_per_owner_entry_in_bytes = env::storage_usage() - initial_storage_usage;
        let owner_id_extra_cost_in_bytes = (tmp_account_id.len() - self.owner_id.len()) as u64;

        self.extra_storage_in_bytes_per_token =
            tokens_per_owner_entry_in_bytes + owner_id_extra_cost_in_bytes;

        self.tokens_per_owner.remove(&tmp_account_id);
        self.tokens_stored_per_owner.remove(&tmp_account_id);
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
            owner_id: "contract.ze.testnet".to_string(),
            extra_storage_in_bytes_per_token: 0,
            metadata: LazyOption::new(StorageKey::NftMetadata.try_to_vec().unwrap(), None),
            users_val: HashMap::new(),
            tokens_stored_per_owner: UnorderedMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap())
        }
    }
}

