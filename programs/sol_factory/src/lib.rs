use anchor_lang::prelude::*;
mod state;
mod errors;
mod constant;
mod context;
use context::*;

declare_id!("8AETe8uj6pAeDBrNVWvwXFSywjdGjdVapeQADgoWqNH");

#[program]
pub mod sol_factory {
    use super::*;

    pub fn initialize_protocol_account(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.initialize_protocol()
    }

    pub fn lock_protocol(ctx: Context<ProtocolSetting>) -> Result<()> {
        ctx.accounts.change_locked_setting()
    }

    pub fn initialize_admin_account(ctx: Context<AdminInit>, 
        username: String
    ) -> Result<()> {
        ctx.accounts.initialize_admin(username)
    }

    pub fn remove_admin_account(ctx: Context<AdminRemove>) -> Result<()> {
        ctx.accounts.remove_admin()
    }

    pub fn create_collection(ctx: Context<CreateCollection>, 
        reference: Pubkey, 
        name: String, 
        symbol: String, 
        url: String, 
        sale_start_time: i64, 
        sale_end_time: i64,
        max_supply: u64, 
        price: f32, 
        stable_id: String, 
    ) -> Result<()> {
        ctx.accounts.create(reference, name, symbol, url, sale_start_time, sale_end_time, max_supply, price, stable_id)
    }

    pub fn create_nft(ctx: Context<CreateNft>, 
        id: u64, 
        uri: String, 
        name: String,  
        attributes: Vec<Attributes>
    ) -> Result<()> {
        ctx.accounts.create(id, uri, name, attributes, ctx.bumps)
    }

    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        ctx.accounts.transfer(ctx.bumps)
    }

    pub fn create_placeholder(ctx: Context<CreatePlaceholder>, 
        id: u64, 
        uri: String
    ) -> Result<()> {
        ctx.accounts.create(id, uri, ctx.bumps)
    }

    pub fn buy_placeholder(ctx: Context<BuyPlaceholder>) -> Result<()> {
        ctx.accounts.buy(ctx.bumps)
    }

    pub fn airdrop_placeholder(ctx: Context<AirdropPlaceholder>) -> Result<()> {
        ctx.accounts.airdrop(ctx.bumps)
    }
}

