use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
use anchor_spl::{token::{TokenAccount, Mint, Token}};

use crate::state::*;
use crate::ErrorCode;
#[derive(Accounts)]
pub struct Initialize<'info> {
    
    #[account(
        init_if_needed,
        seeds = [b"faucet_state".as_ref(),allowed_token_mint.key().as_ref(),authority.key().as_ref()],
        bump,
        payer=authority,
        space= 8 + 32 +  32 + 32 + 8 + 8 + 32, // discrementaor + pubkey + pubkey + pubkey + u64 + u64 + SAFE_BUFFER_FOR_ENDCASE
        // has_one=authority
    )]
    pub faucet_state : Account<'info,FaucetState>,

    #[account(
        init_if_needed,
        seeds = [b"token_pool".as_ref(),allowed_token_mint.key().as_ref(),faucet_state.key().as_ref()],
        bump,
        payer=authority,
        token::mint = allowed_token_mint,
        token::authority = faucet_state,
        // space=8
    )]
    pub token_pool_acc: Account<'info,TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub treasury_acc: AccountInfo<'info>,

    #[account(mut)]
    pub allowed_token_mint : Account<'info,Mint>,

    #[account(mut)]
    pub authority : Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}

pub fn handler(ctx: Context<Initialize>, price_per_token : u64) -> Result<()> {
    let state = &mut ctx.accounts.faucet_state;

    // throw error if faucet is already initialized
    require!(state.allowed_token != ctx.accounts.allowed_token_mint.key(), ErrorCode::FaucetAccountAlreadyInitialised);

    state.allowed_token = ctx.accounts.allowed_token_mint.key();
    state.total_supply = 0;
    state.authority = ctx.accounts.authority.key();
    
    state.price_per_token = price_per_token;
    state.treasury = ctx.accounts.treasury_acc.key();
    msg!("Faucet initialized!");
    Ok(())
}
