use anchor_lang::declare_id;

pub mod multisig_wallet {
    use super::*;
    declare_id!("V1S1YNN5qQaufxayw4AJtQTWv5sgj11SeeYCKBtBdSj");
}

pub mod admin_wallet {
    use super::*;
    declare_id!("ADM12HQ5G2EzSwWy2nN1xXMyGjaBULuuX9GTgW2FPwZK");
}

pub const ED25519_PROGRAM_ID: &str = "Ed25519SigVerify111111111111111111111111111";

pub const ADMIN_FEE: u64 = 100000000; // 0.3 SOL
// pub const ADMIN_PERCENTAGE: f32 = 0.3;