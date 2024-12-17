use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    StakeConfig, STAKE_CONFIG_SIZE,
};
use anchor_lang::prelude::*;

use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::sysvar::SysvarId;

pub fn initialize(
    ctx: Context<Initialize>,
    bonding_curve_unbonding_period: u32,
    max_unbonding_period: u32,
    soft_cap: u64,
) -> Result<()> {
    let stake_config = &mut ctx.accounts.stake_config;
    stake_config.authority = ctx.accounts.signer.to_account_info().key();
    stake_config.bump = [ctx.bumps.stake_config];
    stake_config.bonding_curve_unbonding_period = bonding_curve_unbonding_period;
    stake_config.max_unbonding_period = max_unbonding_period;
    stake_config.version = 1;
    stake_config.soft_cap = soft_cap;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        seeds = [STAKE_CONFIG_SEED],
        bump,
        space = STAKE_CONFIG_SIZE,
        payer = signer
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    #[account(
        init_if_needed,
        seeds = [VAULT_SEED, stake_config.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,
}
