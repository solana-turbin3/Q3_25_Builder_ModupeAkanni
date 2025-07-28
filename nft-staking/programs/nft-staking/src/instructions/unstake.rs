use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        mpl_token_metadata::instructions::{
            ThawDelegatedAccountCpi,
            ThawDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount,
        Metadata,
    },
    token::{ revoke, Mint, Revoke, Token, TokenAccount },
};
use crate::{ errors::NFTStakingError, state::{ StakeAccount, StakeConfig, UserAccount } };

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump
    )]
    pub edition: Account<'info, MasterEditionAccount>,

    #[account(seeds = [b"config".as_ref()], bump = stake_config.bump)]
    pub stake_config: Account<'info, StakeConfig>,

    #[account(
        mut,
        close = user,
        seeds = [b"stake", nft_mint.key().as_ref(), stake_config.key().as_ref()],
        bump = stake_account.bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), user_account.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        // There is no need to check user.amount_staked > 0 since
        // a valid stake account is required in the context and
        // a valid stake account can only be initialized via the stake instruction.

        let time_elasped = ((Clock::get()?.unix_timestamp - self.stake_account.staked_at) /
            86400) as u32;

        require!(
            time_elasped >= self.stake_config.freeze_period,
            NFTStakingError::FreezePeriodNotElaspedError
        );

        // Increase user points
        self.user_account.points += time_elasped * (self.stake_config.points_per_stake as u32);

        // Unfreeze delegated NFT authority
        self.unfreeze_delegated_nft_authority()?;

        // Undelegate NFT authority
        self.undelegate_nft_authority()?;

        self.user_account.amount_staked -= 1;

        Ok(())
    }

    pub fn unfreeze_delegated_nft_authority(&mut self) -> Result<()> {
        let cpi_program = &self.metadata_program.to_account_info();

        let cpi_accounts = ThawDelegatedAccountCpiAccounts {
            delegate: &self.stake_account.to_account_info(),
            token_account: &self.user_nft_ata.to_account_info(),
            edition: &self.edition.to_account_info(),
            mint: &self.nft_mint.to_account_info(),
            token_program: &self.token_program.to_account_info(),
        };

        let seeds = &[
            b"stake",
            self.nft_mint.to_account_info().key.as_ref(),
            self.stake_config.to_account_info().key.as_ref(),
            &[self.stake_account.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        ThawDelegatedAccountCpi::new(&cpi_program, cpi_accounts).invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn undelegate_nft_authority(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Revoke {
            source: self.user_nft_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        revoke(cpi_ctx)?;

        Ok(())
    }
}