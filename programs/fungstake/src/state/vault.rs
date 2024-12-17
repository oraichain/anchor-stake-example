use anchor_lang::prelude::*;

use crate::constant::constants;

pub const VAULT_SIZE: usize = 8 + 1 + 1 + 8 + 8 + 1;

#[account]
pub struct Vault {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub version: u8,
    /// SPL token mint or native mint for stake
    pub currency_mint: Pubkey,
    /// total staked
    pub total_staked: u64,
    // after this time, user cannot stake
    pub end_time: i64,
    // reached threshold
    pub reach_soft_cap: bool,
}

impl Vault {
    /// Seeds are unique to authority/pyth feed/currency mint combinations
    pub fn auth_seeds<'a>(&'a self) -> [&'a [u8]; 3] {
        [
            constants::VAULT_SEED.as_ref(),
            self.currency_mint.as_ref(),
            self.bump.as_ref(),
        ]
    }
}
