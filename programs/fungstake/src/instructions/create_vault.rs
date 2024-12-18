use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use crate::{
    constant::constants::{STAKE_CONFIG_SEED, VAULT_SEED},
    StakeConfig, Vault, VAULT_SIZE,
};
use solana_program::sysvar::SysvarId;

#[derive(Accounts)]
pub struct CreateVault<'info> {
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
        init,
        seeds = [
            VAULT_SEED,
            stake_config.key().as_ref(),
            reward_currency_mint.key().as_ref()
        ],
        bump,
        space = VAULT_SIZE,
        payer = signer,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = stake_currency_mint,
        associated_token::authority = vault
    )]
    vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CreateVault<'info> {
    pub fn process(&mut self, vault_bump: u8) -> Result<()> {
        let vault = &mut self.vault;
        vault.bump = [vault_bump];
        vault.reward_currency_mint = self.reward_currency_mint.key();
        vault.total_staked = 0;
        vault.end_time = 0;
        vault.reach_tge = false;
        vault.reach_soft_cap = false;
        vault.total_reward = 0;

        Ok(())
    }
}
