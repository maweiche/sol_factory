pub use anchor_lang::{
    solana_program::{
        sysvar::rent::ID as RENT_ID,
        program::{invoke, invoke_signed},
        system_instruction,
        native_token::LAMPORTS_PER_SOL,
    },
    prelude::*
};

pub use anchor_spl::{
    token_2022::{Token2022, spl_token_2022::instruction::AuthorityType},
    associated_token::{AssociatedToken, Create, create},
    token::{Mint, Token, TokenAccount, Transfer, transfer},  
    token_interface::{MintTo, mint_to, set_authority, SetAuthority}
};

pub use spl_token_2022::{
    extension::ExtensionType,
    instruction::{initialize_mint_close_authority, initialize_permanent_delegate, initialize_mint2},
    extension::{
        transfer_hook::instruction::initialize as intialize_transfer_hook,
        metadata_pointer::instruction::initialize as initialize_metadata_pointer,
    },
};

pub use spl_token_metadata_interface::{
    state::{TokenMetadata, Field},
    instruction::{initialize as initialize_metadata_account, update_field as update_metadata_account},
};

pub use crate::{
    state::{Collection, Nft, Admin},
    errors::BuyingError,
};

#[derive(Accounts)]
#[instruction(
    id: u64,
    reference: String,
    collection: Pubkey,
    image: String,
    seed: u64,
    model_name: String,
    model_hash: String,
    uri: String,
)]
pub struct CreateNft<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        seeds = [b"buyer_state", buyer.key().as_ref()],
        bump
    )]
    pub buyer_state: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            buyer.key().as_ref(),
            token_2022_program.key().as_ref(),
            solai.key().as_ref()
        ],
        seeds::program = associated_token_program.key(),
        bump
    )]
    /// CHECK
    pub buyer_solai_ata: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection", collection.reference.as_bytes().as_ref()],
        bump,
    )] 
    pub collection: Account<'info, Collection>,

    #[account(mut)]
    pub collection_owner: AccountInfo<'info>,

    #[account(
        init,
        payer = buyer,
        seeds = [b"nft", collection.reference.as_bytes(), buyer.key.as_ref()],
        bump,
        space = Nft::INIT_SPACE + reference.len() + image.len() + model_name.len() + model_hash.len(),
    )] 
    pub nft: Account<'info, Nft>,

    /// CHECK: this is fine since we are handling all the checks and creation in the program.
    #[account(
        mut,
        seeds = [b"solai", collection.key().as_ref()],
        bump
    )]
    pub solai: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        seeds = [b"auth"],
        bump
    )]
    pub auth: UncheckedAccount<'info>,

    #[account(address = RENT_ID)]
    /// CHECK: this is fine since we are hard coding the rent sysvar.
    pub rent: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateNft<'info> {
    pub fn create(
        &mut self,
        id: u64,
        reference: String,
        image: String,
        seed: u64,
        model_name: String,
        model_hash: String,
        uri: String,
        bumps: CreateNftBumps,
    ) -> Result<()> {

        self.nft.set_inner(
            Nft {
                id,
                reference,
                collection: self.collection.key(),
                image,
                seed,
                model_name,
                model_hash,
            }
        );

        /* Token2022 Gibberish */

        // Step 1: Initialize Account
        let size = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[
                // ExtensionType::MintCloseAuthority,
                // ExtensionType::PermanentDelegate,
                ExtensionType::MetadataPointer,
                // ExtensionType::TransferHook,
            ],
        ).unwrap();

        let metadata = TokenMetadata {
            update_authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(self.auth.key())).unwrap(),
            mint: self.solai.key(),
            name: "Sol AI Art".to_string(),
            symbol: "ARTSN".to_string(),
            uri,
            additional_metadata: vec![
                ("season".to_string(), "winter".to_string()),
                ("camera angle".to_string(), "low angle".to_string()),
                ("theme".to_string(), "nichijo".to_string()),
                ("seed".to_string(), "3808958`1".to_string()),
                ("model".to_string(), "gpt-3".to_string()),
                ("model hash".to_string(), "0x1234567890".to_string()),
            ]
        };

        let extension_extra_space = metadata.tlv_size_of().unwrap();
        let rent = &Rent::from_account_info(&self.rent.to_account_info())?;
        let lamports = rent.minimum_balance(size + extension_extra_space);

        let collection_key = self.collection.key();
        let seeds: &[&[u8]; 3] = &[
            b"solai",
            collection_key.as_ref(),
            &[bumps.solai],
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &solana_program::system_instruction::create_account(
                &self.buyer.key(),
                &self.solai.key(),
                lamports,
                (size).try_into().unwrap(),
                &spl_token_2022::id(),
            ),
            &vec![
                self.buyer.to_account_info(),
                self.solai.to_account_info(),
            ],
            signer_seeds
        )?;

        // Step 2: Initialize Extension needed: 

        // 2.1: Permanent Delegate, 
        // invoke(
        //     &initialize_permanent_delegate(
        //         &self.token_2022_program.key(),
        //         &self.solai.key(),
        //         &self.buyer.key(),
        //     )?,
        //     &vec![
        //         self.solai.to_account_info(),
        //     ],
        // )?;
        
        // 2.2: Transfer Hook,
        // invoke(
        //     &intialize_transfer_hook(
        //         &self.token_2022_program.key(),
        //         &self.solai.key(),
        //         Some(self.auth.key()),
        //         None, 
        //     )?,
        //     &vec![
        //         self.solai.to_account_info(),
        //     ],
        // )?;
        
        // 2.3: Close Mint Authority, 
        // invoke(
        //     &initialize_mint_close_authority(
        //         &self.token_2022_program.key(),
        //         &self.solai.key(),
        //         Some(&self.auth.key()),
        //     )?,
        //     &vec![
        //         self.solai.to_account_info(),
        //     ],
        // )?;
        
        // 2.4: Metadata Pointer
        invoke(
            &initialize_metadata_pointer(
                &self.token_2022_program.key(),
                &self.solai.key(),
                Some(self.auth.key()),
                Some(self.solai.key()),
            )?,
            &vec![
                self.solai.to_account_info(),
            ],
        )?;

        // Step 3: Initialize Mint & Metadata Account
        invoke_signed(
            &initialize_mint2(
                &self.token_2022_program.key(),
                &self.solai.key(),
                &self.auth.key(),
                None,
                0,
            )?,
            &vec![
                self.solai.to_account_info(),
            ],
            signer_seeds
        )?;

        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &initialize_metadata_account(
                &self.token_2022_program.key(),
                &self.solai.key(),
                &self.auth.key(),
                &self.solai.key(),
                &self.auth.key(),
                metadata.name,
                metadata.symbol,
                metadata.uri,
            ),
            &vec![
                self.solai.to_account_info(),
                self.auth.to_account_info(),
            ],
            signer_seeds
        )?;

        for (field, value) in metadata.additional_metadata.into_iter() {
            invoke_signed(
                &update_metadata_account(
                    &self.token_2022_program.key(),
                    &self.solai.key(),
                    &self.auth.key(),
                    Field::Key(field),
                    value,
                ),
                &vec![
                    self.solai.to_account_info(),
                    self.auth.to_account_info(),
                ],
                signer_seeds
            )?;
        }

        // BUY NFT///////////////////////////////////////////
        let seeds: &[&[u8]; 2] = &[
            b"auth",
            &[bumps.auth],
        ];
        let signer_seeds = &[&seeds[..]];

        require!(
            self.collection.sale_start_time < Clock::get()?.unix_timestamp, 
            BuyingError::NotTimeYet
        );

        // Pay the solai
        let amount_in_lamports = self.collection.price * LAMPORTS_PER_SOL;
        let transfer_instruction = system_instruction::transfer(
            &self.buyer.key(),
            &self.collection.owner.key(),
            amount_in_lamports,
        );
        invoke(
            &transfer_instruction,
            &[
                self.buyer.to_account_info(),
                self.collection_owner.to_account_info(),
            ],
        )?;

        // Initialize ATA
        create(
            CpiContext::new(
                self.token_2022_program.to_account_info(), // NEEDS CHANGE TO ATA PROGRAM
                Create {
                    payer: self.buyer.to_account_info(), // payer
                    associated_token: self.buyer_solai_ata.to_account_info(),
                    authority: self.buyer.to_account_info(), // owner
                    mint: self.solai.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_2022_program.to_account_info(),
                }
            ),
        )?;

        // Mint the solai
        mint_to(
            CpiContext::new_with_signer(
                self.token_2022_program.to_account_info(),
                MintTo {
                    mint: self.solai.to_account_info(),
                    to: self.buyer_solai_ata.to_account_info(),
                    authority: self.auth.to_account_info(),
                },
                signer_seeds
            ),
            1,
        )?;

        self.collection.total_supply += 1;


       Ok(())
    }
}