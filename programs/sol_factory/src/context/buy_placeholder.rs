use solana_program::native_token::LAMPORTS_PER_SOL;

use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
        associated_token::{AssociatedToken, Create, create},
        token::Token,  
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
    solana_program::{system_instruction, program::invoke},
};



use crate::{
    state::{Placeholder, Collection, Protocol}, 
    errors::{BuyingError, ProtocolError},
};

#[derive(Accounts)]
pub struct BuyPlaceholder<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Account<'info, Collection>,
    /// CHECK
    #[account(mut)]
    pub collection_owner: AccountInfo<'info>,

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
    pub mint: UncheckedAccount<'info>,

    #[account(
        seeds = [b"auth"],
        bump
    )]
    /// CHECK:
    pub auth: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyPlaceholder<'info> {
    pub fn buy(
        &mut self,
        bumps: BuyPlaceholderBumps,
    ) -> Result<()> {

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];
    
        require!(
            self.collection.sale_start_time > Clock::get()?.unix_timestamp,
            BuyingError::NotTimeYet
        );

        require!(
            self.collection.max_supply > self.collection.total_supply,
            BuyingError::SoldOut
        );

        // Currency Checks
        // require!(self.currency.key() == ) - TODO

        // Listing Check
        // require!(self.listing.share_sold == self.mint.supply) - TODO

        // Pay the mint
        let amount_in_lamports = (self.placeholder.price * LAMPORTS_PER_SOL) as u64;
        let transfer_instruction = system_instruction::transfer(
            &self.buyer.key(),
            &self.collection_owner.key(),
            amount_in_lamports,
        );

        // The 2nd transfer instruction is the fee for the mint since the admin wallet is the payer of second mint
        // current cost to mint placeholder + nft + burn placeholder = ~0.02 - 0.03 SOL
        let admin_fee = 0.075;
        let admin_fee_in_lamports = admin_fee as u64 * LAMPORTS_PER_SOL;
        let transfer_instruction_two = system_instruction::transfer(
            &self.buyer.key(),
            &self.payer.key(),
            admin_fee_in_lamports,
        );

        
        invoke(
            &transfer_instruction,
            &[
                self.buyer.to_account_info(),
                self.collection_owner.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        invoke(
            &transfer_instruction_two,
            &[
                self.buyer.to_account_info(),
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // Initialize ATA
        create(
            CpiContext::new(
                self.token_2022_program.to_account_info(),
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
        
        // set the collection.total_supply += 1
        self.collection.total_supply += 1;

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

        Ok(())
    }
    
}