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

    pub fn initialize(ctx: Context<Initialize>, lock_period: u32, soft_cap: u64) -> Result<()> {
        initialize::initialize(ctx, lock_period, soft_cap)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake::stake(ctx, amount)
    }

    pub fn destake(ctx: Context<DeStake>) -> Result<()> {
        destake::destake(ctx)
    }

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        ctx.accounts.process()
    }
    
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        ctx.accounts.process()
    }
}
