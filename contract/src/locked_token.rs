use crate::*;
use std::fmt::{Display, Formatter};
pub use serde::{Serialize, Deserialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum LockedTokenState {
    Sale,
    Return,
    Locked,
    TransferToCreditor,
    TransferToBorrower
}

impl Display for LockedTokenState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LockedTokenState::Sale => write!(f, "Sale"),
            LockedTokenState::Return => write!(f, "Return"),
            LockedTokenState::Locked => write!(f, "Locked"),
            LockedTokenState::TransferToCreditor => write!(f, "TransferToCreditor"),
            LockedTokenState::TransferToBorrower => write!(f, "TransferToBorrower"),
        }
    }
}

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
    pub extra: String,
    pub market_type: String,
    pub title: String,
    pub media: String,
    pub state: LockedTokenState,
}
