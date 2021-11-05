mod locked_token;
mod nft_to_json_converter;
mod cross_calls;

use std::cmp::max;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, CryptoHash, PanicOnDefault, Promise, StorageUsage,
};

use near_contract_standards::non_fungible_token::{
    refund_deposit, hash_account_id, TokenId,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

use crate::nft_to_json_converter::*;
use crate::locked_token::*;
use crate::cross_calls::*;

use std::str::FromStr;
use std::time::Duration;
use std::convert::TryFrom;


near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,

    /// The storage size in bytes for one account.
    pub extra_storage_in_bytes_per_token: StorageUsage,

    pub tokens_stored_per_owner: UnorderedMap<AccountId, UnorderedSet<LockedToken>>,

    pub credit_tokens_per_creditor: UnorderedMap<AccountId, UnorderedSet<LockedToken>>,

    pub nft_locker_by_token_id: LookupMap<TokenId, AccountId>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokenTypesLocked,
    NFTsPerOwner,
    NFTsPerOwnerInner { account_id_hash: CryptoHash },
    CreditNFTsPerOwner,
    CreditNFTsPerOwnerInner { account_id_hash: CryptoHash },
    LockerByTokenId,
}

const CONTRACT_NAME: &str = "contract.alevoro.testnet";

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId) -> Self {
        let mut this = Self {
            owner_id: owner_id.into(),
            extra_storage_in_bytes_per_token: 0,
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
        need_all: bool,
    ) -> Vec<JsonLockedToken> {
        let mut locked_tokens_jsons = vec![];
        let locked_tokens = self.get_locked_instances(account_id, need_all);
        for locked_token in locked_tokens.iter() {
            let json_token = self.nft_maybe_to_json(locked_token.token_id.clone()).unwrap().clone();
            locked_tokens_jsons.push(JsonLockedToken {
                json_token: json_token,
                locked_token: locked_token.clone(),
            });
        }
        locked_tokens_jsons
    }

    pub fn get_debtors_tokens(
        &self,
        account_id: AccountId,
    ) -> Vec<JsonLockedToken> {
        let credit_tokens = self.get_tokens_for_lent_money(&&account_id);

        let mut result = vec![];
        for locked_token in credit_tokens.iter() {
            let token_id = locked_token.token_id.clone();
            result.push(
                JsonLockedToken {
                    json_token: self.nft_maybe_to_json(token_id).unwrap(),
                    locked_token: locked_token.clone(),
                }
            )
        }

        result
    }

    #[private]
    fn get_locked_instances(
        &self,
        account_id: AccountId,
        need_all: bool,
    ) -> Vec<LockedToken> {
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
            if need_all || cur_token.state == LockedTokenState::Sale {
                tmp.push(cur_token);
            }
        }
        tmp
    }

    #[payable]
    pub fn nft_on_approve(&mut self, token_id: TokenId, owner_id: AccountId, approval_id: String, msg: String) {
        assert_eq!(env::signer_account_id(), owner_id);
        let initial_storage_usage = env::storage_usage() as i128;

        let params: Vec<&str> = msg.split(",").collect();
        let (borrowed_money,
            apr,
            borrow_duration,
            extra,
            market_type,
            market) = (params[0], params[1], params[2], params[3], params[4], params[5]);

        marketplace::nft_transfer(ValidAccountId::try_from(CONTRACT_NAME).unwrap(),
                                  token_id.to_string(),
                                  Some(approval_id),
                                  None,
                                  &market.clone(),
                                  1,
                                  10_000_000_000_000);

        let mut locked_tokens = self.get_tokens_stored_per_owner(&&owner_id);

        locked_tokens.insert(&LockedToken {
            token_id: token_id.clone().to_string(),
            owner_id: owner_id.clone(),
            duration: borrow_duration.parse::<u64>().unwrap(),
            borrowed_money: borrowed_money.to_string(),
            apr: apr.parse::<u64>().unwrap(),
            creditor: None,
            start_time: None,
            extra: extra.to_string(),
            market_type: market_type.to_string(),
            state: LockedTokenState::Sale,
        });

        self.tokens_stored_per_owner.insert(&owner_id, &locked_tokens);
        self.nft_locker_by_token_id.insert(&token_id.to_string(), &owner_id);

        let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

        let required_storage_in_bytes =
            ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

        refund_deposit(required_storage_in_bytes);
    }

    #[payable]
    pub fn transfer_nft_to_contract(
        &mut self,
        token_id: TokenId,
        borrowed_money: String,
        apr: u64,
        borrow_duration: u64,
        extra: String,
        market_type: String
    ) {
        // let account_id = &env::predecessor_account_id();
        // let initial_storage_usage = env::storage_usage() as i128;
        //
        // let cloned_token = token_id.clone().to_string();
        // let split_id_and_marketplace: Vec<&str> = cloned_token.split(":").collect();
        // let (token_id, marketplace) = (split_id_and_marketplace[0], split_id_and_marketplace[1]);
        //
        // marketplace::nft_transfer(ValidAccountId::try_from(CONTRACT_NAME).unwrap(),
        //                           token_id.to_string(),
        //                           &marketplace,
        //                           1,
        //                           25_000_000_000_000);
        //
        // let mut locked_tokens = self.get_tokens_stored_per_owner(&&account_id);
        //
        // locked_tokens.insert(&LockedToken {
        //     token_id: token_id.clone().to_string(),
        //     owner_id: account_id.clone(),
        //     duration: borrow_duration,
        //     borrowed_money: borrowed_money,
        //     apr,
        //     creditor: None,
        //     start_time: None,
        //     extra: extra,
        //     market_type: market_type,
        //     state: LockedTokenState::Sale,
        // });
        //
        // self.tokens_stored_per_owner.insert(&account_id, &locked_tokens);
        // self.nft_locker_by_token_id.insert(&token_id.to_string(), &account_id);
        //
        // let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);
        //
        // let required_storage_in_bytes =
        //     ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;
        //
        // refund_deposit(required_storage_in_bytes);

        // let cloned_token = token_id.clone().to_string();
        // let split_id_and_marketplace: Vec<&str> = cloned_token.split(":").collect();
        // let (token_id, marketplace) = (split_id_and_marketplace[0], split_id_and_marketplace[1]);
        //
        // marketplace::nft_approve(token_id.to_string(),
        //                          ValidAccountId::try_from(CONTRACT_NAME).unwrap(),
        //                          Some(format!("{},{},{},{},{},{}",
        //                                        borrowed_money, apr,
        //                                        borrow_duration, extra,
        //                                        market_type, marketplace)),
        //                          &marketplace.to_string(),
        //                          env::attached_deposit(),
        //                          200000000000000
        // );
    }

    #[payable]
    pub fn transfer_nft_back(&mut self, token_id: TokenId) {
        let owner_id = &env::predecessor_account_id();
        self.change_status_to_some_returning(&owner_id, &owner_id, token_id, LockedTokenState::Return);
    }

    #[payable]
    fn change_status_to_some_returning(
        &mut self,
        init_owner: &&AccountId,
        return_owner: &&AccountId,
        token_id: TokenId,
        action: LockedTokenState
    ) {
        let initial_storage_usage = env::storage_usage() as i128;

        let mut locked_tokens = self.get_tokens_stored_per_owner(init_owner);
        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert_eq!(token.owner_id.to_string(), init_owner.to_string());

            match action {
                LockedTokenState::Return => {
                    assert_eq!(token.state, LockedTokenState::Sale);
                    assert_eq!(init_owner, return_owner);

                    let mut changed_state_token = token.clone();
                    changed_state_token.state = LockedTokenState::Return;

                    assert!(locked_tokens.remove(&token));

                    locked_tokens.insert(&changed_state_token);

                    self.tokens_stored_per_owner.insert(init_owner, &locked_tokens);
                    // self.nft_locker_by_token_id.remove(&token_id);
                }
                LockedTokenState::TransferToCreditor => {
                    assert_eq!(token.state, LockedTokenState::Locked);
                    assert_eq!(return_owner.to_string(), token.clone().creditor.unwrap());

                    let mut changed_state_token = token.clone();
                    changed_state_token.state = LockedTokenState::TransferToCreditor;

                    assert!(locked_tokens.remove(&token));

                    locked_tokens.insert(&changed_state_token);

                    self.tokens_stored_per_owner.insert(init_owner, &locked_tokens);
                    // self.nft_locker_by_token_id.remove(&token_id);
                }
                LockedTokenState::TransferToBorrower => {
                    assert_eq!(token.state, LockedTokenState::Locked);
                    assert_eq!(return_owner.to_string(), token.owner_id);

                    let mut changed_state_token = token.clone();
                    changed_state_token.state = LockedTokenState::TransferToBorrower;

                    assert!(locked_tokens.remove(&token));

                    locked_tokens.insert(&changed_state_token);

                    self.tokens_stored_per_owner.insert(init_owner, &locked_tokens);
                    // self.nft_locker_by_token_id.remove(&token_id);
                }
                _ => env::panic("Unreachable state!".to_string().as_bytes())
            }

            // TODO()
            // if is_repaid || is_delayed {
            //     let creditor = token.creditor.unwrap();
            //     let mut creditor_tokens_saved = self.get_tokens_for_lent_money(&&creditor);
            //
            //     let rm_token = creditor_tokens_saved
            //         .iter()
            //         .find(|x| x.token_id == token_id).unwrap();
            //
            //     assert!(creditor_tokens_saved.remove(&rm_token));
            //
            //     self.credit_tokens_per_creditor.insert(&creditor, &creditor_tokens_saved);
            // }

            let market_lock_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128);

            let required_storage_in_bytes =
                ((self.extra_storage_in_bytes_per_token as i128) + market_lock_size_in_bytes) as StorageUsage;

            env::log(format!("Was {}. Now: {}. Required: {}.", initial_storage_usage, env::storage_usage(), required_storage_in_bytes).as_bytes());

            // TODO надо ли?
            // if !is_repaid.clone() && !is_delayed.clone() {
            //     refund_deposit(required_storage_in_bytes);
            // }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }

    #[payable]
    pub fn transfer_deposit_for_nft(&mut self, token_id: TokenId) {
        let lender_id = &env::predecessor_account_id();
        let token_owner_id = self.nft_locker_by_token_id.get(&token_id).unwrap();

        assert_ne!(lender_id.to_string(), token_owner_id.to_string());

        env::log(format!("Seller: {}", token_owner_id).as_bytes());
        env::log(format!("TokeID: {}", token_id).as_bytes());

        let mut owner_locked_tokens = self.get_tokens_stored_per_owner(&&token_owner_id);
        let mut tokens_for_lent_money = self.get_tokens_for_lent_money(&&lender_id);

        let token_exists_and_valid = owner_locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            let deposit = env::attached_deposit();
            env::log(format!("State: {}", token.state).as_bytes());

            if token.state == LockedTokenState::Sale {
                let expected_amount_to_lend = u128::from_str(&token.borrowed_money).expect("Failed to parse expected amount to lend.");
                assert_eq!(deposit, expected_amount_to_lend);

                let mut accept_deal_locked_token = token.clone();
                accept_deal_locked_token.state = LockedTokenState::Locked;
                accept_deal_locked_token.creditor = Some(AccountId::from(lender_id));
                accept_deal_locked_token.start_time = Some(env::block_timestamp());

                assert!(owner_locked_tokens.remove(&token));

                owner_locked_tokens.insert(&accept_deal_locked_token);
                self.tokens_stored_per_owner.insert(&token_owner_id, &owner_locked_tokens);


                tokens_for_lent_money.insert(&accept_deal_locked_token);
                self.credit_tokens_per_creditor.insert(lender_id, &tokens_for_lent_money);

                Promise::new(token_owner_id).transfer(deposit);
            } else {
                env::panic("Token has already been bought or owner canceled the order.".as_bytes())
            }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract.", token_id).as_bytes());
        }
    }

    fn get_tokens_for_lent_money(&self, lender_id: &&String) -> UnorderedSet<LockedToken> {
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

        let token_exists_and_valid = contract_locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            env::log(format!("Token state: {}", token.state).as_bytes());
            assert_eq!(&token.owner_id, owner_id);

            if token.state == LockedTokenState::Locked {
                assert!(!self.check_is_token_delayed(token.clone()));

                let mut borrowed_money = u128::from_str(&token.borrowed_money).expect("Failed to parse borrowed amount");
                borrowed_money += borrowed_money * u128::from(token.apr) / 100;

                assert_eq!(deposit, borrowed_money);

                self.change_status_to_some_returning(&owner_id, &owner_id, token_id, LockedTokenState::TransferToBorrower);

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
        let locked_tokens = self.get_tokens_for_lent_money(&&creditor_id.clone());

        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert_eq!(token.creditor.clone().unwrap().to_string(), creditor_id.to_string());
            assert_eq!(token.state, LockedTokenState::Locked);

            if self.check_is_token_delayed(token.clone()) {
                self.change_status_to_some_returning(&&token.owner_id,
                                                     &&token.creditor.unwrap(),
                                                     token_id,
                                                     LockedTokenState::TransferToCreditor);
                env::log(format!("Successfully transferred NFT from {} to {} by creditor request.", token.owner_id, env::predecessor_account_id()).as_bytes());
            } else {
                env::panic(format!("There is still time for borrower to return money.").as_bytes());
            }
        } else {
            env::panic(format!("Can't find token with Id: {} in contract .", token_id).as_bytes());
        }
    }

    #[payable]
    pub fn remove_transferred_token_from_locked_tokens(&mut self, token_id: TokenId) {
        let storage = &env::predecessor_account_id();
        assert_eq!(storage.to_string(), CONTRACT_NAME);

        let init_owner = self.nft_locker_by_token_id.get(&token_id).expect("No such token stored in contract.");

        let mut locked_tokens = self.get_tokens_stored_per_owner(&&init_owner.clone());

        let token_exists_and_valid = locked_tokens
            .iter()
            .find(|x| x.token_id == token_id);

        if let Some(token) = token_exists_and_valid {
            assert!(token.creditor.is_some());

            let mut creditor_lent_money_tokens = self.get_tokens_for_lent_money(&&token.clone().creditor.unwrap());

            assert!(locked_tokens.remove(&token));
            self.tokens_stored_per_owner.insert(&init_owner, &locked_tokens);

            assert!(creditor_lent_money_tokens.remove(&token));
            self.credit_tokens_per_creditor.insert(&token.creditor.unwrap(), &creditor_lent_money_tokens);

            assert!(self.nft_locker_by_token_id.remove(&token_id).is_some());

            env::log(format!("Fully removed token: {} from contract.", token_id).as_bytes());
        } else {
            env::panic(format!("Can't find token with Id: {} in locked tokens of last owner.", token_id).as_bytes());
        }
    }

    fn check_is_token_delayed(&self, token: LockedToken) -> bool {
        let now = Duration::from_nanos(env::block_timestamp());
        let deal_time = Duration::from_nanos(token.start_time.unwrap());

        let diff = now - deal_time;
        let sec_diff = diff.as_secs();

        return sec_diff >= token.duration;
    }


    fn measure_min_token_storage_cost(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = "a".repeat(64);
        let locked_u = UnorderedSet::new(
            StorageKey::NFTsPerOwnerInner {
                account_id_hash: hash_account_id(&tmp_account_id),
            }
                .try_to_vec()
                .unwrap(),
        );
        self.tokens_stored_per_owner.insert(&tmp_account_id, &locked_u);
        self.credit_tokens_per_creditor.insert(&tmp_account_id, &locked_u);


        let tokens_per_owner_entry_in_bytes = env::storage_usage() - initial_storage_usage;
        let owner_id_extra_cost_in_bytes = (tmp_account_id.len() - self.owner_id.len()) as u64;

        self.extra_storage_in_bytes_per_token =
            tokens_per_owner_entry_in_bytes + owner_id_extra_cost_in_bytes;

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

        //     pub users_val: HashMap<AccountId, i8>,

        //     pub tokens_stored_per_owner: LookupMap<AccountId, UnorderedSet<LockedToken>>,

        //     pub nft_locker_by_token_id: LookupMap<TokenId, AccountId>
        // }
        // let state_1: Old = env::state_read().expect("Error");

        Self {
            // tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            // tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            // token_metadata_by_id: UnorderedMap::new(StorageKey::TokenMetadataById.try_to_vec().unwrap()),
            owner_id: CONTRACT_NAME.to_string(),
            extra_storage_in_bytes_per_token: 0,
            tokens_stored_per_owner: UnorderedMap::new(StorageKey::NFTsPerOwner.try_to_vec().unwrap()),
            credit_tokens_per_creditor: UnorderedMap::new(StorageKey::CreditNFTsPerOwner.try_to_vec().unwrap()),
            nft_locker_by_token_id: LookupMap::new(StorageKey::LockerByTokenId.try_to_vec().unwrap()),
        }
    }
}

