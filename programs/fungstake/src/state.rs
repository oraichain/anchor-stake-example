use anchor_lang::prelude::*;

#[account]
pub struct StakeInfo {
    pub staked_at_slot: u64,
    pub is_staked: bool,
}