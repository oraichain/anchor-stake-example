use crate::{
    constant::constants::STAKE_CONFIG_SEED, state::StakeInfo, utils::token_transfer_with_signer,
    StakeConfig, Vault,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::clock::Clock;

use crate::constant::constants::{STAKE_INFO_SEED, VAULT_SEED};
use crate::error::ErrorCode;

#[derive(Accounts)]
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
            reward_currency_mint.key().as_ref()
        ],
        bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        associated_token::mint = stake_currency_mint,
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
        associated_token::mint = stake_currency_mint,
        associated_token::authority = signer,
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub stake_currency_mint: Account<'info, Mint>,

    pub reward_currency_mint: Account<'info, Mint>,

    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> DeStake<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        let stake_info = &mut self.staker_info;
        let vault = &mut self.vault;
        let vault_config = &self.stake_config.to_account_info();

        if stake_info.stake_amount == 0 {
            return Err(ErrorCode::NotStaked.into());
        }

        let current_timestamp = Clock::get()?.unix_timestamp;
        if current_timestamp < stake_info.unstaked_at_time {
            return Err(ErrorCode::UnbondingTimeNotOverYet.into());
        }

        // if soft cap reached -> everyone needs to stay locked until reaching TGE -> ensure that the vault's softcap is valid
        if vault.end_time > 0 && current_timestamp <= vault.end_time {
            return Err(ErrorCode::TgeNotYetReached.into());
        }

        // eg: stake amount = 9, amount = 10 -> unstake_amount = 9
        let unstake_amount = std::cmp::min(stake_info.stake_amount, amount);

        stake_info.stake_amount -= unstake_amount;
        // if soft cap reached -> don't subtract total stake & snapshot_amount of user
        if vault.end_time == 0 {
            vault.total_staked -= unstake_amount;
            stake_info.snapshot_amount = stake_info.stake_amount;
        }

        // transfer to user
        token_transfer_with_signer(
            self.vault_token_account.to_account_info(),
            vault.to_account_info(),
            self.staker_token_account.to_account_info(),
            &self.token_program,
            &[vault.auth_seeds(&vault_config.key().to_bytes()).as_ref()],
            unstake_amount,
        )?;

        Ok(())
    }
}
