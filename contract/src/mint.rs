use crate::*;
use std::cmp::max;

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
        // env::log(format!("TOKEN OWNER ID OLD: {}", owner_id).as_bytes());
        if let Some(receiver_id) = receiver_id {
            owner_id = receiver_id.into();
        }

        env::log(format!("1 Usage {}", env::storage_usage()).as_bytes());

        // CUSTOM - create royalty map
        let mut royalty = HashMap::new();
        // user added perpetual_royalties (percentage paid with every transfer)
        if let Some(perpetual_royalties) = perpetual_royalties {
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }
        // env::log(format!("TOKEN OWNER ID: {}", owner_id).as_bytes());
        env::log(format!("2 Usage {}", env::storage_usage()).as_bytes());
        let token = Token {
            owner_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
        };
        env::log(format!("3 Usage {}", env::storage_usage()).as_bytes());
        assert!(
            self.tokens_by_id.insert(&final_token_id, &token).is_none(),
            "Token already exists"
        );
        env::log(format!("4 Usage {}", env::storage_usage()).as_bytes());
        self.token_metadata_by_id.insert(&final_token_id, &metadata);
        env::log(format!("5 Usage {}", env::storage_usage()).as_bytes());
        self.internal_add_token_to_owner(&token.owner_id, &final_token_id);
        env::log(format!("6 Usage {}", env::storage_usage()).as_bytes());

        env::log(format!("Usage {}. Was {}", env::storage_usage(), initial_storage_usage).as_bytes());

        let new_token_size_in_bytes = max(0, env::storage_usage() as i128 - initial_storage_usage as i128) as StorageUsage;
        let required_storage_in_bytes =
            self.extra_storage_in_bytes_per_token + new_token_size_in_bytes;

        env::log(format!("Mint req storage {}", required_storage_in_bytes).as_bytes());

        refund_deposit(required_storage_in_bytes);
    }
}