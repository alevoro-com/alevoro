use std::collections::HashMap;
use std::cmp::min;

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
use crate::locked_token::LockedToken;

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

    pub tokens_stored_per_owner: LookupMap<AccountId, UnorderedSet<LockedToken>>,

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
            tokens_stored_per_owner: LookupMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
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

    pub fn get_locked_tokens(
        &self,
        account_id: AccountId,
    ) -> Vec<JsonToken> {
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
            tmp.push(self.nft_token(keys.get(i).unwrap().token_id).unwrap());
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
    pub fn transfer_nft_to_contract(&mut self, token_id: TokenId, borrowed_money: u128, apr: u64, borrow_duration: u64) {
        let account_id = &env::predecessor_account_id();
        let token_id_cloned = token_id.clone();

        self.nft_transfer(ValidAccountId::try_from("contract.alevoro.testnet".to_string()).unwrap(), token_id, None, None);

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
        env::log(format!("ADDED TOKEN ID: {}, SELLER: {}", token_id_cloned, self.nft_locker_by_token_id.get(&token_id_cloned).unwrap()).as_bytes())
    }

    #[payable]
    pub fn transfer_nft_back(&mut self, token_id: TokenId) {
        let account_id = &env::predecessor_account_id();
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
            //assert!(token.owner_id == account_id);
            assert!(!token.is_confirmed);
            assert!(locked_tokens.remove(&token));

            self.tokens_stored_per_owner.insert(account_id, &locked_tokens);
            self.nft_locker_by_token_id.remove(&token_id);
            //self.internal_add_token_to_owner(account_id, &token_id);
            self.internal_transfer(&"contract.alevoro.testnet".to_string(), account_id, &token_id, None, None);
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

        let mut token_exists_and_valid = contract_locked_tokens
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
                assert_eq!(deposit, token.borrowed_money);
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
        self.tokens_per_owner.insert(&tmp_account_id, &u);

        let tokens_per_owner_entry_in_bytes = env::storage_usage() - initial_storage_usage;
        let owner_id_extra_cost_in_bytes = (tmp_account_id.len() - self.owner_id.len()) as u64;

        self.extra_storage_in_bytes_per_token =
            tokens_per_owner_entry_in_bytes + owner_id_extra_cost_in_bytes;

        self.tokens_per_owner.remove(&tmp_account_id);
    }


    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        #[derive(BorshDeserialize)]
        struct Old {
            pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

            pub tokens_by_id: LookupMap<TokenId, Token>,

            pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

            pub owner_id: AccountId,

            /// The storage size in bytes for one account.
            pub extra_storage_in_bytes_per_token: StorageUsage,

            pub metadata: LazyOption<NFTMetadata>,

            pub users_val: HashMap<AccountId, i8>,

            pub tokens_stored_per_owner: LookupMap<AccountId, UnorderedSet<LockedToken>>,
        }
        let state_1: Old = env::state_read().expect("Error");
//        let metadata = NFTMetadata {
//            spec: "nft-1.0.0".to_string(),              // required, essentially a version like "nft-1.0.0"
//            name: "Mosaics".to_string(),              // required, ex. "Mosaics"
//            symbol: "MOSIAC".to_string(),
//            icon: None,      // Data URL
//            base_uri: None, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
//            reference: None, // URL to a JSON file with more info
//            reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
//        };

        Self {
            tokens_per_owner: state_1.tokens_per_owner,
            tokens_by_id: state_1.tokens_by_id,
            token_metadata_by_id: state_1.token_metadata_by_id,
            owner_id: state_1.owner_id,
            extra_storage_in_bytes_per_token: state_1.extra_storage_in_bytes_per_token,
            metadata: state_1.metadata,
            users_val: state_1.users_val,
            tokens_stored_per_owner: state_1.tokens_stored_per_owner,
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap())
        }
    }
}

