use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, LookupMap};
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

    // TODO unlimited LIMIT
    pub fn nft_tokens(&self, from_index: u64, limit: u64) -> Vec<Token>{
        // pagination tutorial as mention in this doc: https://www.near-sdk.io/contract-structure/collections#pagination-with-persistent-collections
        let values = self.token_by_id_map.values_as_vector();
        let mut tokens: Vec<Token> = Vec::new();
        for i in from_index..std::cmp::min(from_index + limit, self.token_by_id_map.len()){
            let token_input = values.get(i).unwrap();

            let token = Token{
                owner_id: token_input.owner_id,
                token_id: token_input.token_id,
                metadata: token_input.metadata, 
                approved_account_ids: token_input.approved_account_ids,
            };
            
            tokens.push(token);
        }

        tokens
    }

    pub fn nft_previous_owner(&self, nft_contract_id: AccountId, token_id: TokenId) -> Option<String>{
        self.previous_owner_of_token_map.get(&(self.get_token_input_key(nft_contract_id, token_id)))
    }

    pub fn nft_token_for_contract(&self, nft_contract_id: AccountId, from_index: u64, limit: u64) -> Vec<Token>{
        let mut tokens: Vec<Token> = Vec::new();
        let token_keys = match self.tokens_by_contract_map.get(&nft_contract_id) {
            Some(v) => v,
            None => return tokens
        };

        let values = token_keys.as_vector();
        for v in values.iter(){
            env::log(v.as_bytes());
        };

        for i in from_index..std::cmp::min(from_index + limit, values.len()){
            let key = values.get(i).unwrap();

            env::log(key.as_bytes());

            match self.token_by_id_map.get(&key) {
                Some(token_input) => {
                    let token = Token{
                        owner_id: token_input.owner_id,
                        token_id: token_input.token_id,
                        metadata: token_input.metadata, 
                        approved_account_ids: token_input.approved_account_ids,
                    };
                    
                    tokens.push(token);
                },
                None => ()
            }
        }

        tokens
    }

    pub fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: u64, limit: u64) -> Vec<Token>{
        let res: Vec<Token> = Vec::new();
        res
    }

    pub fn consume_tokens(&mut self, nft_contract_id: AccountId, tokens: Vec<Token>){
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
            if let Some(previous_token) = self.token_by_id_map.get(&key){
                if token.owner_id != previous_token.owner_id{
                    self.previous_owner_of_token_map.insert(&key, &previous_token.owner_id);

                    // delete previous token by owner if the owner changed
                    if let Some(mut tokens_map) = self.tokens_by_owner_map.get(&previous_token.owner_id){
                        tokens_map.remove(&key);
                        self.tokens_by_owner_map.insert(&(previous_token.owner_id.clone()), &tokens_map); // i think this is uneficient, TODO More research
                    }
                } else {

                    // iterate to the next loop if the data exist and owner_id not changed
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
            match self.tokens_by_owner_map.get(&token.owner_id){
                Some(mut tokens_map) => {
                    tokens_map.insert(&key);
                    self.tokens_by_owner_map.insert(&(token.owner_id.clone()), &tokens_map); // i think this is uneficient, TODO More research
                },
                None => {
                    let mut set = UnorderedSet::new(StorageKeys::TokensByOwnerMapSet{ account_hash: env::sha256(token.owner_id.as_bytes()) });
                    set.insert(&key);
                    self.tokens_by_owner_map.insert(&(token.owner_id.clone()), &set);
                }
            };
        }
    }
}

// private function outside near_bindgen
impl Oracle{
    pub fn get_token_input_key(&self, nft_contract_id: AccountId, token_id: TokenId) -> String{
        let key_prefix = token_id;
        let key_suffix = nft_contract_id;
        key_prefix + ":" +&key_suffix
    }
}