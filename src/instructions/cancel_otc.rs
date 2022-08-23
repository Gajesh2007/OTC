use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
pub struct CancelOTC<'info> {
    #[account(
        constraint = otc.seller == *seller.key
    )]
    pub seller: Signer<'info>,

    #[account(
        mut,
        constraint = otc.seller == *seller.key,
    )]
    pub otc: Box<Account<'info, Otc>>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CancelOTC<'info>>) -> Result<()> {
    let otc = &mut ctx.accounts.otc;

    require!(otc.executed == false, ErrorCode::AlreadyExecuted);

    otc.executed = true;
    Ok(())
}
