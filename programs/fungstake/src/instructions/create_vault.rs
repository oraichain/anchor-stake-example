use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
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
        seeds = [STAKE_CONFIG_SEED],
        bump,
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    pub currency_mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [
            VAULT_SEED,
            currency_mint.key().as_ref()
        ],
        bump,
        space = VAULT_SIZE,
        payer = signer,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = currency_mint,
        associated_token::authority = vault
    )]
    vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CreateVault<'info> {
    pub fn process(&mut self) -> Result<()> {
        let vault = &mut self.vault;
        vault.currency_mint = self.currency_mint.key();
        vault.total_staked = 0;

        Ok(())
    }
}
