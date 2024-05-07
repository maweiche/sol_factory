use anchor_lang::prelude::*;

mod state;
mod errors;
mod constant;

mod context;
use context::*;

declare_id!("2wj57nXtBwFAJS7mezos1mWirfVJWqxiqYAVtSt7W6F6");

#[program]
pub mod sol_factory {
    use super::*;

    // Protocol Config
    pub fn intialize_protocol_account(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.initialize_protocol()
    }

    pub fn lock_protocol(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.change_locked_setting()
    }

    // Adming Config
    pub fn initialize_admin_account(ctx: Context<AdminInit>, username: String) -> Result<()> {
        ctx.accounts.initialize_admin(username)
    }

    // Watch Config
    pub fn create_collection(ctx: Context<CreateCollection>, 
        name: String, 
        symbol: String, 
        sale_start_time: i64, 
        max_supply: u64, 
        price: u64, 
        stable_id: String, 
        reference: String
    ) -> Result<()> {
        ctx.accounts.create(name, symbol, sale_start_time, max_supply, price, stable_id, reference)
    }

    pub fn create_nft(
        ctx: Context<CreateNft>,
        id: u64,
        reference: String,
        collection: Pubkey,
        uri: String,
        image: String,
        seed: u64,
        model_name: String,
        model_hash: String
    ) -> Result<()> {
        ctx.accounts.create(
            id,
            reference,
            uri,
            image,
            seed,
            model_name,
            model_hash,
            ctx.bumps
        )
    }

}

