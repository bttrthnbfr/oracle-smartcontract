use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, LookupMap};
use near_sdk::json_types::{U128};
use near_sdk::{near_bindgen, PanicOnDefault, AccountId, BorshStorageKey, env};
use near_contract_standards::non_fungible_token::{TokenId, Token};

near_sdk::setup_alloc!();

// copy Token struct from standart library to implement BorshDeserialize & BorshSerialize traits
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenInput {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Oracle {
    pub token_by_id_map: UnorderedMap<String, TokenInput>,
    pub previous_owner_of_token_map: LookupMap<String, String>,
    pub tokens_by_contract_map: LookupMap<String, UnorderedSet<String>>,
    pub tokens_by_owner_map: LookupMap<String, UnorderedSet<String>>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys{
    TokenByIDMap,
    PreviousOwnerOfTokenMap,
    TokensByContractMap,
    TokensByContractMapSet { contract_hash: Vec<u8>  },
    TokensByOwnerMap,
    TokensByOwnerMapSet { account_hash: Vec<u8>  }
}

pub trait OracleInterface {
    // Get a list of all tokens
    //
    // Arguments:
    // * `from_index`: a string representing an unsigned 128-bit integer,
    // representing the starting index of tokens to return
    // * `limit`: the maximum number of tokens to return
    //
    // Returns an array of Token objects, as described in Core standard, and
    // an empty array if there are no tokens
    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>;

    // Get the previous owner of an NFT
    //
    // Arguments:
    // * `nft_contract_id`: a valid NEAR account (NFT smart contract)
    // * `token_id`: a string representing the token_id
    //
    // Returns a valid NEAR account that owned the token before, return null
    // otherwise
    fn nft_previous_owner(&self, nft_contract_id: AccountId, token_id: TokenId) -> Option<String>;

    // Get list of all tokens by a given smart contract account
    //
    // Arguments:
    // * `account_id`: a valid NEAR account (an NFT smart contract)
    // * `from_index`: a string representing an unsigned 128-bit integer,
    // representing the starting index of tokens to return
    // * `limit`: the maximum number of tokens to return
    //
    // Returns a paginated list of all tokens from the NFT smart contract,
    // and an empty array if there are no tokens
    fn nft_token_for_contract(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>;

    // Get list of all tokens owned from multiple contracts by a given
    // account
    //
    // Arguments:
    // * `account_id`: a valid NEAR account
    // * `from_index`: a string representing an unsigned 128-bit integer,
    // representing the starting index of tokens to return
    // * `limit`: the maximum number of tokens to return
    //
    // Returns a paginated list of all tokens owned by this account, and an
    // empty array if there are no tokens
    fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>;

    // Consume token from indexer
    //
    // Arguments:
    // * `nft_contract_id`: a valid NEAR account
    // * `tokens`: array of object token, limited by gas fee 
    //
    //  Return empty if succeded
    fn consume_tokens(&mut self, nft_contract_id: AccountId, tokens: Vec<Token>);

    // Update owner of token
    //
    // Arguments:
    // * `nft_contract_id`: a valid NEAR account
    // * `token_id`: a string representing the token_id
    // * `owner_id`: a valid NEAR account
    //
    //  Return empty if succeded
    fn nft_update_owner_of_token(&mut self, nft_contract_id: AccountId, token_id: TokenId, owner_id: AccountId);
}

#[near_bindgen]
impl Oracle {
    #[init]
    pub fn new() -> Self {
        Self{
            token_by_id_map: UnorderedMap::new(StorageKeys::TokenByIDMap),
            previous_owner_of_token_map: LookupMap::new(StorageKeys::PreviousOwnerOfTokenMap),
            tokens_by_contract_map: LookupMap::new(StorageKeys::TokensByContractMap),
            tokens_by_owner_map: LookupMap::new(StorageKeys::TokensByOwnerMap)
        }
    }
}

#[near_bindgen]
impl OracleInterface for Oracle{
    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>{ 
        let values = self.token_by_id_map.values_as_vector();
        let (start_index, limit) = self.start_limit_paginate(from_index, limit);
        values
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_input|{
                self.parse_token_input_to_token(token_input)
            })
            .collect()
    }

    fn nft_previous_owner(&self, nft_contract_id: AccountId, token_id: TokenId) -> Option<String>{
        self.previous_owner_of_token_map.get(&(self.get_token_input_key(nft_contract_id, token_id)))
    }

    fn nft_token_for_contract(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>{
        let nft_contract_id = account_id;
        let tokens: Vec<Token> = Vec::new();
        let token_keys = match self.tokens_by_contract_map.get(&nft_contract_id) {
            Some(v) => v,
            None => return tokens
        };

        self.get_tokens_by_values(token_keys, from_index, limit)
    }

    fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>{
        let tokens: Vec<Token> = Vec::new();
        let token_keys = match self.tokens_by_owner_map.get(&account_id) {
            Some(v) => v,
            None => return tokens
        };

        self.get_tokens_by_values(token_keys, from_index, limit)
    }
    
    fn consume_tokens(&mut self, nft_contract_id: AccountId, tokens: Vec<Token>){
        if tokens.len() == 0{
            return;
        }

        for v in &tokens{
            let token =  v.clone();

            // parse token_input to standart token struct
            let token_input = TokenInput{
                owner_id: token.owner_id.clone(),
                token_id: token.token_id.clone(),
                metadata: token.metadata, 
                approved_account_ids: token.approved_account_ids,
            };

            let key = self.get_token_input_key(nft_contract_id.clone(), token.token_id.clone());

            // set previous owner if current input (owner_id) is different from previous token data
            // skip when previous owner == current owner input
            if let Some(previous_token) = self.token_by_id_map.get(&key){
                if !(self.update_owner_of_token(&token_input.owner_id, &key, &previous_token)){
                    continue;
                }
            }

            // insert token_input to storage
            self.token_by_id_map.insert(&key, &token_input);

            // map tokens by contract storage
            match self.tokens_by_contract_map.get(&nft_contract_id){
                Some(mut tokens_map) => {
                    tokens_map.insert(&key);
                    self.tokens_by_contract_map.insert(&(nft_contract_id.clone()), &tokens_map); // i think this is uneficient, TODO More research
                },
                None => {
                    let mut set = UnorderedSet::new(StorageKeys::TokensByContractMapSet{ contract_hash: env::sha256(nft_contract_id.as_bytes()) });
                    set.insert(&key);
                    self.tokens_by_contract_map.insert(&(nft_contract_id.clone()), &set);
                }
            };

            // map tokens by owner storage
            self.map_token_by_owner(&key, &token.owner_id);
        }
    }

    fn nft_update_owner_of_token(&mut self, nft_contract_id: AccountId, token_id: TokenId, owner_id: AccountId){
        let key = self.get_token_input_key(nft_contract_id, token_id);
        if let Some(mut previous_token) = self.token_by_id_map.get(&key){
            if !(self.update_owner_of_token(&owner_id, &key, &previous_token)){
                return
            }
            previous_token.owner_id = owner_id.clone();

            self.token_by_id_map.insert(&key, &previous_token);
            self.map_token_by_owner(&key, &owner_id)
        }
    }
}

// private function outside near_bindgen
impl Oracle{
    fn map_token_by_owner(&mut self, key: &String, owner_id: &String){
        match self.tokens_by_owner_map.get(owner_id){
            Some(mut tokens_map) => {
                tokens_map.insert(key);
                self.tokens_by_owner_map.insert(owner_id, &tokens_map); // i think this is uneficient, TODO More research
            },
            None => {
                let mut set = UnorderedSet::new(StorageKeys::TokensByOwnerMapSet{ account_hash: env::sha256((*owner_id).as_bytes()) });
                set.insert(key);
                self.tokens_by_owner_map.insert(owner_id, &set);
            }
        };
    }

    fn update_owner_of_token(&mut self, owner_id: &AccountId, key: &String, previous_token: &TokenInput) -> bool{
        if previous_token.owner_id != *owner_id{
            self.previous_owner_of_token_map.insert(&key, &previous_token.owner_id);

            // delete previous token by owner if the owner changed
            if let Some(mut tokens_map) = self.tokens_by_owner_map.get(&previous_token.owner_id){
                tokens_map.remove(&key);
                self.tokens_by_owner_map.insert(&(previous_token.owner_id.clone()), &tokens_map); // i think this is uneficient, TODO More research
            }
            return true

        } else {
            return false
        }
    }

    fn get_token_input_key(&self, nft_contract_id: AccountId, token_id: TokenId) -> String{
        let key_prefix = token_id;
        let key_suffix = nft_contract_id;
        key_prefix + ":" +&key_suffix
    }

    fn get_tokens_by_values(&self, token_keys: UnorderedSet<String>, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>{
        let (start_index, limit) = self.start_limit_paginate(from_index, limit);

        let values = token_keys.as_vector();
        values
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|key| {
                let token_input = self.token_by_id_map.get(&key).unwrap();
                self.parse_token_input_to_token(token_input)
            })
            .collect()
    }

    fn parse_token_input_to_token(&self, token_input: TokenInput) -> Token{
        Token{
            owner_id: token_input.owner_id,
            token_id: token_input.token_id,
            metadata: token_input.metadata, 
            approved_account_ids: token_input.approved_account_ids,
        }
    }

    fn start_limit_paginate(&self, from_index: Option<U128>, limit: Option<u64>) -> (u128, usize){
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        (start_index, limit)
    }
}