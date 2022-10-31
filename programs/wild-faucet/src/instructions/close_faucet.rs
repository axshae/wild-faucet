use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
use anchor_spl::{token::{TokenAccount, Mint, Token}, associated_token::AssociatedToken};

use crate::state::*;
use crate::ErrorCode;
// close accounts
#[derive(Accounts)]
pub struct CloseFaucet<'info>{

    #[account(
        mut,
        close = authority,
        seeds = [b"faucet_state".as_ref(),allowed_token_mint.key().as_ref(),authority.key().as_ref()],
        bump,
        has_one=authority
    )]
    pub faucet_state : Account<'info,FaucetState>,

    #[account(
        mut,
        // close = authority, // tokens goes to the authority associated token account
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
    pub token_ata : Account<'info,TokenAccount>,
    
    #[account(mut)]
    pub allowed_token_mint : Account<'info,Mint>,

    #[account(mut)]
    pub authority : Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx:Context<CloseFaucet>) ->  Result<()>{
    let token = ctx.accounts.allowed_token_mint.to_account_info().clone();
        let authority = ctx.accounts.authority.to_account_info();
        let bump = ctx.bumps.get("faucet_state").unwrap(); // get the faucet_state pda bump and unwrap

        let bump_vector = bump.to_le_bytes();
        let faucet_seeds = vec![
            b"faucet_state".as_ref(),
            // token.key().as_ref(),
            token.key.as_ref(),
            authority.key.as_ref(),
            bump_vector.as_ref() // bump is required to sign tx with pda
        ];

        let outer_faucet_seeds = vec![faucet_seeds.as_slice()];
        let close = anchor_spl::token::CloseAccount{
            account: ctx.accounts.token_pool_acc.to_account_info(),
            destination: ctx.accounts.authority.to_account_info(),
            authority: ctx.accounts.faucet_state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), 
        close, outer_faucet_seeds.as_slice());

        anchor_spl::token::close_account(cpi_ctx)?;
        
        msg!(
            "{} Faucet and Token pool {} are now closed", 
            ctx.accounts.faucet_state.key(), 
            ctx.accounts.token_pool_acc.key()
        );
        Ok(())

}