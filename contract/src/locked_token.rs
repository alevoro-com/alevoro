use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockedToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub duration: u64,
    pub lend_money: u128,
    pub apr: u64,
    pub creditor: Optional<AccountId>,
    pub start_time: Optional<u64>,
    pub is_confirmed: bool
}