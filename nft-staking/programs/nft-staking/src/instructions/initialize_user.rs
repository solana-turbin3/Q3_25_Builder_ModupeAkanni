use anchor_lang::prelude::*;

use anchor_spl::token::{self, Mint, Token};

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"user".as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserAccount::INIT_SPACE,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

impl <'info> InitializeUser <'info> {
    pub fn initialize_config(&mut self, bumps: &InitializeUserBumps) -> Result<()> {
        self.config.set_inner(UserAccount {
            points: 0,
            amount_staked: 0,
            bump: bumps.user_account,
        });

        Ok(())
    }
}
