use anchor_lang::prelude::*;

mod state;
mod instructions;

use instructions::*;

declare_id!("43hdv5N9Lb2XxZSaaYSLAFhoiDNCNka8ezRhnLRDozJN");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
