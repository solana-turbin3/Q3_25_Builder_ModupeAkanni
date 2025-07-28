use anchor_lang::prelude::*;

declare_id!("FZwFhjZST6yfWyVY4nogHMqMEXBafLPXb37p7iYaEkRs");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        fee_percentage: u8,
    ) -> Result<()> {
        ctx.accounts.initialize(fee_percentage, ctx.bumps)?;
        Ok(())
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.list(price, ctx.bumps)?;
        ctx.accounts.transfer()
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.delist()
    }

    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.transfer_nft()?;
        ctx.accounts.transfer_sol()?;
        ctx.accounts.delist_nft()
    }
}
