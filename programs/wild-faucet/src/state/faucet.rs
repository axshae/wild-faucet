
use anchor_lang::prelude::*;

#[account]
pub struct FaucetState{
    pub authority:Pubkey,
    pub treasury: Pubkey,
    pub allowed_token:Pubkey,
    pub price_per_token:u64,
    pub total_supply:u64, // will be dynamic  - to be updated when funding is done
    // pub bump:u8,
    
}