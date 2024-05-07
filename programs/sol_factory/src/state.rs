use anchor_lang::prelude::*;

// Setup State
#[account]
pub struct Protocol {
    pub locked: bool,
}

impl Space for Protocol {
    const INIT_SPACE: usize = 8 + 1;
}

#[account]
pub struct Admin {
    pub publickey: Pubkey,
    pub username: String,
    pub initialized: i64,
}

impl Space for Admin {
    const INIT_SPACE: usize = 8 + 32 + 4 + 8;
}

#[account]
pub struct Collection {
    pub name: String,
    pub symbol: String,
    pub owner: Pubkey,
    pub sale_start_time: i64,
    pub max_supply: u64,
    pub total_supply: u64,
    pub price: u64,
    pub stable_id: String,
    pub reference: String,
}

impl Space for Collection {
    const INIT_SPACE: usize = 8 + 4 + 4 + 32 + 8 + 8 + 8 + 8 + 4 + 4;
}

#[account]
pub struct Nft {
    pub id: u64,
    pub reference: String,
    pub collection: Pubkey,
    pub image: String,
    pub seed: u64,
    pub model_name: String,
    pub model_hash: String,
}

impl Space for Nft {
    const INIT_SPACE: usize = 8 + 4 + 4  + 32 + 4 + 4 + 8 + 4 + 4;
}