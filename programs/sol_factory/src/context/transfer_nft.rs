use {
    anchor_lang::{
        prelude::*,
        solana_program::program::invoke_signed
    },
    anchor_spl::{
        token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
        associated_token::{AssociatedToken, Create, create},  
        token::Token,
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
};
use spl_token_2022::instruction::burn;
use crate::state::{AiNft, Collection, Placeholder};

#[derive(Accounts)]
pub struct TransferNft<'info> {
    /// CHECK
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            buyer.key().as_ref(),
            token_2022_program.key().as_ref(),
            mint.key().as_ref()
        ],
        seeds::program = associated_token_program.key(),
        bump
    )]
    /// CHECK
    pub buyer_mint_ata: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"ainft", nft.collection.key().as_ref(), nft.id.to_le_bytes().as_ref()],
        bump,
    )] 
    pub nft: Account<'info, AiNft>,

    #[account(
        mut,
        seeds = [b"mint", nft.key().as_ref()],
        bump
    )]
    /// CHECK
    pub mint: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection", collection.owner.key().as_ref()],
        bump,
    )] 
    pub collection: Account<'info, Collection>,

    #[account(
        seeds = [b"auth"],
        bump
    )]
    /// CHECK:
    pub auth: UncheckedAccount<'info>,

    // *************
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
    pub buyer_placeholder_mint_ata: UncheckedAccount<'info>,
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
    pub placeholder_mint_authority: Signer<'info>,
    // *************

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> TransferNft<'info> {
    pub fn transfer(
        &mut self,
        bumps: TransferNftBumps,
    ) -> Result<()> {

        // sale_start_time and whitelist_start_time are both unix timestamps converted to big numbers
        // if it is before the sale start time, we should check to see if the whitelist is active, if it is not we should throw an error,
        // if it is active we should check to see if the buyer is in the whitelist, if they are not we should throw an error
        // if self.collection.sale_start_time > Clock::get()?.unix_timestamp {
        //     if self.collection.whitelist_start_time < Clock::get()?.unix_timestamp {
        //         if !self.collection.whitelist.wallets.contains(&self.buyer.key()) {
        //             return Err(BuyingError::NotInWhitelist.into());
        //         }
        //     } else {
        //         return Err(BuyingError::NotTimeYet.into());
        //     }
        // }

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];

        // Initialize ATA
        create(
            CpiContext::new(
                self.token_2022_program.to_account_info(), // NEEDS CHANGE TO ATA PROGRAM
                Create {
                    payer: self.payer.to_account_info(), // payer
                    associated_token: self.buyer_mint_ata.to_account_info(),
                    authority: self.buyer.to_account_info(), // owner
                    mint: self.mint.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_2022_program.to_account_info(),
                }
            ),
        )?;

        // Mint the mint
        mint_to(
            CpiContext::new_with_signer(
                self.token_2022_program.to_account_info(),
                MintTo {
                    mint: self.mint.to_account_info(),
                    to: self.buyer_mint_ata.to_account_info(),
                    authority: self.auth.to_account_info(),
                },
                signer_seeds
            ),
            1,
        )?;

        set_authority(
            CpiContext::new_with_signer(
                self.token_2022_program.to_account_info(), 
                SetAuthority {
                    current_authority: self.auth.to_account_info(),
                    account_or_mint: self.mint.to_account_info(),
                }, 
                signer_seeds
            ), 
            AuthorityType::MintTokens, 
            None
        )?;

        let whitelist = &mut self.collection.whitelist.wallets;
        // if the buyer is in the whitelist, remove them
        if whitelist.contains(&self.buyer.key()) {
            whitelist.retain(|&x| x != self.buyer.key());
        }

        let ix = burn(
            &self.token_2022_program.key,
            self.buyer_placeholder_mint_ata.key,
            self.placeholder_mint.key,
            &self.auth.key,
            &[&self.auth.key()],
            1,
        )?;

        invoke_signed(
            &ix,
            &[
                self.buyer_placeholder_mint_ata.to_account_info(),
                self.placeholder_mint.to_account_info(),
                self.placeholder_mint_authority.to_account_info(),
                self.auth.to_account_info(),
                self.token_2022_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }
    
}