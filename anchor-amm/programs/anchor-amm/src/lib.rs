use anchor_lang::prelude::*;

declare_id!("d8HffbmM4CGdfpAgeksN4QcnQcHDyS7Nszg71h4H7Jr");

pub mod errors;
pub mod instructions;
pub mod states;

pub use instructions::*;

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.initialize(seed, fee, authority, &ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
