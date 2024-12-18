use anchor_lang::prelude::*;

use crate::constant::constants;

pub const VAULT_SIZE: usize = 8 + 1 + 1 + 32 + 8 + 8 + 1 + 8 + 1;

#[account]
pub struct Vault {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub version: u8,
    /// SPL token mint or native mint for claiming reward. This is not MAX!
    pub reward_currency_mint: Pubkey,
    /// total staked
    pub total_staked: u64,
    // after this time, user cannot stake
    pub end_time: i64,
    // reached threshold
    pub reach_soft_cap: bool,
    /// total reward
    pub total_reward: u64,
    pub reach_tge: bool,
}

impl Vault {
    /// Seeds are unique to authority/pyth feed/currency mint combinations
    pub fn auth_seeds<'a>(&'a self, vault_config: &'a [u8]) -> [&'a [u8]; 4] {
        [
            constants::VAULT_SEED,
            vault_config,
            self.reward_currency_mint.as_ref(),
            self.bump.as_ref(),
        ]
    }
}
