use anchor_lang::prelude::*;

use crate::constant::constants;

pub const STAKE_CONFIG_SIZE: usize = 8 + 1 + 1 + 32 + 4 + 4 + 8;

#[account]
pub struct StakeConfig {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub version: u8,
    /// Owner of the configuration
    pub authority: Pubkey,
    /// after finish bonding curve -> can unbond after bonding_curve_unbonding_period
    pub bonding_curve_unbonding_period: u32,
    /// after max_unbonding_period -> can unbond
    pub max_unbonding_period: u32,
    /// soft cap for token launch
    pub soft_cap: u64,
}

impl StakeConfig {
    /// Seeds are unique to authority/pyth feed/currency mint combinations
    pub fn auth_seeds<'a>(&'a self) -> [&'a [u8]; 3] {
        [
            constants::STAKE_CONFIG_SEED.as_ref(),
            self.authority.as_ref(),
            self.bump.as_ref(),
        ]
    }
}
