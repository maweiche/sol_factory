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
    instruction::{initialize_mint_close_authority, initialize_mint2},
    extension::metadata_pointer::instruction::initialize as initialize_metadata_pointer
};

pub use spl_token_metadata_interface::{
    state::{TokenMetadata, Field},
    instruction::{initialize as initialize_metadata_account, update_field as update_metadata_account},
};


pub use crate::state::{Protocol, Collection, Admin, AiNft, Attributes};
pub use crate::errors::ProtocolError;

#[derive(Accounts)]
#[instruction(id: u64, uri: String, name: String, attributes: Vec<Attributes>)]
pub struct CreateNft<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
    
    #[account(
        seeds = [b"collection", collection.owner.key().as_ref()],
        bump,
    )] 
    pub collection: Account<'info, Collection>,
    #[account(
        init,
        payer = admin,
        seeds = [b"ainft", collection.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        space = AiNft::INIT_SPACE + attributes.iter().map(|attr| attr.key.len() + attr.value.len()).sum::<usize>(),
    )] 
    pub nft: Account<'info, AiNft>,

    /// CHECK: this is fine since we are handling all the checks and creation in the program.
    #[account(
        mut,
        seeds = [b"mint", nft.key().as_ref()],
        bump
    )]
    pub mint: UncheckedAccount<'info>,

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
    #[account(
        seeds = [b"protocol"],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateNft<'info> {
    pub fn create(
        &mut self,
        id: u64,
        uri: String,
        name: String,
        attributes: Vec<Attributes>,
        bumps: CreateNftBumps,
    ) -> Result<()> {

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);

        self.nft.set_inner(
            AiNft {
                id,
                collection: self.collection.key(),
                reference: self.collection.reference.to_string(),
                price: self.collection.price,
                time_stamp: Clock::get()?.unix_timestamp,
                inscription: "none".to_string(),
                rank: self.collection.total_supply as u16,
            }
        );

        /* Token2022 Gibberish */

        // Step 1: Initialize Account
        let size = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[
                ExtensionType::MintCloseAuthority,
                // ExtensionType::PermanentDelegate,
                ExtensionType::MetadataPointer,
                // ExtensionType::TransferHook,
            ],
        ).unwrap();

        let metadata = TokenMetadata {
            update_authority: spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(Some(self.auth.key())).unwrap(),
            mint: self.mint.key(),
            name: name.to_string(),
            symbol: self.collection.symbol.to_string(),
            uri,
            additional_metadata: attributes.into_iter().map(|attr| (attr.key, attr.value)).collect(),
        };

        let extension_extra_space = metadata.tlv_size_of().unwrap();
        let rent = &Rent::from_account_info(&self.rent.to_account_info())?;
        let lamports = rent.minimum_balance(size + extension_extra_space);

        let nft_key = self.nft.key();
        let seeds: &[&[u8]; 3] = &[
            b"mint",
            nft_key.as_ref(),
            &[bumps.mint],
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &solana_program::system_instruction::create_account(
                &self.admin.key(),
                &self.mint.key(),
                lamports,
                (size).try_into().unwrap(),
                &spl_token_2022::id(),
            ),
            &vec![
                self.admin.to_account_info(),
                self.mint.to_account_info(),
            ],
            signer_seeds
        )?;
        
        // 2.3: Close Mint Authority, 
        invoke(
            &initialize_mint_close_authority(
                &self.token_2022_program.key(),
                &self.mint.key(),
                Some(&self.auth.key()),
            )?,
            &vec![
                self.mint.to_account_info(),
            ],
        )?;
        
        // 2.4: Metadata Pointer
        invoke(
            &initialize_metadata_pointer(
                &self.token_2022_program.key(),
                &self.mint.key(),
                Some(self.auth.key()),
                Some(self.mint.key()),
            )?,
            &vec![
                self.mint.to_account_info(),
            ],
        )?;

        // Step 3: Initialize Mint & Metadata Account
        invoke_signed(
            &initialize_mint2(
                &self.token_2022_program.key(),
                &self.mint.key(),
                &self.auth.key(),
                None,
                0,
            )?,
            &vec![
                self.mint.to_account_info(),
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
                &self.mint.key(),
                &self.auth.key(),
                &self.mint.key(),
                &self.auth.key(),
                metadata.name,
                metadata.symbol,
                metadata.uri,
            ),
            &vec![
                self.mint.to_account_info(),
                self.auth.to_account_info(),
            ],
            signer_seeds
        )?;

        for (field, value) in metadata.additional_metadata.into_iter() {
            invoke_signed(
                &update_metadata_account(
                    &self.token_2022_program.key(),
                    &self.mint.key(),
                    &self.auth.key(),
                    Field::Key(field),
                    value,
                ),
                &vec![
                    self.mint.to_account_info(),
                    self.auth.to_account_info(),
                ],
                signer_seeds
            )?;
        }

       Ok(())
    }
}