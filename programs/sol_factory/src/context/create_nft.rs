pub use anchor_lang::{
    solana_program::{
        sysvar::rent::ID as RENT_ID,
        program::{invoke, invoke_signed}
    },
    prelude::*
};

pub use anchor_spl::token_2022::Token2022;

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


pub use crate::state::{Nft, Collection, Admin};

#[derive(Accounts)]
#[instruction(id: u64, reference: String, collection: Pubkey, image: String, seed: u64, model_name: String, model_hash: String)]
pub struct CreateNft<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
    
    #[account(
        seeds = [b"collection", collection.reference.as_bytes().as_ref()],
        bump,
    )] 
    pub collection: Account<'info, Collection>,
    #[account(
        init,
        payer = admin,
        seeds = [b"nft", collection.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        space = Nft::INIT_SPACE + collection.reference.len() + image.len() + model_name.len() + model_hash.len(),
    )] 
    pub nft: Account<'info, Nft>,

    /// CHECK: this is fine since we are handling all the checks and creation in the program.
    #[account(
        mut,
        seeds = [b"genai", nft.key().as_ref()],
        bump
    )]
    pub genai: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        seeds = [b"auth"],
        bump
    )]
    pub auth: UncheckedAccount<'info>,

    #[account(address = RENT_ID)]
    /// CHECK: this is fine since we are hard coding the rent sysvar.
    pub rent: UncheckedAccount<'info>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateNft<'info> {
    pub fn create(
        &mut self,
        id: u64,
        reference: String,
        uri: String,
        image: String,
        seed: u64,
        model_name: String,
        model_hash: String,
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
                ExtensionType::MintCloseAuthority,
                ExtensionType::PermanentDelegate,
                ExtensionType::MetadataPointer,
                ExtensionType::TransferHook,
            ],
        ).unwrap();

        let metadata = TokenMetadata {
            update_authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(self.auth.key())).unwrap(),
            mint: self.genai.key(),
            name: "Sol AI Artwork".to_string(),
            symbol: "SOLAI".to_string(),
            uri,
            additional_metadata: vec![
                ("season".to_string(), "winter".to_string()),
                ("camera angle".to_string(), "low angle".to_string()),
                ("theme".to_string(), "nichijo".to_string()),
                ("seed".to_string(), "3808958`1".to_string()),
                ("seed".to_string(), seed.to_string())
            ]
        };

        let extension_extra_space = metadata.tlv_size_of().unwrap();
        let rent = &Rent::from_account_info(&self.rent.to_account_info())?;
        let lamports = rent.minimum_balance(size + extension_extra_space);

        let nft_key = self.nft.key();
        let seeds: &[&[u8]; 3] = &[
            b"genai",
            nft_key.as_ref(),
            &[bumps.genai],
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &solana_program::system_instruction::create_account(
                &self.admin.key(),
                &self.genai.key(),
                lamports,
                (size).try_into().unwrap(),
                &spl_token_2022::id(),
            ),
            &vec![
                self.admin.to_account_info(),
                self.genai.to_account_info(),
            ],
            signer_seeds
        )?;

        // Step 2: Initialize Extension needed: 

        // 2.1: Permanent Delegate, 
        invoke(
            &initialize_permanent_delegate(
                &self.token_2022_program.key(),
                &self.genai.key(),
                &self.auth.key(),
            )?,
            &vec![
                self.genai.to_account_info(),
            ],
        )?;
        
        // 2.2: Transfer Hook,
        invoke(
            &intialize_transfer_hook(
                &self.token_2022_program.key(),
                &self.genai.key(),
                Some(self.auth.key()),
                None, 
            )?,
            &vec![
                self.genai.to_account_info(),
            ],
        )?;
        
        // 2.3: Close Mint Authority, 
        invoke(
            &initialize_mint_close_authority(
                &self.token_2022_program.key(),
                &self.genai.key(),
                Some(&self.auth.key()),
            )?,
            &vec![
                self.genai.to_account_info(),
            ],
        )?;
        
        // 2.4: Metadata Pointer
        invoke(
            &initialize_metadata_pointer(
                &self.token_2022_program.key(),
                &self.genai.key(),
                Some(self.auth.key()),
                Some(self.genai.key()),
            )?,
            &vec![
                self.genai.to_account_info(),
            ],
        )?;

        // Step 3: Initialize Mint & Metadata Account
        invoke_signed(
            &initialize_mint2(
                &self.token_2022_program.key(),
                &self.genai.key(),
                &self.auth.key(),
                None,
                0,
            )?,
            &vec![
                self.genai.to_account_info(),
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
                &self.genai.key(),
                &self.auth.key(),
                &self.genai.key(),
                &self.auth.key(),
                metadata.name,
                metadata.symbol,
                metadata.uri,
            ),
            &vec![
                self.genai.to_account_info(),
                self.auth.to_account_info(),
            ],
            signer_seeds
        )?;

        for (field, value) in metadata.additional_metadata.into_iter() {
            invoke_signed(
                &update_metadata_account(
                    &self.token_2022_program.key(),
                    &self.genai.key(),
                    &self.auth.key(),
                    Field::Key(field),
                    value,
                ),
                &vec![
                    self.genai.to_account_info(),
                    self.auth.to_account_info(),
                ],
                signer_seeds
            )?;
        }
        
       Ok(())
    }
}