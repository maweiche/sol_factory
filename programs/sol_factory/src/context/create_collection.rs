use anchor_lang::prelude::*;

use crate::state::{Collection, Protocol};
use crate::errors::ProtocolError;
use crate::state::WhiteList;

#[derive(Accounts)]
#[instruction(
    reference: Pubkey,
    name: String,
    symbol: String,
    url: String,
    sale_start_time: i64,
    max_supply: u64,
    price: u64,
    stable_id: String,
    whitelist: Vec<Pubkey>,
    whitelist_start_time: i64,
    whitelist_price: u64,
)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [b"collection", owner.key().as_ref()],
        bump,
        payer = owner,
        space = Collection::INIT_SPACE + 54 + url.len() + name.len() + stable_id.len() + (whitelist.len() * 32) as usize,
    )] 
    pub collection: Account<'info, Collection>,
    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateCollection<'info> {
    pub fn create(
        &mut self,
        reference: Pubkey,
        name: String,
        symbol: String,
        url: String,
        sale_start_time: i64,
        max_supply: u64,
        price: u64,
        stable_id: String,
        whitelist: Vec<Pubkey>,
        whitelist_start_time: i64,
        whitelist_price: u64,

    ) -> Result<()> {

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        self.collection.set_inner(
            Collection {
                reference,
                name,
                symbol,
                owner: *self.owner.key,
                url,
                sale_start_time,
                max_supply,
                total_supply: 0,
                price,
                stable_id,
                whitelist: WhiteList {
                    wallets: whitelist,
                },
                whitelist_start_time,
                whitelist_price,
            }
        );

        Ok(())
    }
}

