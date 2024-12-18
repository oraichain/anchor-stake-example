use anchor_lang::prelude::*;

mod constant;
mod error;
pub mod state;
pub use state::*;
pub mod instructions;
pub use instructions::*;
pub mod utils;

declare_id!("5VgFt7VaM9eMchXbhLmepFvgwVBniab4PUYBskcYesML");

#[program]
pub mod fungstake {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        lock_period: u32,
        lock_extend_time: u32,
        soft_cap: u64,
    ) -> Result<()> {
        ctx.accounts.process(
            lock_period,
            lock_extend_time,
            soft_cap,
            ctx.bumps.stake_config,
        )
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    pub fn destake(ctx: Context<DeStake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        ctx.accounts.process()
    }
}
