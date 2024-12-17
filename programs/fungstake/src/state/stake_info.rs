use anchor_lang::prelude::*;

pub const STAKE_INFO_SIZE: usize = 8 + 1 + 8 + 8 + 1;

#[account]
pub struct StakeInfo {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub staked_at_slot: u64,
    pub stake_amount: u64,
    pub is_staked: bool,
}
