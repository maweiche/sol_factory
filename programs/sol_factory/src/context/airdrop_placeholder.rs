use solana_program::native_token::LAMPORTS_PER_SOL;
use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
        associated_token::{AssociatedToken, Create, create},
        token::Token,  
        token_interface::{MintTo, mint_to, set_authority, SetAuthority}
    },
    solana_program::{
        system_instruction, 
        program::invoke,
        sysvar::instructions::{
            self,
            load_current_index_checked,
            load_instruction_at_checked
        }
    },
};
use std::str::FromStr;
use crate::{
    state::{Placeholder, Collection, Protocol}, 
    errors::{BuyingError, ProtocolError},
    constant::ED25519_PROGRAM_ID
};

#[derive(Accounts)]
pub struct AirdropPlaceholder<'info> {
    /// CHECK: Buyer is being added by the collection owner
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Account<'info, Collection>,
    /// CHECK
    #[account(mut)]
    pub collection_owner: Signer<'info>,
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
    #[account(address = instructions::ID)]
    /// CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,
}

impl<'info> AirdropPlaceholder<'info> {
    pub fn airdrop(
        &mut self,
        bumps: AirdropPlaceholderBumps,
    ) -> Result<()> {

        /*
        
            Airdrop Placeholder Nft Ix:

            Some security check:
            - The admin_state.publickey must match the signing admin.

            What these Instructions do:
            - Creates a transfer of a placeholder NFT.
            - Invokes a transfer of SOL (price of mint + adminFee) from the buyer to the collection owner & admin.
            - Increase the total_supply on the collection (total minted nfts).

            - Airdrop Functionality
                - Attached to the instructions will be a ED25519 txn w/ a signature and message
                - If signature matches admin, then the buyer will be airdropped the mint without paying the mint price
                - The inputted buyer must match the buyer from the ED25519 message
        */

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

        // The 2nd transfer instruction is the fee for the mint since the admin wallet is the payer of second mint
        // current cost to mint placeholder + nft + burn placeholder = ~0.02 - 0.03 SOL
        let admin_fee = 0.075;
        let admin_fee_in_lamports = admin_fee as u64 * LAMPORTS_PER_SOL;
        let transfer_instruction_two = system_instruction::transfer(
            &self.collection_owner.key(),
            &self.payer.key(),
            admin_fee_in_lamports,
        );

         // Instruction Check
         let ixs = self.instructions.to_account_info();
         let current_index = load_current_index_checked(&ixs)? as usize;
        
         // If the current index is greater than 0, then we can check for the airdrop instructions
         if current_index > 0 {
             match load_instruction_at_checked(current_index - 1, &ixs) {
                Ok(signature_ix) => {
                    if Pubkey::from_str(ED25519_PROGRAM_ID).unwrap() == signature_ix.program_id {
                         // Ensure signing authority is correct
                       require!(
                         self.collection_owner.key
                             .to_bytes()
                             .eq(&signature_ix.data[16..48]),
                         ProtocolError::UnauthorizedAdmin,
                       );
 
                       let mut message_data: [u8; 32] = [0; 32];
                       message_data.copy_from_slice(&signature_ix.data[112..144]);
                       let _buyer = Pubkey::from(message_data);

                       require!(
                         _buyer == *self.buyer.key,
                         ProtocolError::UnauthorizedAdmin,
                       );

                       invoke(
                        &transfer_instruction_two,
                        &[
                            self.collection_owner.to_account_info(),
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
                    } else {
                        // NO ED25519 instruction
                        Err(ProtocolError::InstructionsNotCorrect)?;
                    }
                }
                Err(_) => {
                    // NO ED25519 instruction
                    Err(ProtocolError::InstructionsNotCorrect)?;
                }
            }
        }
        Ok(())
    }
}