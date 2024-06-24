use anchor_lang::prelude::*;

mod state;
mod errors;
mod constant;

mod context;
use context::*;

declare_id!("4GuhLkfXp3hJAeVrgozxhimPVvpJJ93MHpahqbnxAddG");

#[program]
pub mod sol_factory {
    use super::*;

    pub fn intialize_protocol_account(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.initialize_protocol()
    }

    pub fn lock_protocol(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.change_locked_setting()
    }

    pub fn initialize_admin_account(ctx: Context<AdminInit>, username: String) -> Result<()> {
        ctx.accounts.initialize_admin(username)
    }

    pub fn create_collection(ctx: Context<CreateCollection>, 
        reference: Pubkey, name: String, symbol: String, sale_start_time: i64, max_supply: u64, price: u64, stable_id: String, whitelist: Vec<Pubkey>, whitelist_start_time: i64, whitelist_price: u64) -> Result<()> {
        ctx.accounts.create(reference, name, symbol, sale_start_time, max_supply, price, stable_id, whitelist, whitelist_start_time, whitelist_price)
    }

    pub fn create_nft(ctx: Context<CreateNft>, id: u64, uri: String, name: String,  attributes: Vec<Attributes>) -> Result<()> {
        ctx.accounts.create(id, uri, name, attributes, ctx.bumps)
    }

    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        ctx.accounts.transfer(ctx.bumps)
    }

    pub fn create_placeholder(ctx: Context<CreatePlaceholder>, id: u64, uri: String) -> Result<()> {
        ctx.accounts.create(id, uri, ctx.bumps)
    }

    pub fn buy_placeholder(ctx: Context<BuyPlaceholder>) -> Result<()> {
        ctx.accounts.buy(ctx.bumps)
    }

    pub fn burn_placeholder(ctx: Context<BurnPlaceholder>) -> Result<()> {
        ctx.accounts.burn(ctx.bumps)
    }
}

