use anchor_lang::{prelude::*,solana_program::pubkey::Pubkey};
use anchor_spl::{token::{TokenAccount, Mint, Token, Transfer}, associated_token::AssociatedToken};

use crate::state::*;
use crate::ErrorCode;

// mod ErrorCode;

#[derive(Accounts)]
#[instruction(airdrop_amount:u64)]

pub struct Airdrop<'info>{
    #[account(
        mut,
        seeds = [b"faucet_state".as_ref(),allowed_token_mint.key().as_ref(),faucet_authority.key().as_ref()],
        bump,
        constraint = faucet_authority.key() == faucet_state.authority
        // has_one=faucet_authority
    )]
    pub faucet_state : Account<'info,FaucetState>,

    #[account(
        mut,
        seeds = [b"token_pool".as_ref(),allowed_token_mint.key().as_ref(),faucet_state.key().as_ref()],
        bump,
        token::mint = allowed_token_mint,
        token::authority = faucet_state,
        constraint = faucet_state.total_supply > 0 && faucet_state.total_supply >= airdrop_amount @ ErrorCode::InsufficientPoolBalance
        // owner = faucet_state.key()
        // space=8
    )]
    pub token_pool_acc: Account<'info,TokenAccount>,
      
    #[account(mut,address = faucet_state.allowed_token)]
    pub allowed_token_mint : Account<'info,Mint>,


    /// CHECK:
    #[account(address = faucet_state.authority)]
    pub faucet_authority : AccountInfo<'info>,

    /// CHECK:
    #[account(mut,address = faucet_state.treasury)]
    pub treasury_acc : AccountInfo<'info>,
    #[account(
            init_if_needed,
            associated_token::mint = allowed_token_mint,
            associated_token::authority = signer,
            payer = signer
        )
    ]
    pub receiver_ata : Account<'info,TokenAccount>,
    #[account(mut)]
    pub signer : Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx:Context<Airdrop>,airdrop_amount:u64)-> Result<()>{

    let allowed_token = ctx.accounts.allowed_token_mint.to_account_info();
    let faucet_state = ctx.accounts.faucet_state.to_account_info();
    let token_pool =  ctx.accounts.token_pool_acc.to_account_info();
    let signer   = ctx.accounts.signer.to_account_info();
    let token_decimals = ctx.accounts.allowed_token_mint.decimals;
    
    let lamports_required = (airdrop_amount / u64::pow(10, token_decimals as u32)) * ctx.accounts.faucet_state.price_per_token; 

    let faucet_bump_opt = ctx.bumps.get("faucet_state");
    let faucet_bump = faucet_bump_opt.unwrap();
    let faucet_bump_vector = faucet_bump.to_le_bytes();
    
    // step 1 : transfer lamports / payment

    if **ctx.accounts.signer.try_borrow_lamports()? < lamports_required {
        return Err(ErrorCode::InsufficientBalance.into());
    }
    
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &ctx.accounts.treasury_acc.key(),
        lamports_required,
    );
    let result = anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.treasury_acc.to_account_info(),
        ],
    );

    require!(result.is_ok(),ErrorCode::AirdropFailed); // check if sol payment is passed

    // step 2: airdrop tokens

    
    let faucet_seeds = vec![
        b"faucet_state".as_ref(),
        allowed_token.key.as_ref(),
        ctx.accounts.faucet_authority.key.as_ref(),        
        faucet_bump_vector.as_ref(),
         // bump is required to sign tx with pda
    ];

    let transfer_instruction = Transfer{
        from:token_pool,
        to: ctx.accounts.receiver_ata.to_account_info().clone(),
        authority:faucet_state,
    };

    let token_program = ctx.accounts.token_program.to_account_info();
    let seeds_outer = vec![faucet_seeds.as_slice()];
    let cpi_ctx = CpiContext::new_with_signer(token_program,
     transfer_instruction, seeds_outer.as_slice());

    let result  = anchor_spl::token::transfer(cpi_ctx,airdrop_amount );

    require!(result.is_ok(),ErrorCode::AirdropFailed); // check if spl transfer is passed
   
    // change supply count

    ctx.accounts.faucet_state.total_supply-=airdrop_amount;
    Ok(())
}
