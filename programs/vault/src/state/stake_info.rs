use anchor_lang::prelude::*;

pub const STAKER_INFO_SIZE: usize = 8 + 1 + 8 + 8;

#[account]
pub struct StakerInfo {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub total_stake: u64,
    pub current_id: u64,
}

pub const STAKE_DETAIL_SIZE: usize = 8 + 1 + 8 + 8 + 8 + 32;
#[account]
pub struct StakeDetail {
    /// Bump seed used to generate the program address / authority
    pub bump: [u8; 1],
    pub id: u64,
    pub stake_amount: u64,
    pub unstaked_at_time: i64,
    pub staker: Pubkey,
}
