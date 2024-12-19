use crate::{constant::constants::STAKE_CONFIG_SEED, StakeConfig, STAKE_CONFIG_SIZE};
use anchor_lang::{prelude::*, system_program};

use anchor_spl::token::{self, Mint, Token};
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
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let stake_config = &mut self.stake_config;
        stake_config.authority = self.signer.to_account_info().key();
        stake_config.stake_currency_mint = self.stake_currency_mint.to_account_info().key();
        stake_config.bump = [bump];
        stake_config.version = 1;

        Ok(())
    }
}
