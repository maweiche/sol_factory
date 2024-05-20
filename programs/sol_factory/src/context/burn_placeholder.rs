pub use anchor_lang::{
    solana_program::program::invoke_signed,
    prelude::*
};
use anchor_spl::{
    token_2022::Token2022,
    associated_token::AssociatedToken
};

use spl_token_2022::instruction::burn;
use crate::state::Placeholder;


#[derive(Accounts)]
pub struct BurnPlaceholder<'info> {
    ///CHECK: This is fine since we are burning the NFT.
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [
            buyer.key().as_ref(),
            token_2022_program.key().as_ref(),
            placeholder_mint.key().as_ref()
        ],
        seeds::program = associated_token_program.key(),
        bump
    )]
    /// CHECK
    pub buyer_mint_ata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"placeholder", placeholder.collection.key().as_ref(), placeholder.id.to_le_bytes().as_ref()],
        bump,
    )] 
    pub placeholder: Account<'info, Placeholder>,

    #[account(
        mut,
        seeds = [b"mint", placeholder.key().as_ref()],
        bump
    )]
    /// CHECK
    pub placeholder_mint: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        seeds = [b"auth"],
        bump
    )]
    pub authority: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_2022_program: Program<'info, Token2022>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
    ///CHECK: This is fine since we are burning the NFT.
    pub mint: AccountInfo<'info>,
    pub mint_authority: Signer<'info>,
}

impl<'info> BurnPlaceholder<'info> {
    pub fn burn(
        &mut self,
        bumps: BurnPlaceholderBumps,
    ) -> Result<()> {
        let authority = self.authority.key;
        // let mint_pubkey = self.mint.key();
        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.authority],
        ];

        let ix = burn(
            &self.token_2022_program.key,
            self.buyer_mint_ata.key,
            self.mint.key,
            &authority,
            &[authority],
            1,
        )?;

        invoke_signed(
            &ix,
            &[
                self.buyer_mint_ata.to_account_info(),
                self.mint.to_account_info(),
                self.mint_authority.to_account_info(),
                self.authority.to_account_info(),
                self.token_2022_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }
}