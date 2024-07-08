use anchor_lang::declare_id;
use solana_program::native_token::LAMPORTS_PER_SOL;

pub mod multisig_wallet {
    use super::*;
    declare_id!("6KuX26FZqzqpsHDLfkXoBXbQRPEDEbstqNiPBKHNJQ9e");
}

pub const ED25519_PROGRAM_ID: &str = "Ed25519SigVerify111111111111111111111111111";

pub const ADMIN_FEE: u64 = LAMPORTS_PER_SOL / 50; // 0.02 SOL
pub const ADMIN_PERCENTAGE: f32 = 0.3;