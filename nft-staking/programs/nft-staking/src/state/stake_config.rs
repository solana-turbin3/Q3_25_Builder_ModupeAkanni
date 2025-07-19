use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfig {
    pub points_per_stake: u8,
    pub max_stake: u8, // Max amount of NFTs to stake
    pub freeze_period: u32, // for how long an NFT must be locked
    pub rewards_bump: u8,
    pub bump: u8, // config bump
}