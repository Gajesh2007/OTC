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

    /// CHECK: TODO
    #[account(
        constraint = otc.buyer == *buyer.key
    )]
    pub buyer: UncheckedAccount<'info>,

    // pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,
    #[account(mut)]
    pub otc: Box<Account<'info, OTC>>,

        /// CHECK: TODO
    #[account(
        seeds = [
            otc.to_account_info().key.as_ref(),
        ],
        bump = otc.nonce
    )]
    pub otc_signer: UncheckedAccount<'info>,

    // Misc
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CancelOTC<'info>>) -> Result<()> {
    let otc = &mut ctx.accounts.otc;
    // let c = clock::Clock::get().unwrap();

    require!(otc.executed == false, ErrorCode::AlreadyExecuted);

    let max_remaining_account_size: usize = otc.buy_asset.len();
    let mut remaining_accounts_counter: usize = 0;

    for n in 0..max_remaining_account_size {
        let token_a_mint = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;

        if token_a_mint.key() != otc.sell_asset[n].asset_mint
            && token_a_mint.key() == ctx.accounts.seller.key()
        {
            return Err(error!(ErrorCode::RemainingAccountsWrong));
        }

        let token_a_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let token_a_otc_vault_info = &ctx.remaining_accounts[remaining_accounts_counter];

        if otc.sell_asset[n].asset_vault != token_a_otc_vault_info.key() {
            return Err(error!(ErrorCode::RemainingAccountsWrong));
        }
        let seeds = &[otc.to_account_info().key.as_ref(), &[otc.nonce]];
        let otc_signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: token_a_otc_vault_info.to_account_info(),
                to: token_a_account_info.to_account_info(),
                authority: ctx.accounts.otc_signer.to_account_info(),
            },
            otc_signer,
        );
        token::transfer(cpi_ctx, otc.sell_asset[n].amount as u64)?;
    }
    Ok(())
}
