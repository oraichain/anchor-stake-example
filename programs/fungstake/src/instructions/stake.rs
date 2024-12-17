use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    state::StakeInfo,
    utils::token_transfer_user,
    StakeConfig, Vault, STAKE_INFO_SIZE,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use solana_program::clock::Clock;

use crate::constant::constants::STAKE_INFO_SEED;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [STAKE_CONFIG_SEED],
        bump,
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED,
            currency_mint.key().as_ref()
        ],
        bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        associated_token::mint = currency_mint,
        associated_token::authority = vault
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds = [STAKE_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
        payer = signer,
        space = STAKE_INFO_SIZE
    )]
    pub user_stake_info_pda: Account<'info, StakeInfo>,

    #[account(
        mut,
        associated_token::mint = currency_mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub currency_mint: Account<'info, Mint>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let stake_info = &mut ctx.accounts.user_stake_info_pda;
    let vault = &mut ctx.accounts.vault;
    let stake_config = &mut ctx.accounts.stake_config;

    if amount <= 0 {
        return Err(ErrorCode::NoTokens.into());
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if vault.end_time > 0 && current_timestamp > vault.end_time {
        return Err(ErrorCode::VaultEnded.into());
    }

    stake_info.un_staked_at_time =
        current_timestamp + stake_config.bonding_curve_unbonding_period as i64;
    stake_info.stake_amount += amount;
    vault.total_staked += amount;
    // check reach soft cap
    if vault.total_staked >= stake_config.soft_cap {
        vault.end_time = current_timestamp + stake_config.lock_extend_time as i64;
    }

    // transfer(cpi_ctx, stake_amount)?;
    token_transfer_user(
        ctx.accounts.user_token_account.to_account_info(),
        &ctx.accounts.signer,
        ctx.accounts.vault_token_account.to_account_info(),
        &ctx.accounts.token_program,
        amount,
    )?;

    Ok(())
}
