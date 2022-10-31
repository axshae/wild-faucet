use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
use anchor_spl::{token::{TokenAccount, Mint, Token, Transfer}, associated_token::AssociatedToken};

use crate::state::*;
use crate::ErrorCode;
#[derive(Accounts)]
pub struct Refund<'info>{
    #[account(
        mut,
        seeds = [b"faucet_state".as_ref(),allowed_token_mint.key().as_ref(),authority.key().as_ref()],
        bump,
        has_one=authority
    )]
    pub faucet_state : Account<'info,FaucetState>,

    #[account(
        mut,
        seeds = [b"token_pool".as_ref(),allowed_token_mint.key().as_ref(),faucet_state.key().as_ref()],
        bump,
        token::mint = allowed_token_mint,
        token::authority = faucet_state,

        // owner = faucet_state.key()
        // space=8
    )]
    pub token_pool_acc: Account<'info,TokenAccount>,
    #[
        account(
            mut,
            associated_token::mint = allowed_token_mint,
            associated_token::authority = authority,
        )
    ]
    pub receiver_ata : Account<'info,TokenAccount>,
    
    #[account(mut)]
    pub allowed_token_mint : Account<'info,Mint>,

    #[account(mut)]
    pub authority : Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}


pub fn handler(ctx : Context<Refund>) -> Result<()>{

    let faucet_state_mut = &mut ctx.accounts.faucet_state;
    let allowed_token = ctx.accounts.allowed_token_mint.to_account_info();
    let token_pool =  ctx.accounts.token_pool_acc.to_account_info();

    let faucet_bump_opt = ctx.bumps.get("faucet_state");
    let faucet_bump = faucet_bump_opt.unwrap();
    let faucet_bump_vector = faucet_bump.to_le_bytes();
   
    // step 1: airdrop tokens
    
    let faucet_seeds = vec![
        b"faucet_state".as_ref(),
        allowed_token.key.as_ref(),
        ctx.accounts.authority.key.as_ref(),        
        faucet_bump_vector.as_ref(),
         // bump is required to sign tx with pda
    ];
    let refund_amount = ctx.accounts.token_pool_acc.amount;
    let transfer_instruction = Transfer{
        from:token_pool,
        to: ctx.accounts.receiver_ata.to_account_info(),
        authority:faucet_state_mut.to_account_info(),
    };

    
    let token_program = ctx.accounts.token_program.to_account_info();
    let seeds_outer = vec![faucet_seeds.as_slice()];
    let cpi_ctx = CpiContext::new_with_signer(token_program,
     transfer_instruction, seeds_outer.as_slice());

    let result  = anchor_spl::token::transfer(cpi_ctx,refund_amount );

    require!(result.is_ok(),ErrorCode::AirdropFailed); // check if spl transfer is passed
   
    // step 2: set supply to 0
    faucet_state_mut.total_supply = 0;

    Ok(())
}