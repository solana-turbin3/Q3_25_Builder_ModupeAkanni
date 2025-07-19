use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_x: Account<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_y: Account<'info, Mint>,

    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_x: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_y: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);
        require!(amount_in > 0, AmmError::InvalidAmount);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            None,
        )
        .map_err(AmmError::from)?;

        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let swap_result = curve
            .swap(p, amount_in, min_amount_out)
            .map_err(AmmError::from)?;

        require!(swap_result.deposit != 0, AmmError::InvalidAmount);
        require!(swap_result.withdraw != 0, AmmError::InvalidAmount);

        self.deposit_token(is_x, swap_result.deposit)?;
        self.withdraw_token(!is_x, swap_result.withdraw)?;

        Ok(())
    }

    pub fn deposit_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.user_ata_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_ata_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_context, amount)
    }

    pub fn withdraw_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_ata_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_ata_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_context, amount)
    }
}
