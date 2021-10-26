use crate::*;
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LockedToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub duration: u64,
    pub borrowed_money: String,
    pub apr: u64,
    pub creditor: Option<AccountId>,
    pub start_time: Option<u64>,
    pub is_confirmed: bool,
    pub approval_id: u64
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
    pub approved_account_ids: HashMap<AccountId, u64>,

    // // CUSTOM - fields
    // pub royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonLockedToken {
    pub json_token: JsonToken,
    pub locked_token: LockedToken,
}