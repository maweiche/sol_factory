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
    pub reference: Pubkey,
    pub name: String,
    pub symbol: String,
    pub owner: Pubkey,
    pub url: String,
    pub sale_start_time: i64,
    pub sale_end_time: i64,
    pub max_supply: u64,
    pub total_supply: u64,
    pub price: f32,
    pub stable_id: String,
}

impl Space for Collection {
    const INIT_SPACE: usize = 8 + 32 + 4 + 4 + 32 + 4 + 8 + 8 + 8 + 8 + 4 + 4 + 8 + 8; 
}

#[account]
pub struct Placeholder {
    pub id: u64,
    pub collection: Pubkey,
    pub reference: String,
    pub name: String,
    pub price: f32,
    pub time_stamp: i64,
}

impl Space for Placeholder {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 2 + 8 + 8;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Attributes {
    pub key: String,
    pub value: String,
}

#[account]
pub struct AiNft {
    pub id: u64,
    pub collection: Pubkey,
    pub reference: String,
    pub price: f32,
    pub time_stamp: i64,
}

impl Space for AiNft {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 32 + 2 + 8;
}