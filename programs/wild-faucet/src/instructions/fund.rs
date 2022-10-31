use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
use anchor_spl::{token::{TokenAccount, Mint, Token, Transfer}, associated_token::AssociatedToken};

use crate::state::*;
use crate::ErrorCode;
#[derive(Accounts)]
#[instruction(amount:u64)]
pub struct Fund<'info>{

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
            constraint = amount > 0 && token_ata.amount >= amount @ErrorCode::InsufficientBalance
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

pub fn handler(ctx:Context<Fund>,amount:u64) -> Result<()> {

    // msg!("Fund called amt:{} tokens:{}",amount,ctx.accounts.token_ata.amount);

    // let state_pda = ctx.accounts.faucet_state.clone();
    let pool = ctx.accounts.token_pool_acc.clone().to_account_info();
    let signer = ctx.accounts.authority.clone();
    
    let token = ctx.accounts.allowed_token_mint.to_account_info().clone();
    let transfer_instruction = Transfer{
        from:ctx.accounts.token_ata.to_account_info(),
        to: pool,
        authority: signer.to_account_info()
    };

    /* let seeds:[&[u8];3] = [
        b"faucet_state".as_ref(),
        // token.key().as_ref(),
        token.key.as_ref(),
        signer.key.as_ref(),
        ]; */
    
    // let (pda,bump) = Pubkey::find_program_address(&seeds,&ctx.program_id );

    let bump = ctx.bumps.get("faucet_state").unwrap(); // get the faucet_state pda bump and unwrap
    let bump_vector = bump.to_le_bytes();
    let seeds_with_bump = [
        b"faucet_state".as_ref(),
        // token.key().as_ref(),
        token.key.as_ref(),
        signer.key.as_ref(),
        bump_vector.as_ref() // bump is required to sign tx with pda
    ];

    // msg!("PDA generated:{}",pda.to_string());
    let cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );

    // cpi.with_signer is optional. It's just to additionally sign the transaction with the state pda
    // once signed it all show in tx history of faucet state
    let result= anchor_spl::token::transfer(cpi.with_signer(&[&seeds_with_bump]), amount); 

    // throw error if result is not ok
    require!(result.is_ok(),ErrorCode::TransferFailed);

    // increase total supply

    ctx.accounts.faucet_state.total_supply+=amount;
    msg!("Funded {} with {} tokens",ctx.accounts.token_pool_acc.key().to_string(),amount);
    
    // if amount > 0 && ctx.accounts.token_ata.amount >= amount{
    //     panic!("Not enough amount")
    // }
    // if 
    // 1st find associated token account and check supply - [DONE USING ANCHOR]
    // 2nd transfer from ata to pda pool
    // 3rd set total supply var

    // Start fund transfer
    
    Ok(())
}
