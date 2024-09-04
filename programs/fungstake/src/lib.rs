use anchor_lang::prelude::*;

mod constant;
mod error;
pub mod state;
pub use state::*;
pub mod instructions;
pub use instructions::*;

declare_id!("GnSc9kr2VfWzvck2cZSZSuQzSBfWgNXrKJBb96KH5bQe");

#[program]
pub mod fungstake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::initialize(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake::stake(ctx, amount)
    }

    pub fn destake(ctx: Context<DeStake>) -> Result<()> {
        destake::destake(ctx)
    }
}
