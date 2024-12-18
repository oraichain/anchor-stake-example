use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    state::StakeInfo,
    utils::token_transfer_user,
    StakeConfig, Vault, STAKE_INFO_SIZE,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::clock::Clock;

use crate::constant::constants::STAKE_INFO_SEED;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Stake<'info> {
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

    /// CHECK: staking ATA of vault
    #[account(
        mut,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = vault
    )]
    pub vault_staking_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds = [STAKE_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
        payer = signer,
        space = STAKE_INFO_SIZE
    )]
    pub user_stake_info_pda: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: the SPL token for rewarding, not staking
    pub reward_currency_mint: Account<'info, Mint>,

    // CHECK: the SPL token for staking, not rewarding
    pub stake_currency_mint: Account<'info, Mint>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Stake<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        let stake_info = &mut self.user_stake_info_pda;
        let vault = &mut self.vault;
        let stake_config = &mut self.stake_config;

        if amount <= 0 {
            return Err(ErrorCode::NoTokens.into());
        }

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        if vault.end_time > 0 && current_timestamp > vault.end_time {
            return Err(ErrorCode::VaultEnded.into());
        }

        stake_info.unstaked_at_time = current_timestamp + stake_config.lock_period as i64;
        stake_info.stake_amount += amount;
        stake_info.snapshot_amount = stake_info.stake_amount;

        vault.total_staked += amount;
        // check reach soft cap. Only update end_time one time
        if !vault.reach_soft_cap && vault.total_staked >= stake_config.soft_cap {
            vault.end_time = current_timestamp + stake_config.lock_extend_time as i64;
            vault.reach_soft_cap = true;
        }

        // transfer(cpi_ctx, stake_amount)?;
        token_transfer_user(
            self.user_token_account.to_account_info(),
            &self.signer,
            self.vault_staking_token_account.to_account_info(),
            &self.token_program,
            amount,
        )?;

        Ok(())
    }
}
