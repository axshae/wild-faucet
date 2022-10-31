use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
// use anchor_spl::{token::{TokenAccount, Mint, Token,Transfer}, associated_token::AssociatedToken};
use instructions::*;
mod instructions;
mod state;
declare_id!("9PFwwQwLtGpFAPRuTW2ze8tRddBtVRQVJVBXpG96fGaq");

#[program]
pub mod faucet {
    use super::*;

    pub fn initialize_faucet(ctx: Context<Initialize>, price_per_token : u64) -> Result<()> {
        instructions::initialize::handler(ctx, price_per_token)?;
        Ok(())
    }
    pub fn update_faucet(ctx: Context<Initialize>,price_per_token : u64) -> Result<()>{


        let state = &mut ctx.accounts.faucet_state;
        
        require!(state.allowed_token == ctx.accounts.allowed_token_mint.key(), ErrorCode::FaucetAccountNotInitialised);

        state.price_per_token = price_per_token;
        state.treasury = ctx.accounts.treasury_acc.key();
        msg!("Faucet updated!");
        Ok(())
    }

    // pub fn close_faucet(ctx)

    pub fn close_faucet(ctx: Context<CloseFaucet>) -> Result<()>{

        instructions::close_faucet::handler(ctx)?;
        
        Ok(())
    }
    pub fn fund_faucet(ctx:Context<Fund>,amount:u64) -> Result<()> {
        instructions::fund::handler(ctx, amount)?;
        Ok(())
    }

    pub fn refund_balance(ctx: Context<Refund>) -> Result<()>{
        msg!("Refund balance");
        instructions::refund::handler(ctx)
    }
    pub fn request_airdrop(ctx:Context<Airdrop>,airdrop_amount:u64)-> Result<()>{
        instructions::airdrop::handler(ctx, airdrop_amount)?;
        Ok(())
    }

   
}


#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Balance in your account.")]
    InsufficientBalance,
    #[msg("Transfer failed.")]
    TransferFailed,
    #[msg("Not authorized to make changes. Invalid Authority.")]
    InvalidAuthority,
    #[msg("Insufficient pool balance")]
    InsufficientPoolBalance,
    #[msg("Invalid Treasury account")]
    InvalidTreasuryAccount,
    #[msg("Airdrop failed.")]
    AirdropFailed,
    #[msg("Faucet Account not initialised")]
    FaucetAccountNotInitialised,
    #[msg("Faucet Account already initialised")]
    FaucetAccountAlreadyInitialised
}