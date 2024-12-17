use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    state::StakeInfo,
    StakeConfig, STAKE_CONFIG_SIZE, STAKE_INFO_SIZE,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use solana_program::clock::Clock;

use crate::constant::constants::{STAKE_INFO_SEED, TOKEN_SEED};
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
        init_if_needed,
        seeds = [VAULT_SEED, stake_config.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds = [STAKE_INFO_SEED, stake_config.key().as_ref(), mint.key().as_ref(), signer.key.as_ref()],
        bump,
        payer = signer,
        space = STAKE_INFO_SIZE
    )]
    pub user_stake_info_pda: Account<'info, StakeInfo>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let stake_info = &mut ctx.accounts.user_stake_info_pda;

    if stake_info.is_staked {
        return Err(ErrorCode::IsStaked.into());
    }

    if amount <= 0 {
        return Err(ErrorCode::NoTokens.into());
    }

    let clock = Clock::get()?;

    stake_info.staked_at_slot = clock.slot;
    stake_info.is_staked = true;

    let stake_amount = amount;

    let transfer_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.user_stake_info_pda.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    transfer(cpi_ctx, stake_amount)?;

    Ok(())
}
