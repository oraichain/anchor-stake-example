use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, TokenAccount, Token, Transfer},
};
use solana_program::clock::Clock;
use crate::state::StakeInfo;

use crate::error::ErrorCode;
use crate::constant::constants::{STAKE_INFO_SEED,TOKEN_SEED};



pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let stake_info = &mut ctx.accounts.stake_info_account;

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
        to: ctx.accounts.stake_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    transfer(cpi_ctx, stake_amount)?;

    Ok(())
}







#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [STAKE_INFO_SEED, signer.key.as_ref()],
        bump,
        payer = signer,
        space = 8 + std::mem::size_of::<StakeInfo>()
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        seeds = [TOKEN_SEED, signer.key.as_ref()],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = stake_account,
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
