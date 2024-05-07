use anchor_lang::prelude::*;

use crate::state::{Collection, Admin};

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    saleStartTime: i64,
    maxSupply: u64,
    price: u64,
    stableId: String,
    reference: String,
)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
    #[account(
        init,
        seeds = [b"collection", reference.as_bytes()],
        bump,
        payer = admin,
        space = Collection::INIT_SPACE + name.len() + symbol.len() + stableId.len() + reference.len(),
    )] 
    pub collection: Account<'info, Collection>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateCollection<'info> {
    pub fn create(
        &mut self,
        name: String,
        symbol: String,
        sale_start_time: i64,
        max_supply: u64,
        price: u64,
        stable_id: String,
        reference: String,
    ) -> Result<()> {
        
        self.collection.set_inner(
            Collection {
                name,
                symbol,
                owner: *self.admin.key,
                sale_start_time,
                max_supply,
                total_supply: 0,
                price,
                stable_id,
                reference,
            }
        );

        Ok(())
    }
}

