use anchor_lang::prelude::*;

use crate::constant::constants;

pub const VAULT_SIZE: usize = 8 + 1 + 1 + 32 + 8 + 8;

#[account]
pub struct Vault {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub version: u8,
    pub vault_config: Pubkey,
    /// total staked
    pub total_staked: u64,
    pub lock_period: u64,
}

impl Vault {
    pub fn auth_seeds<'a>(&'a self, lock_period_bytes: &'a [u8]) -> [&'a [u8]; 4] {
        [
            constants::VAULT_SEED,
            self.vault_config.as_ref(),
            lock_period_bytes,
            self.bump.as_ref(),
        ]
    }
}
