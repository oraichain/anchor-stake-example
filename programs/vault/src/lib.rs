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
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.process(ctx.bumps.stake_config)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    pub fn destake(ctx: Context<DeStake>, id: u64, amount: u64) -> Result<()> {
        ctx.accounts.process(id, amount)
    }

    pub fn create_vault(ctx: Context<CreateVault>, lock_period: u64) -> Result<()> {
        ctx.accounts.process(lock_period, ctx.bumps.vault)
    }
}