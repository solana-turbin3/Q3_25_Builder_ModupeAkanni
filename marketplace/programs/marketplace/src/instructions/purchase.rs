use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Token, TransferChecked},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    errors::MarketplaceError,
    state::{Listing, Marketplace},
};

#[derive(Accounts)]
pub struct Purchase<'info> {
    pub nft: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [
            b"listing",
            marketplace.key().as_ref(),
            seller.key().as_ref(),
            nft.key().as_ref(),
        ],
        bump
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        associated_token::mint = nft,
        associated_token::authority = listing,
    )]
    pub listing_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we check in the instruction logic
    pub seller: AccountInfo<'info>,

    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Purchase<'info> {
    pub fn transfer_nft(&mut self) -> Result<()> {
        require!(
            self.listing.is_active && self.listing.seller == self.seller.key(),
            MarketplaceError::ListingNotActive
        );

        let marketplace = self.marketplace.key();
        let seller = self.seller.key();
        let nft = self.nft.key();
        let listing_seeds: &[&[u8]] = &[
            b"listing",
            marketplace.as_ref(),
            seller.as_ref(),
            nft.as_ref(),
            &[self.listing.bump],
        ];
        let signer = &[listing_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.listing_token_account.to_account_info(),
                mint: self.nft.to_account_info(),
                to: self.buyer_token_account.to_account_info(),
                authority: self.listing.to_account_info(),
            },
            signer,
        );

        transfer_checked(cpi_ctx, 1, self.nft.decimals)
    }

    pub fn transfer_sol(&mut self) -> Result<()> {
        let fee_lamports = (self.marketplace.fee_percentage as u64)
            .checked_mul(self.listing.price)
            .ok_or(MarketplaceError::MathOverflow)?
            .checked_div(100)
            .ok_or(MarketplaceError::MathOverflow)?;

        let seller_lamports = self
            .listing
            .price
            .checked_sub(fee_lamports)
            .ok_or(MarketplaceError::MathOverflow)?;

        let treasury_transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.buyer.to_account_info(),
                to: self.treasury.to_account_info(),
            },
        );
        transfer(treasury_transfer_ctx, fee_lamports)?;

        let seller_transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.buyer.to_account_info(),
                to: self.seller.to_account_info(),
            },
        );
        transfer(seller_transfer_ctx, seller_lamports)?;

        Ok(())
    }

    pub fn delist_nft(&mut self) -> Result<()> {
        self.listing.is_active = false;
        Ok(())
    }
}