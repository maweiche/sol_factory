use anchor_lang::prelude::*;
use crate::{
    state::Protocol,
    constant::multisig_wallet,
    errors::SetupError,
};

#[derive(Accounts)]
pub struct ProtocolSetting<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        space = Protocol::INIT_SPACE,
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> ProtocolSetting<'info> {

    /*
        
        Change Protocol Settings Ix:

        Some security check:
        - Check if the account that is interacting with this instruction is the mutlisig account 
        of the team that is the highest security clearance for the enitre protocol.

        What these Instructions do:
        - Initialize the Protocol account with the new settings.
        - Toggle the lock on the Protocol: render the protocol useless/useful.
    */

    pub fn initialize_protocol(
        &mut self,
    ) -> Result<()> {

        require!(self.admin.key() == multisig_wallet::id(), SetupError::Unauthorized);
        
        self.protocol.locked = true;

        Ok(())
    }

    pub fn change_locked_setting(
        &mut self,
    ) -> Result<()> {

        require!(self.admin.key() == multisig_wallet::id(), SetupError::Unauthorized);
        
        self.protocol.locked = !self.protocol.locked;

        Ok(())
    }
}

