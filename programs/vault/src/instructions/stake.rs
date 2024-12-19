use crate::{
    constant::constants::{STAKER_INFO_SEED, STAKE_CONFIG_SEED, STAKE_DETAIL_SEED, VAULT_SEED},
    state::StakerInfo,
    utils::token_transfer_user,
    StakeConfig, StakeDetail, Vault, STAKER_INFO_SIZE, STAKE_DETAIL_SIZE,
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::clock::Clock;

use crate::error::ErrorCode;

#[derive(Accounts)]
/// CHECK: has to use lock_period & current_staker_id as arguments to avoid maximum account depth stack err (prob related to recursion)
#[instruction(lock_period: u64)]
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
            &lock_period.to_le_bytes()
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
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        seeds = [STAKER_INFO_SEED, vault.key().as_ref(), signer.key.as_ref()],
        bump,
        payer = signer,
        space = STAKER_INFO_SIZE,
    )]
    pub staker_info_pda: Box<Account<'info, StakerInfo>>,

    #[account(
        init,
        seeds = [STAKE_DETAIL_SEED, staker_info_pda.key().as_ref(), &(staker_info_pda.current_id + 1).to_le_bytes()],
        bump,
        payer = signer,
        space = STAKE_DETAIL_SIZE
    )]
    pub stake_detail_pda: Box<Account<'info, StakeDetail>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    // CHECK: the SPL token for staking, not rewarding
    pub stake_currency_mint: Box<Account<'info, Mint>>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Stake<'info> {
    pub fn process(&mut self, _: u64, amount: u64) -> Result<()> {
        let staker_info = &mut self.staker_info_pda;
        let vault = &mut self.vault;
        let stake_detail = &mut self.stake_detail_pda;
        if amount <= 0 {
            return Err(ErrorCode::NoTokens.into());
        }

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        // update stake detail
        stake_detail.unstaked_at_time = current_timestamp + vault.lock_period as i64;
        stake_detail.stake_amount = amount;
        stake_detail.id = staker_info.current_id + 1;
        stake_detail.staker = self.signer.key();

        // update staker info
        staker_info.total_stake += amount;
        staker_info.current_id += 1;

        // update vault
        vault.total_staked += amount;

        // transfer(cpi_ctx, stake_amount)?;
        token_transfer_user(
            self.user_token_account.to_account_info(),
            &self.signer,
            self.vault_token_account.to_account_info(),
            &self.token_program,
            amount,
        )?;

        Ok(())
    }
}
