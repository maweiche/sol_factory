use anchor_lang::prelude::*;
use crate::{
    state::{
        Admin,
        Protocol
    },
    constant,
    errors::{SetupError, ProtocolError},
};

#[derive(Accounts)]
pub struct AdminRemove<'info> {
    /// CHECK: This is the admin being removed, it's ok because the signer will be required to be the overall authority on program
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    #[account(
        mut,
        close = primary_admin, // this is where the account rent funds will be sent to after the admin is removed
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
    pub primary_admin: Signer<'info>,
    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> AdminRemove<'info> {
    pub fn remove_admin(
        &mut self
    ) -> Result<()> {

        /*
        
            Remove Admin Ix:

            Some security check:
            - Check if the account signing is the primary admin from the multisig wallet.

            What the Instruction does:
            - Closes the Admin_State account which is necessary for Admin rights, this is intended to only be used when the admin is compromised.
            - Returns any account rent of the Admin_State account to the multisig wallet.   

        */
        
        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);
        require!(self.primary_admin.key() == constant::multisig_wallet::id(), SetupError::Unauthorized);
    
        
        Ok(())
    }
}

