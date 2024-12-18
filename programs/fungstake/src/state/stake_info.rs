use anchor_lang::prelude::*;

pub const STAKE_INFO_SIZE: usize = 8 + 1 + 8 + 8 + 8 + 1;

#[account]
pub struct StakeInfo {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub unstaked_at_time: i64,
    pub stake_amount: u64,
    pub snapshot_amount: u64,
    /// check if user has claimed the rewards
    pub has_claimed: bool,
}
