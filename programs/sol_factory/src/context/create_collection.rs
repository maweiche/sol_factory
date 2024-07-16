use anchor_lang::{
    solana_program::{
        sysvar::rent::ID as RENT_ID,
        program::{invoke, invoke_signed}
    },
    prelude::*
};
pub use anchor_spl::token_2022::Token2022;
use crate::state::{Collection, Protocol, Admin};
use crate::errors::ProtocolError;
pub use spl_token_2022::{
    extension::ExtensionType,
    extension::group_pointer::instruction::initialize as initialize_group_pointer,
};

#[derive(Accounts)]
#[instruction(
    reference: Pubkey,
    name: String,
    symbol: String,
    url: String,
    sale_start_time: i64,
    sale_end_time: i64,
    max_supply: u64,
    price: f32,
    stable_id: String,
)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: this is ok because admin is setting up on owner behalf
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(
        init,
        seeds = [b"collection", owner.key().as_ref()],
        bump,
        payer = admin,
        space = Collection::INIT_SPACE + 54 + url.len() + name.len() + stable_id.len(),
    )] 
    pub collection: Account<'info, Collection>,
    #[account(
        seeds = [b"admin_state", admin.key().as_ref()],
        bump
    )]
    pub admin_state: Account<'info, Admin>,
    /// CHECK: this is fine since we are handling all the checks and creation in the program.
    #[account(
        mut,
        seeds = [b"mint", collection.key().as_ref()],
        bump
    )]
    pub mint: UncheckedAccount<'info>,
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

impl<'info> CreateCollection<'info> {
    pub fn create(
        &mut self,
        reference: Pubkey,
        name: String,
        symbol: String,
        url: String,
        sale_start_time: i64,
        sale_end_time: i64,
        max_supply: u64,
        price: f32,
        stable_id: String,
        bumps: CreateCollectionBumps,
    ) -> Result<()> {

        /*
        
            Create Collection Ix:

            Some security check:
            - The admin_state.publickey must match the signing admin.

            What these Instructions do:
            - Creates a Collection that can be used to mint NFTs.
        */

        require!(!self.protocol.locked, ProtocolError::ProtocolLocked);
        require!(self.admin_state.publickey == *self.admin.key, ProtocolError::UnauthorizedAdmin);

        // sanity check

        require!(sale_start_time < sale_end_time, ProtocolError::InvalidSaleTime);
        require!(sale_start_time > 0, ProtocolError::InvalidSaleTime);
        require!(sale_end_time > 0, ProtocolError::InvalidSaleTime);
        require!(max_supply > 0, ProtocolError::InvalidMaxSupply);
        require!(price >= 0.0, ProtocolError::InvalidPrice);

        
        // msg!("Sale start time is {}", sale_start_time);
        // msg!("Sale end time is {}", sale_end_time);
        // msg!("Current time is {}", Clock::get()?.unix_timestamp);


        self.collection.set_inner(
            Collection {
                reference,
                name,
                symbol,
                owner: *self.owner.key,
                url,
                sale_start_time,
                sale_end_time,
                max_supply,
                total_supply: 0,
                mint_count: 0,
                price,
                stable_id,
            }
        );

        // Step 1: Initialize Account
        let size = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[
                ExtensionType::GroupPointer
            ],
        ).unwrap();

        let rent = &Rent::from_account_info(&self.rent.to_account_info())?;
        let lamports = rent.minimum_balance(size );

        let collection_key = self.collection.key();
        let seeds: &[&[u8]; 3] = &[
            b"mint",
            collection_key.as_ref(),
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

        invoke(
            &initialize_group_pointer(
                &self.token_2022_program.key(),
                &self.mint.key(),
                Some(self.admin.key()),
                Some(self.mint.key()),
            )?,
            &vec![
                self.mint.to_account_info(),
            ],  
        )?;

        Ok(())
    }

}

// add_sale_start_time