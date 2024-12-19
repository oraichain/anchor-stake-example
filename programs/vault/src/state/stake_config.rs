use anchor_lang::prelude::*;

use crate::constant::constants;

pub const STAKE_CONFIG_SIZE: usize = 8 + 1 + 1 + 32 + 32 + 4;

#[account]
pub struct StakeConfig {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub version: u8,
    /// Owner of the configuration
    pub authority: Pubkey,
    /// currency mint of token to stake
    pub stake_currency_mint: Pubkey,
}

impl StakeConfig {
    /// Seeds are unique to authority/pyth feed/currency mint combinations
    pub fn auth_seeds<'a>(&'a self) -> [&'a [u8]; 4] {
        [
            constants::STAKE_CONFIG_SEED.as_ref(),
            self.authority.as_ref(),
            self.stake_currency_mint.as_ref(),
            self.bump.as_ref(),
        ]
    }
}
