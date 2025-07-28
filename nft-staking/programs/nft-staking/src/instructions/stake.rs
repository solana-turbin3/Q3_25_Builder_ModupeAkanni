use anchor_lang::prelude::*;

use anchor_spl::{
    metadata::{
        MasterEditionAccount, Metadata, MetadataAccount,
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts,
        },
    },
    token::{Approve, Mint, Token, TokenAccount, approve},
};

use crate::{
    errors::NFTStakingError,
    state::{StakeAccount, StakeConfig, UserAccount},
};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub nft_mint: Account<'info, Mint>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref()],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().verified == true,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() ==
        collection_mint.key().as_ref()
    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump
    )]
    pub edition: Account<'info, MasterEditionAccount>,

    #[account(seeds = [b"config".as_ref()], bump = stake_config.bump)]
    pub stake_config: Account<'info, StakeConfig>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake", nft_mint.key().as_ref(), stake_config.key().as_ref()],
        bump,
        space = 8 + StakeAccount::INIT_SPACE
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

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(
            self.user_account.amount_staked < self.stake_config.max_stake,
            NFTStakingError::MaxStakeReachedError
        );

        // Initialize stake account
        self.initialize_stake_account(bumps)?;

        // Delegate NFT authority
        self.delegate_nft_authority()?;

        // Freeze delegated NFT authority
        self.freeze_delegated_nft_authority()?;

        self.user_account.amount_staked += 1;

        Ok(())
    }

    pub fn initialize_stake_account(&mut self, bumps: &StakeBumps) -> Result<()> {
        self.stake_account.set_inner(StakeAccount {
            mint: self.nft_mint.key(),
            owner: self.user.key(),
            staked_at: Clock::get()?.unix_timestamp,
            bump: bumps.stake_account,
        });

        Ok(())
    }

    pub fn delegate_nft_authority(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Approve {
            to: self.user_nft_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_ctx, 1)?;

        Ok(())
    }

    pub fn freeze_delegated_nft_authority(&mut self) -> Result<()> {
        let cpi_program = &self.metadata_program.to_account_info();

        let cpi_accounts = FreezeDelegatedAccountCpiAccounts {
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

        FreezeDelegatedAccountCpi::new(&cpi_program, cpi_accounts).invoke_signed(signer_seeds)?;

        Ok(())
    }
}
