use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    #[msg("Tokens are already staked")]
    IsStaked,
    #[msg("Tokens not staked")]
    NotStaked,
    #[msg("No Tokens to stake")]
    NoTokens,
}
