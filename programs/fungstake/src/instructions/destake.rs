use crate::{state::StakeInfo, utils::token_transfer_with_signer, Vault};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::clock::Clock;

use crate::constant::constants::{STAKE_INFO_SEED, VAULT_SEED};
use crate::error::ErrorCode;

pub fn destake(ctx: Context<DeStake>) -> Result<()> {
    let stake_info = &mut ctx.accounts.staker_info;
    let vault = &mut ctx.accounts.vault;
    let current_mint = ctx.accounts.currency_mint.to_account_info();

    if stake_info.stake_amount == 0 {
        return Err(ErrorCode::NotStaked.into());
    }

    let current_timestamp = Clock::get()?.unix_timestamp;
    if current_timestamp < stake_info.un_staked_at_time {
        return Err(ErrorCode::UnbondingTimeNotOverYet.into());
    }

    let stake_amount = stake_info.stake_amount;
    stake_info.stake_amount = 0;
    // after locked time, we will not decrease totalStaked
    if vault.end_time > 0 && current_timestamp > vault.end_time {
        vault.total_staked -= stake_amount;
    }

    let current_mint_key = current_mint.key();
    let vault_signer_seeds: &[&[&[u8]]] =
        &[&[VAULT_SEED, &current_mint_key.as_ref(), &[ctx.bumps.vault]]];

    // transfer to user
    token_transfer_with_signer(
        ctx.accounts.vault_token_account.to_account_info(),
        vault.to_account_info(),
        ctx.accounts.staker_token_account.to_account_info(),
        &ctx.accounts.token_program,
        vault_signer_seeds,
        stake_amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DeStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

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
        mut,
        seeds = [STAKE_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
    )]
    pub staker_info: Account<'info, StakeInfo>,

    #[account(
        mut,
        associated_token::mint = currency_mint,
        associated_token::authority = signer,
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub currency_mint: Account<'info, Mint>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}
