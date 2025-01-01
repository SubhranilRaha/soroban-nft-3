use crate::event;
use crate::types::*;
use soroban_sdk::token::TokenClient;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, BytesN, Env, Map, String, Symbol, Vec,
};
use soroban_sdk::symbol_short;

const VERSION: u32 = 1;

const MAX_NAME_LENGTH: u32 = 128;
const MAX_DESCRIPTION_LENGTH: u32 = 512;
const MAX_LINK_LENGTH: u32 = 100;
const MAX_ROYALTY_SHARE: u32 = 5000;

const MAX_TOKENS: u32 = 10000; // Limit of 10,000 tokens
const MINT_COST: i128 = 1; // 1 XML required to mint each token
const FREE_MINT_LIMIT: u32 = 1000; // First 1000 NFTs are free

#[contract]
pub struct StellarLogos;

#[contractimpl]
impl StellarLogos {
    pub fn initialize(env: Env, collection_info: CollectionInfo, freeze_minter: bool) {
        if env.storage().instance().has(&DataKey::CollectionInfo) {
            panic_with_error!(&env, Error::Initialized);
        }

        collection_info.creator.require_auth();

        if freeze_minter {
            env.storage().instance().set(&DataKey::MinterFrozen, &true);
        }

        _set_collection_info(&env, &collection_info);

        event::initialized(&env);
    }

    pub fn hello(env: Env) {
        event::hello(&env);
    }

    pub fn hello2(env: Env) {
        event::hello(&env);
    }

    pub fn transfer(env: Env, from: Address, to: Address, token_id: u32) {
        let mut token: TokenInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Token(token_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotNFT));

        // Verify the sender is the owner
        if token.owner != from {
            panic_with_error!(&env, Error::NotAuthorized);
        }

        from.require_auth();

        //   token.check_can_send(&env, from.clone());

        token.owner = to.clone();
        //  token.approvals = Map::new(&env);

        _set_token_info(&env, token_id, &token);

        event::transfer(&env, token_id, to);
    }

    pub fn mint(env: Env, owner: Address, token_uri: String) {
        owner.require_auth();
        // let topics:(Symbol, Address, String) = (
        //     symbol_short!("mint"),
        //     owner.clone(),  // Clone here for first use
        //     token_uri.clone()  // Clone here for first use
        // );
        let token_id =
            env.storage()
                .instance()
                .get(&DataKey::TokensCount)
                .map_or(1, |count: u32| {
                    count
                        .checked_add(1)
                        .unwrap_or_else(|| panic_with_error!(&env, Error::MaxTokensReached))
                });

        if token_id > MAX_TOKENS {
            panic_with_error!(&env, Error::MaxTokensReached);
        }

        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        if token_id > FREE_MINT_LIMIT {
            let token_client = TokenClient::new(&env, &collection_info.payment_token);
            let required_amount = MINT_COST * 10_000_000;

            // Single transfer call combines balance check and payment
            token_client.transfer(&owner, &collection_info.creator, &required_amount);
        }
        let token: TokenInfo = TokenInfo {
            owner: owner.clone(),
            approvals: Map::new(&env),
            token_uri: token_uri.clone(),
        };

        _set_token_info(&env, token_id, &token);

        _change_tokens_count(&env, false);

        let max_token_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MaxTokenId)
            .unwrap_or(0);

        if max_token_id < token_id {
            env.storage()
                .instance()
                .set(&DataKey::MaxTokenId, &token_id);
        }
        // env.events().publish(topics, (owner.clone(), token_uri));
        event::mint(&env, owner, token_id);
    }

    pub fn bulk_mint(env: Env, owner: Address, token_uris: Vec<String>) {
        owner.require_auth();

        let current_count = env
            .storage()
            .instance()
            .get(&DataKey::TokensCount)
            .unwrap_or(0);
        let new_count = current_count + (token_uris.len() as u32);

        if new_count > MAX_TOKENS {
            panic_with_error!(&env, Error::MaxTokensReached);
        }

        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        // Calculate how many tokens need to be paid for
        let free_tokens_remaining = if current_count < FREE_MINT_LIMIT {
            FREE_MINT_LIMIT - current_count
        } else {
            0
        };

        let paid_tokens_count = if token_uris.len() as u32 > free_tokens_remaining {
            (token_uris.len() as u32 - free_tokens_remaining) as i128
        } else {
            0
        };

        //CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC

        // Handle payment if necessary
        if paid_tokens_count > 0 {
            // let address_str = String::from_str(&env,&collection_info.payment_token);
            let token_client = TokenClient::new(&env, &collection_info.payment_token);

            // Calculate required payment
            let required_amount = paid_tokens_count * MINT_COST * 10000000;

            // Check if owner has sufficient balance
            let owner_balance = token_client.balance(&owner);
            if owner_balance < required_amount {
                panic_with_error!(&env, Error::InsufficientBalance);
            }

            // Transfer payment for paid tokens
            token_client.transfer(
                &owner,                   // from
                &collection_info.creator, // to
                &required_amount,         // amount
            );
        }

        // Proceed with minting
        let mut tokens_count = current_count;
        let mut minted_tokens: Vec<(u32, String)> = Vec::new(&env);

        for token_uri in token_uris.iter() {
            tokens_count += 1;
            let token_id = tokens_count;

            let token: TokenInfo = TokenInfo {
                owner: owner.clone(),
                approvals: Map::new(&env),
                token_uri: token_uri.clone(),
            };

            _set_token_info(&env, token_id, &token);
            minted_tokens.push_back((token_id, token_uri.clone()));
        }

        env.storage()
            .instance()
            .set(&DataKey::MaxTokenId, &tokens_count);
        env.storage()
            .instance()
            .set(&DataKey::TokensCount, &tokens_count);

        event::bulk_mint(&env, owner, minted_tokens);
    }

    pub fn freeze_collection(env: Env) {
        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        collection_info.admin.require_auth();

        env.storage().instance().set(&DataKey::Frozen, &true);

        event::freeze(&env);
    }

    pub fn update_token_url(env: Env, token_id: u32, token_uri: String) {
        if env.storage().instance().has(&DataKey::Frozen) {
            panic_with_error!(&env, Error::Frozen)
        }

        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        collection_info.admin.require_auth();

        let mut token: TokenInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Token(token_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotNFT));

        token.token_uri = token_uri.clone();

        _set_token_info(&env, token_id, &token);

        event::token_updated(&env, token_id, token_uri);
    }

    pub fn update_collection_info(env: Env, new_collection_info: CollectionInfo) {
        if env.storage().instance().has(&DataKey::Frozen) {
            panic_with_error!(&env, Error::Frozen)
        }

        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        collection_info.admin.require_auth();

        if collection_info.minter != new_collection_info.minter {
            if env.storage().instance().has(&DataKey::MinterFrozen) {
                panic_with_error!(&env, Error::MinterFrozen)
            }
        }

        if collection_info.creator != new_collection_info.creator {
            panic_with_error!(&env, Error::InvalidCollectionInfo)
        }

        _set_collection_info(&env, &new_collection_info);

        event::collection_updated(&env);
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Frozen) {
            panic_with_error!(&env, Error::Frozen)
        }

        let collection_info: CollectionInfo = env
            .storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));

        collection_info.admin.require_auth();

        env.deployer().update_current_contract_wasm(hash.clone());

        event::upgraded(&env, hash);
    }

    pub fn extend_ttl_collection(env: Env, start_after: u32, limit: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if limit > 0 {
            let mut end = start_after + limit;

            let max_token_id: u32 = env
                .storage()
                .instance()
                .get(&DataKey::MaxTokenId)
                .unwrap_or(0);

            if end > max_token_id {
                end = max_token_id;
            }

            for n in start_after..=end {
                if env.storage().persistent().has(&DataKey::Token(n)) {
                    env.storage().persistent().extend_ttl(
                        &DataKey::Token(n),
                        PERSISTENT_LIFETIME_THRESHOLD,
                        PERSISTENT_BUMP_AMOUNT,
                    );
                }
            }
        }
    }

    pub fn extend_ttl_item(env: Env, token_id: u32) {
        if env.storage().persistent().has(&DataKey::Token(token_id)) {
            env.storage().persistent().extend_ttl(
                &DataKey::Token(token_id),
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }
    }

    // Getters

    pub fn is_collection_frozen(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Frozen)
    }

    pub fn get_collection_info(env: Env) -> CollectionInfo {
        env.storage()
            .instance()
            .get(&DataKey::CollectionInfo)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }

    pub fn get_token_info(env: Env, token_id: u32) -> TokenInfo {
        let token: TokenInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Token(token_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotNFT));

        token
    }

    // ... existing code ...

    pub fn get_all_minted_tokens(
        env: Env,
        start_after: u32,
        limit: u32,
    ) -> Vec<(u32, String, Address)> {
        let mut output: Vec<(u32, String, Address)> = Vec::new(&env);
        let max_token_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MaxTokenId)
            .unwrap_or(0);

        let mut end = start_after + limit;
        if end > max_token_id {
            end = max_token_id;
        }

        // Iterate from end to start_after to get the latest tokens first
        for token_id in (start_after..=end).rev() {
            if env.storage().persistent().has(&DataKey::Token(token_id)) {
                let token: TokenInfo = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Token(token_id))
                    .unwrap();
                output.push_back((token_id, token.token_uri, token.owner));
            }
        }

        output
    }

    pub fn get_tokens(
        env: Env,
        owner: Option<Address>,
        start_after: u32,
        limit: u32,
    ) -> Vec<(u32, String, Address)> {
        let mut output: Vec<(u32, String, Address)> = Vec::new(&env);
        let max_token_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MaxTokenId)
            .unwrap_or(0);

        let mut end = start_after + limit;
        if end > max_token_id {
            end = max_token_id;
        }

        if owner.is_some() {
            let owner_unwrapped = owner.unwrap();

            // Iterate from end to start_after to get the latest tokens first
            for n in (start_after..=end).rev() {
                if env.storage().persistent().has(&DataKey::Token(n)) {
                    let token: TokenInfo = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Token(n))
                        .unwrap_or_else(|| panic_with_error!(&env, Error::NotNFT));

                    if token.owner == owner_unwrapped {
                        output.push_back((n, token.token_uri, token.owner));
                    }
                }
            }
        } else {
            // Iterate from end to start_after to get the latest tokens first
            for n in (start_after..=end).rev() {
                if env.storage().persistent().has(&DataKey::Token(n)) {
                    let token: TokenInfo = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Token(n))
                        .unwrap_or_else(|| panic_with_error!(&env, Error::NotNFT));
                    output.push_back((n, token.token_uri, token.owner));
                }
            }
        }

        output
    }

    pub fn get_max_token_id(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::MaxTokenId)
            .unwrap_or(0)
    }

    pub fn get_tokens_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TokensCount)
            .unwrap_or(0)
    }

    pub fn version() -> u32 {
        VERSION
    }
}

fn _change_tokens_count(env: &Env, decrease: bool) {
    let mut tokens_count: u32 = env
        .storage()
        .instance()
        .get(&DataKey::TokensCount)
        .unwrap_or(0);

    if decrease {
        tokens_count -= 1;
    } else {
        tokens_count += 1;
    }

    env.storage()
        .instance()
        .set(&DataKey::TokensCount, &tokens_count);
}

fn _set_token_info(env: &Env, token_id: u32, token_info: &TokenInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::Token(token_id), token_info);
    env.storage().persistent().extend_ttl(
        &DataKey::Token(token_id),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

fn _set_collection_info(env: &Env, collection_info: &CollectionInfo) {
    if collection_info
        .name
        .as_ref()
        .map_or(false, |name| name.len() > MAX_NAME_LENGTH)
        || collection_info
            .description
            .as_ref()
            .map_or(false, |desc| desc.len() > MAX_DESCRIPTION_LENGTH)
        || collection_info
            .external_link
            .as_ref()
            .map_or(false, |link| link.len() > MAX_LINK_LENGTH)
        || collection_info
            .royalty_info
            .as_ref()
            .map_or(false, |royalty| royalty.share > MAX_ROYALTY_SHARE)
    {
        panic_with_error!(env, Error::InvalidCollectionInfo);
    }

    if collection_info.external_link.is_some() {
        if collection_info.external_link.clone().unwrap().len() > MAX_LINK_LENGTH {
            panic_with_error!(&env, Error::InvalidCollectionInfo);
        }
    }

    if collection_info.royalty_info.is_some() {
        if collection_info.royalty_info.clone().unwrap().share > MAX_ROYALTY_SHARE {
            panic_with_error!(&env, Error::InvalidCollectionInfo);
        }
    }

    env.storage()
        .instance()
        .set(&DataKey::CollectionInfo, collection_info);
}
