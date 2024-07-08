use {
    anchor_lang::{
        prelude::*,
        solana_program::program::invoke_signed
    },
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
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
};
use spl_token_2022::instruction::burn;
use crate::state::{Protocol, AiNft, Collection, Placeholder};
use crate::errors::ProtocolError;

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
    pub buyer_placeholder_mint_ata: AccountInfo<'info>,
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

impl<'info> TransferNft<'info> {
    pub fn transfer(
        &mut self,
        bumps: TransferNftBumps,
    ) -> Result<()> {

        /*
        
            Transfer Nft Ix:

            Some security check:
            - The authority of the burn instruction must be admin of the protocol.

            What these Instructions do:
            - Initialize the transfer of the created Ai NFT.
            - Burn the Placeholder NFT.
        */

        // Check if the protocol is locked, if it is, return an error
        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];

        // Initialize ATA if it doesn't exist
        if self.buyer_mint_ata.owner != &self.buyer.key() {
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
        }
        
        // balance before minting
        let _before_data = self.buyer_mint_ata.data.borrow();
        let _before_state = StateWithExtensions::<TokenAccount>::unpack(&_before_data)?;
        
        msg!("before mint balance={}", _before_state.base.amount);

        require!(
            _before_state.base.amount == 0,
            ProtocolError::InvalidBalancePreMint
        );

        

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

        // balance after minting, reload the data
        // let _after_data = self.buyer_mint_ata.data.borrow();
        // let _after_state = StateWithExtensions::<TokenAccount>::unpack(&_after_data)?;

        // msg!("after mint balance={}", _after_state.base.amount);

    //     require!(_after_state.base.amount == 1, ProtocolError::InvalidBalancePostMint);

    //     let _before_burn_data = self.buyer_placeholder_mint_ata.data.borrow();
    //     let _before_burn_state = StateWithExtensions::<TokenAccount>::unpack(&_before_burn_data)?;

    //     // Always check if you got the correct ATA for the burn
    //    require!(_before_burn_state.base.amount == 0, ProtocolError::InvalidBalancePreBurn);

    //     msg!("before burn balance={}", _before_burn_state.base.amount);


        // Burn the placeholder nft
        let ix = burn(
            &self.token_2022_program.key,
            self.buyer_placeholder_mint_ata.to_account_info().key,
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

        // check the post balance of the burn

        // let _after_burn_data = self.buyer_placeholder_mint_ata.data.borrow();   
        // let _after_burn_state = StateWithExtensions::<TokenAccount>::unpack(&_after_burn_data)?;

        // msg!("after burn balance={}", _after_burn_state.base.amount);

        // require!(_after_burn_state.base.amount == 0, ProtocolError::InvalidBalancePostBurn);


        Ok(())
    }
    
}