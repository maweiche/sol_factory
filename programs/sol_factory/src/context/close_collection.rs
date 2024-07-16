use anchor_lang::prelude::*;
use crate::state::{Collection, Protocol, Admin};
use crate::errors::ProtocolError;

#[derive(Accounts)]

pub struct CloseCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: this is ok because admin is setting up on owner behalf
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"collection", owner.key().as_ref()],
        bump,
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

impl<'info> CloseCollection<'info> {
    pub fn close(
        &mut self,
    ) -> Result<()> {

        /*
        
            Create Collection Ix:

            Some security check:
            - The admin_state.publickey must match the signing admin.

            What these Instructions do:
            - Closes the collection by updating the sale end time to the current moment and setting the max supply to the total supply.
        */

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);
        require!(self.admin_state.publickey == *self.admin.key, ProtocolError::UnauthorizedAdmin);


        // update the collection to sale_end_time: Clock::get()?.unix_timestamp, and max_supply: self.collection.total_supply,
        self.collection.sale_end_time = Clock::get()?.unix_timestamp;
        self.collection.max_supply = self.collection.total_supply;
                

        Ok(())
    }

}

// add_sale_start_time