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
#[instruction(username: String)]
pub struct AdminInit<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Option<Account<'info, Admin>>,

    pub new_admin: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = Admin::INIT_SPACE + 5,
        seeds = [b"admin_state", new_admin.key().as_ref()],
        bump
    )]
    pub new_admin_state: Account<'info, Admin>,

    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> AdminInit<'info> {
    pub fn initialize_admin(
        &mut self,
        username: String,
    ) -> Result<()> {

        /*
        
            Create a new Admin Ix:

            Some security check:
            - Check if the account that is initializing the admin is the admin itself, if not 
            it should be the mutlisig account that is the admin of the enitre protocol.
            - Save the Time of initialization to render it useless for the first 12h of initialization.

            What the Instruction does:
            - Initialize the new admin account with the username (so we can monitor who are the admin
            account atm in an easy way) and the publickey of the new admin.

        */
        
        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        require!(self.admin_state.is_some() || self.admin.key() == constant::multisig_wallet::id(), SetupError::Unauthorized);
        
        self.new_admin_state.set_inner(Admin {
            publickey: self.new_admin.key(),
            username,
            initialized: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

