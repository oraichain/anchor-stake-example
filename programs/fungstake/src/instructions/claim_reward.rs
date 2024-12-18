use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use crate::{
    constant::constants::STAKE_INFO_SEED, error::ErrorCode, utils::token_transfer_with_signer,
    StakeInfo,
};
use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    StakeConfig, Vault,
};
use solana_program::sysvar::SysvarId;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [STAKE_CONFIG_SEED, stake_currency_mint.key().as_ref()],
        bump,
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    /// CHECK: currency_mint for rewarding, not staking
    pub reward_currency_mint: Account<'info, Mint>,

    /// CHECK: currency_mint for rewarding, not staking
    pub stake_currency_mint: Account<'info, Mint>,

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

    /// CHECK: vault_reward_token_account should be init by the launchpad program
    #[account(
        mut,
        associated_token::mint = reward_currency_mint,
        associated_token::authority = vault
    )]
    vault_reward_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: pda should be init when stake
    #[account(
        seeds = [STAKE_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
    )]
    pub user_stake_info_pda: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = reward_currency_mint,
        associated_token::authority = signer,
    )]
    pub user_reward_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimReward<'info> {
    pub fn process(&mut self) -> Result<()> {
        let vault = &mut self.vault;
        let vault_config = &self.stake_config.to_account_info();

        if self.user_stake_info_pda.has_claimed {
            return Err(ErrorCode::AlreadyClaimed.into());
        }

        if vault.end_time == 0 {
            return Err(ErrorCode::VaultNotStarted.into());
        }

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        // can only claim reward after tge
        if current_timestamp <= vault.end_time {
            return Err(ErrorCode::TgeNotYetReached.into());
        }

        // For the first user who claims the reward, update vault's total reward
        if !vault.reach_tge {
            // this means relayer has not invoked tge yet, can't claim
            if self.vault_reward_token_account.amount == 0 {
                return Err(ErrorCode::TgeNotYetReached.into());
            }

            vault.reach_tge = true;
            // update vault's reward balance
            vault.total_reward = self.vault_reward_token_account.amount;
        }

        let earned_amount = get_earned_amount(
            self.user_stake_info_pda.snapshot_amount,
            vault.total_staked,
            vault.total_reward,
        )?;

        self.user_stake_info_pda.has_claimed = true;

        token_transfer_with_signer(
            self.vault_reward_token_account.to_account_info(),
            vault.to_account_info(),
            self.user_reward_token_account.to_account_info(),
            &self.token_program,
            &[&vault.auth_seeds(&vault_config.key().to_bytes())],
            earned_amount,
        )?;

        Ok(())
    }
}

fn get_earned_amount(
    staked_amount: u64,
    total_staked_amount: u64,
    total_reward: u64,
) -> Result<u64> {
    // Divide the losing pool by winning for earnings multiplier
    Ok((staked_amount as u128)
        .checked_mul(total_reward as u128)
        .ok_or::<ErrorCode>(ErrorCode::OverflowError.into())?
        .checked_div(total_staked_amount as u128)
        .ok_or::<ErrorCode>(ErrorCode::OverflowError.into())? as u64)
}
