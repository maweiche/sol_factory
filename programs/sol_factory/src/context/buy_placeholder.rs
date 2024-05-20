use solana_program::native_token::LAMPORTS_PER_SOL;

use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
        associated_token::{AssociatedToken, Create, create},
        token::Token,  
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
    solana_program::{program_memory::sol_memcpy, system_instruction, program::invoke},
};



use crate::{
    state::{Placeholder, CompletedPlaceholder, Collection}, 
    errors::BuyingError
};

#[derive(Accounts)]
pub struct BuyPlaceholder<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Account<'info, Collection>,

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
    pub system_program: Program<'info, System>,
}

impl<'info> BuyPlaceholder<'info> {
    pub fn buy(
        &mut self,
        bumps: BuyPlaceholderBumps,
    ) -> Result<()> {

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];
        
        // require!(
        //     self.listing.starting_time < Clock::get()?.unix_timestamp || 
        //     self.listing.starting_time < Clock::get()?.unix_timestamp - 6 * 3600 && membership == 1 || 
        //     self.listing.starting_time < Clock::get()?.unix_timestamp - 24 * 3600 && membership == 2, 
        //     BuyingError::NotTimeYet
        // );

        require!(
            self.collection.sale_start_time <= Clock::get()?.unix_timestamp,
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
        let amount_in_lamports = self.placeholder.price * LAMPORTS_PER_SOL;
        let transfer_instruction = system_instruction::transfer(
            &self.buyer.key(),
            &self.placeholder.to_account_info().key,
            amount_in_lamports,
        );

        invoke(
            &transfer_instruction,
            &[
                self.buyer.to_account_info(),
                self.payer.to_account_info(),
                self.placeholder.to_account_info(),
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

        let info = self.placeholder.to_account_info(); 
        let mut data = info.try_borrow_mut_data()?;

        // Transform to CompletedPlaceholder
        let completed_placeholder = CompletedPlaceholder {
            id: self.placeholder.id,
            collection: self.placeholder.collection,
            reference: self.placeholder.reference.clone(),
            name: self.placeholder.name.clone(),
            price : self.placeholder.price,
            time_stamp: Clock::get()?.unix_timestamp,
            buyer: self.buyer.key(),
        };

        // set the collection.total_supply += 1
        self.collection.total_supply += 1;

        // Serialize
        let mut writer: Vec<u8> = vec![];
        completed_placeholder.try_serialize(&mut writer)?;
        writer.truncate(CompletedPlaceholder::INIT_SPACE);

        sol_memcpy(&mut data, &writer, writer.len());

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