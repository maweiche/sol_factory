use anchor_lang::declare_id;

pub mod multisig_wallet {
    use super::*;
    declare_id!("6KuX26FZqzqpsHDLfkXoBXbQRPEDEbstqNiPBKHNJQ9e");
}

pub mod admin_wallet {
    use super::*;
    declare_id!("DEVJb1nq3caksGybAFxoxsYXLi9nyp8ZQnmAFmfAYMSN");
}

pub const ED25519_PROGRAM_ID: &str = "Ed25519SigVerify111111111111111111111111111";

pub const ADMIN_FEE: u64 = 100000000; // 0.3 SOL
// pub const ADMIN_PERCENTAGE: f32 = 0.3;