use anchor_lang::prelude::*;
use crate::state::{Collection, Protocol, Admin};
use crate::errors::ProtocolError;

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
)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [b"collection", owner.key().as_ref()],
        bump,
        payer = owner,
        space = Collection::INIT_SPACE + 54 + url.len() + name.len() + stable_id.len(),
    )] 
    pub collection: Account<'info, Collection>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
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
    ) -> Result<()> {

        /*
        
            Create Collection Ix:

            Some security check:
            - The admin_state.publickey must match the signing admin.

            What these Instructions do:
            - Creates a Collection that can be used to mint NFTs.
        */

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);
        require!(self.admin_state.publickey == *self.admin.key, ProtocolError::UnauthorizedAdmin);

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
            }
        );

        Ok(())
    }
}

