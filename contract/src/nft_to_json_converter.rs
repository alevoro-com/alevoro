use crate::*;

pub trait NonFungibleTokenToJsonConverter {
    fn nft_maybe_to_json(&self, token_id: TokenId) -> Option<JsonToken>;
}

impl NonFungibleTokenToJsonConverter for Contract {
    fn nft_maybe_to_json(&self, token_id: TokenId) -> Option<JsonToken> {
        None
    }
}