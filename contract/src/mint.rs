use crate::*;

#[near_bindgen]
impl Contract {

    /// only the contract owner can mint NFTs
    
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: Option<TokenId>,
        metadata: TokenMetadata,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
        receiver_id: Option<ValidAccountId>,
    ) {
        //self.assert_owner();

        let mut final_token_id = format!("{}", self.token_metadata_by_id.len() + 1);
        if let Some(token_id) = token_id {
            final_token_id = token_id
        }

        let initial_storage_usage = env::storage_usage();
        let mut owner_id = env::predecessor_account_id();
        if let Some(receiver_id) = receiver_id {
            owner_id = receiver_id.into();
        }

        // CUSTOM - create royalty map
        let mut royalty = HashMap::new();
        // user added perpetual_royalties (percentage paid with every transfer)
        if let Some(perpetual_royalties) = perpetual_royalties {
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        let token = Token {
            owner_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
        };
        assert!(
            self.tokens_by_id.insert(&final_token_id, &token).is_none(),
            "Token already exists"
        );
        self.token_metadata_by_id.insert(&final_token_id, &metadata);
        self.internal_add_token_to_owner(&token.owner_id, &final_token_id);

        let new_token_size_in_bytes = env::storage_usage() - initial_storage_usage;
        let required_storage_in_bytes =
            self.extra_storage_in_bytes_per_token + new_token_size_in_bytes;

        refund_deposit(required_storage_in_bytes);
    }
}