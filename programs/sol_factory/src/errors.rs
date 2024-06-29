use anchor_lang::error_code;

#[error_code]
pub enum SetupError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
}

#[error_code]
pub enum BuyingError {
    #[msg("Listing is not Live yet, come back later!")]
    NotTimeYet,
    #[msg("The collection is sold out!")]
    SoldOut,
    #[msg("You are not in the Whitelist!")]
    NotInWhitelist,
}
#[error_code]
pub enum ProtocolError {
    #[msg("The Protocol is locked, you can't perform this action")]
    ProtocolLocked,
    #[msg("You are not authorized to perform this action")]
    UnauthorizedAdmin,
}