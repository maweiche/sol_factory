use anchor_lang::prelude::*;

use crate::state::Collection;

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    sale_start_time: i64,
    max_supply: u64,
    price: u64,
    stable_id: String,
    reference: String
)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        seeds = [b"collection", owner.key().as_ref()],
        bump,
        payer = owner,
        space = Collection::INIT_SPACE + name.len() + symbol.len() + stable_id.len() + 8 + 8 + 8 + 8 + 4 + 4,
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
        reference: String
    ) -> Result<()> {
        
        self.collection.set_inner(
            Collection {
                name,
                symbol,
                owner: *self.owner.key,
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

