use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LockedToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub duration: u64,
    pub borrowed_money: u128,
    pub apr: u64,
    pub creditor: Option<AccountId>,
    pub start_time: Option<u64>,
    pub is_confirmed: bool
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonLockedToken {
    pub json_token: JsonToken,
    pub locked_token: LockedToken,
}