use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Expired")]
    Expired,
    #[msg("Already executed")]
    AlreadyExecuted,
    #[msg("Remaining Account's Wrong")]
    RemainingAccountsWrong,
    #[msg("Mint Account Wrong in Remaining Accounts")]
    MintAccountWrong,
    #[msg("Owner and Mint Does not Matches")]
    MisMatchMintOwner,
    #[msg("Token Account Not owner by OTC Signer")]
    MisMatchOTCVaultOwner,
    #[msg("Token Account not owner by Seller")]
    MisMatchBuyerOwner,
    #[msg("Length MisMatch")]
    LengthMisMatch,
}
