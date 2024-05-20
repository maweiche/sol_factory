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

// Fragment State
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
pub struct Placeholder {
    pub id: u64,
    pub collection: Pubkey,
    pub reference: String,
    pub name: String,
    pub price: u64,
    pub time_stamp: i64,
}

impl Space for Placeholder {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 2 + 8 + 8;
}

#[account]
pub struct CompletedPlaceholder {
    pub id: u64,
    pub collection: Pubkey,
    pub reference: String,
    pub name: String,
    pub price: u64,
    pub time_stamp: i64,
    pub buyer: Pubkey,
}

impl Space for CompletedPlaceholder {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 8 + 32;
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
    pub price: u64,
    pub time_stamp: i64,
    // pub inscription: Pubkey,
    pub inscription: String,
    pub rank: u16,
}

impl Space for AiNft {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 32 + 2 + 8;
}

#[account]
pub struct CompletedAiNft {
    pub id: u64,
    pub collection: Pubkey,
    pub reference: String,
    pub price: u64,
    // pub inscription: Pubkey,
    pub inscription: String,
    pub rank: u16,
    pub buyer: Pubkey,
}

impl Space for CompletedAiNft {
    const INIT_SPACE: usize = 8 + 8 + 32 + 4 + 2 + 32 + 2 + 8 + 32;
}