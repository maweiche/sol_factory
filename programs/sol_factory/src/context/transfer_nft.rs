use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
        associated_token::{AssociatedToken, Create, create},  
        token::Token,
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
    solana_program::program_memory::sol_memcpy,
};

use crate::{
    state::{AiNft, CompletedAiNft}, 
    // errors::BuyingError
};

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
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

impl<'info> TransferNft<'info> {
    pub fn transfer(
        &mut self,
        bumps: TransferNftBumps,
    ) -> Result<()> {

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

        let info = self.nft.to_account_info(); 
        let mut data = info.try_borrow_mut_data()?;

        // Transform to CompletedAiNft
        let completed_nft = CompletedAiNft {
            id: self.nft.id,
            collection: self.nft.collection,
            reference: self.nft.reference.clone(),
            price : self.nft.price,
            inscription: self.nft.inscription.clone(),
            rank: self.nft.rank,
            buyer: self.buyer.key(),
        };

        // Serialize
        let mut writer: Vec<u8> = vec![];
        completed_nft.try_serialize(&mut writer)?;
        writer.truncate(CompletedAiNft::INIT_SPACE);

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