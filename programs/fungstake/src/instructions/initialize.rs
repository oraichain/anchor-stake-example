use crate::{constant::constants::STAKE_CONFIG_SEED, StakeConfig, STAKE_CONFIG_SIZE};
use anchor_lang::prelude::*;

use anchor_spl::token::{Mint, Token};
use solana_program::sysvar::SysvarId;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        seeds = [STAKE_CONFIG_SEED, stake_currency_mint.key().as_ref()],
        bump,
        space = STAKE_CONFIG_SIZE,
        payer = signer
    )]
    pub stake_config: Box<Account<'info, StakeConfig>>,

    #[account(
        mint::token_program = token_program
    )]
    pub stake_currency_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn process(
        &mut self,
        lock_period: u32,
        lock_extend_time: u32,
        soft_cap: u64,
        bump: u8,
    ) -> Result<()> {
        let stake_config = &mut self.stake_config;
        stake_config.authority = self.signer.to_account_info().key();
        stake_config.stake_currency_mint = self.stake_currency_mint.to_account_info().key();
        stake_config.bump = [bump];
        stake_config.lock_period = lock_period;
        stake_config.version = 1;
        stake_config.soft_cap = soft_cap;
        stake_config.lock_extend_time = lock_extend_time;

        Ok(())
    }
}
