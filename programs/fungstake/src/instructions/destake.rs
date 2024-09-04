use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, TokenAccount, Token, Transfer},
};
use solana_program::clock::Clock;
use crate::state::StakeInfo;

use crate::error::ErrorCode;
use crate::constant::constants::{STAKE_INFO_SEED,TOKEN_SEED,VAULT_SEED};



pub fn destake(ctx: Context<DeStake>) -> Result<()> {
    let stake_info = &mut ctx.accounts.stake_info_account;

    if !stake_info.is_staked {
        return Err(ErrorCode::NotStaked.into());
    }

    let clock = Clock::get()?;

    let slot_passed = clock.slot - stake_info.staked_at_slot;

    let stake_amount = ctx.accounts.stake_account.amount;

    let reward = slot_passed as u64;

    let bump_for_vault = ctx.bumps.token_vault_account;

    let signer_seeds_for_reward: &[&[&[u8]]] = &[&[VAULT_SEED, &[bump_for_vault]]];

    let transfer_from_vault_accounts = Transfer {
        from: ctx.accounts.token_vault_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.token_vault_account.to_account_info(),
    };

    let ctxx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_from_vault_accounts,
        signer_seeds_for_reward,
    );

    transfer(ctxx, reward)?;

    let staker = ctx.accounts.signer.key();

    let bump_for_stake_account = ctx.bumps.stake_account;

    let signer_seeds_for_user_stake: &[&[&[u8]]] = &[&[
        TOKEN_SEED,
        staker.as_ref(),
        &[bump_for_stake_account],
    ]];

    let transfer_from_stake_accounts = Transfer {
        from: ctx.accounts.stake_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.stake_account.to_account_info(),
    };

    let ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_from_stake_accounts,
        signer_seeds_for_user_stake,
    );

    transfer(ctx, stake_amount)?;

    stake_info.is_staked = false;
    stake_info.staked_at_slot = clock.slot;

    Ok(())
}


#[derive(Accounts)]
pub struct DeStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [STAKE_INFO_SEED, signer.key.as_ref()],
        bump,
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        seeds = [TOKEN_SEED, signer.key.as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, TokenAccount>,

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