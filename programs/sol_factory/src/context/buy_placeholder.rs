use solana_program::native_token::LAMPORTS_PER_SOL;
use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{
            Token2022, 
            spl_token_2022::{
                instruction::AuthorityType,
                state::Account as TokenAccount,
                extension::StateWithExtensions,
            }},
        associated_token::{AssociatedToken, Create, create},
        token::Token,  
        token_interface::{MintTo, mint_to, set_authority, SetAuthority},
    },
    solana_program::{system_instruction, program::invoke},
};

use crate::{
    constant::{
        ADMIN_FEE, 
        // ADMIN_PERCENTAGE
    }, 
    errors::{BuyingError, ProtocolError}, state::{Collection, Placeholder, Protocol}
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

        /*
        
            Buy Placeholder Nft Ix:

            Some security check:
            - The admin_state.publickey must match the signing admin.

            What these Instructions do:
            - Creates a transfer of a placeholder NFT.
            - Invokes a transfer of SOL (price of mint + adminFee) from the buyer to the collection owner & admin.
            - Increase the total_supply on the collection (total minted nfts).
        */

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        // make sure the current time is greater than the self.collection.sale_start_time 
        // and make sure the current time is less than the self.collection.sale_end_time

        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= self.collection.sale_start_time,
            BuyingError::NotTimeYet
        );

        require!(
            current_time <= self.collection.sale_end_time,
            BuyingError::Expired
        );

        require!(
            self.collection.total_supply < self.collection.max_supply,
            BuyingError::SoldOut
        );

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];
        

        // Pay the mint
        let amount_in_lamports = ((self.placeholder.price * LAMPORTS_PER_SOL as f32) as u64) - ADMIN_FEE;  //// ex. should be (0.3 * 1000000000) - 100000000 = 200000000
        let transfer_instruction = system_instruction::transfer(
            &self.buyer.key(),
            &self.collection_owner.key(),
            amount_in_lamports as u64,
        );  

        // let _admin_fee = ((self.placeholder.price * ADMIN_PERCENTAGE) * LAMPORTS_PER_SOL as f32) + ADMIN_FEE as f32;

        let transfer_instruction_two = system_instruction::transfer(
            &self.buyer.key(),
            &self.payer.key(),
            ADMIN_FEE as u64,
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

        // balance before minting
        {
            let _before_data = self.buyer_mint_ata.data.borrow();
            let _before_state = StateWithExtensions::<TokenAccount>::unpack(&_before_data)?;
        
            // msg!("before mint balance={}", _before_state.base.amount);
        }
        

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

        self.collection.total_supply += 1;

        // msg!("Total supply: {}", self.collection.total_supply);

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

        // check the post balance of the mint
        {
            let _after_data = self.buyer_mint_ata.data.borrow();
            let _after_state = StateWithExtensions::<TokenAccount>::unpack(&_after_data)?;

            // msg!("after mint balance={}", _after_state.base.amount);

            require!(_after_state.base.amount == 1, ProtocolError::InvalidBalancePostMint);
        }

        Ok(())
    }
    
}