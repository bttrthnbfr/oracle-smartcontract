use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use near_sdk::json_types::{U64};
use near_contract_standards::non_fungible_token::{Token};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Counter {
    // val: i8, // i8 is signed. unsigned integers are also available: u8, u16, u32, u64, u128
}

#[near_bindgen]
impl Counter {
    pub fn nft_tokens(&self, from_index: Option<String>, limit: Option<U64>) -> Vec<Token>{
        let res: Vec<Token> = Vec::new();
        res
    }

    pub fn nft_previous_owner(&self, nft_contract_id: String, token_id: String) -> &str{
        let owner = "test previous owner";
        owner
    }

    pub fn nft_token_for_contract(&self, account_id: String, from_index: Option<String>, limit: Option<U64>) -> Vec<Token>{
        let res: Vec<Token> = Vec::new();
        res
    }

    pub fn nft_tokens_for_owner(&self, account_id: String, from_index: Option<String>, limit: Option<U64>) -> Vec<Token>{
        let res: Vec<Token> = Vec::new();
        res
    }
}