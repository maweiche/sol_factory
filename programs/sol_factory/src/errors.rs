use anchor_lang::error_code;

#[error_code]
pub enum SetupError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("You are already verified!")]
    ProfileAlreadyVerified,
    #[msg("You passed in the wrong Membership Type!")]
    InvalidMembershipType,
    #[msg("You used an invalid condition")]
    InvalidCondition,
}

#[error_code]
pub enum BuyingError {
    #[msg("Total supply is already at max!")]
    MaxSupplyReached,
    #[msg("Collection is not Live yet, come back later!")]
    NotTimeYet
}