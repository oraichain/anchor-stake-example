use crate::{
    constant::constants::{STAKE_CONFIG_SEED, STAKE_DETAIL_SEED},
    state::StakerInfo,
    utils::token_transfer_with_signer,
    StakeConfig, StakeDetail, Vault,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::clock::Clock;

use crate::constant::constants::{STAKER_INFO_SEED, VAULT_SEED};
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct DeStake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [STAKE_CONFIG_SEED, stake_currency_mint.key().as_ref()],
        bump,
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED,
            stake_config.key().as_ref(),
            &vault.lock_period.to_le_bytes()
        ],
        bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = vault
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [STAKER_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
    )]
    pub staker_info: Box<Account<'info, StakerInfo>>,

    #[account(
       mut,
        seeds = [STAKE_DETAIL_SEED, staker_info.key().as_ref(), &id.to_le_bytes()],
        bump,
    )]
    pub stake_detail: Box<Account<'info, StakeDetail>>,

    #[account(
        mut,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = signer,
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub stake_currency_mint: Account<'info, Mint>,

    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> DeStake<'info> {
    pub fn process(&mut self, _: u64, amount: u64) -> Result<()> {
        let staker_info = &mut self.staker_info;
        let vault = &mut self.vault;
        let stake_detail = &mut self.stake_detail;

        if stake_detail.stake_amount == 0 {
            return Err(ErrorCode::NotStaked.into());
        }

        let current_timestamp = Clock::get()?.unix_timestamp;
        if current_timestamp < stake_detail.unstaked_at_time {
            return Err(ErrorCode::UnbondingTimeNotOverYet.into());
        }

        // eg: stake amount = 9, amount = 10 -> unstake_amount = 9
        let unstake_amount = std::cmp::min(stake_detail.stake_amount, amount);

        // update stake detail
        stake_detail.stake_amount -= unstake_amount;

        // update staker info
        staker_info.total_stake -= unstake_amount;

        // update vault
        vault.total_staked -= unstake_amount;

        // transfer to user
        token_transfer_with_signer(
            self.vault_token_account.to_account_info(),
            vault.to_account_info(),
            self.staker_token_account.to_account_info(),
            &self.token_program,
            &[&vault.auth_seeds(&vault.lock_period.to_le_bytes())],
            unstake_amount,
        )?;

        Ok(())
    }
}
