use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Tokens are already staked")]
    IsStaked,
    #[msg("Tokens not staked")]
    NotStaked,
    #[msg("No Tokens to stake")]
    NoTokens,
    #[msg("Vault has been ended")]
    VaultEnded,
    #[msg("The unbonding time is not over yet")]
    UnbondingTimeNotOverYet,
    #[msg("Soft cap reached, but need to wait til TGE. Cannot unstake!")]
    TgeNotYetReached,
}
