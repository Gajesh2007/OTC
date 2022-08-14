use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(otc_nonce: u8)]
pub struct CreateOTC<'info> {
    /// CHECK: TODO
    pub buyer: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    // pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    // pub token_b_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_b_vault: Vec<Box<Account<'info, TokenAccount>>>,
    #[account(init, space=1000, payer = seller)]
    pub otc: Box<Account<'info, OTC>>,

    /// CHECK: TODO
    #[account(
        seeds = [
            otc.to_account_info().key.as_ref(),
        ],
        bump = otc_nonce
    )]
    pub otc_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateOTC<'info>>,
    otc_nonce: u8,
    amount_a: Vec<u128>,
    amount_b: Vec<u128>,
    expires: u128,
) -> Result<()> {
    let otc = &mut ctx.accounts.otc;
    otc.seller = ctx.accounts.seller.key();
    otc.nonce = otc_nonce;
    otc.buyer = ctx.accounts.buyer.key();
    otc.expires = expires;
    otc.executed = false;
    if amount_a.len() != amount_b.len() {
        return Err(error!(ErrorCode::LengthMisMatch));
    }
    let max_remaining_account_size: usize = amount_a.len();
    let mut remaining_accounts_counter: usize = 0;
    if ctx.remaining_accounts.len() == 0
        && ctx.remaining_accounts.len() % 5 != 0
        && ctx.remaining_accounts.len() > 20
    {
        return Err(error!(ErrorCode::RemainingAccountsWrong));
    }

    for n in 0..max_remaining_account_size {
        let token_a_mint = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        if token_a_mint.key() == ctx.accounts.seller.key() {
            return Err(error!(ErrorCode::MintAccountWrong));
        }
        let seller_token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let seller_token_account = Account::<TokenAccount>::try_from(seller_token_account_info)?;
        msg!(
            "seller_token_account: Owner {:?}",
            seller_token_account.owner
        );
        msg!("Seller Key {:?}", *ctx.accounts.seller.key);
        msg!("SELLER TOKEN MINT {:?}", seller_token_account.mint);
        msg!("TOKEN A MINT {:?}", token_a_mint.key());
        if seller_token_account.owner != *ctx.accounts.seller.key
            && seller_token_account.mint != token_a_mint.key()
        {
            return Err(error!(ErrorCode::MisMatchMintOwner));
        }

        let token_a_otc_vault_info = &ctx.remaining_accounts[remaining_accounts_counter];
        let token_otc_vault = Account::<TokenAccount>::try_from(token_a_otc_vault_info)?;
        remaining_accounts_counter += 1;
        if token_otc_vault.owner != *ctx.accounts.otc_signer.key {
            return Err(error!(ErrorCode::MisMatchOTCVaultOwner));
        }
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: seller_token_account.to_account_info(),
                to: token_a_otc_vault_info.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_a[n] as u64)?;
        otc.sell_asset.push(Asset {
            asset_mint: *token_a_mint.key,
            asset_vault: token_otc_vault.key(), // token_a_otc_vault
            amount: amount_a[n] as u128,
        });

        let token_b_mint = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let token_b_seller_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        let token_b_seller_account =
            Account::<TokenAccount>::try_from(token_b_seller_account_info)?;
        remaining_accounts_counter += 1;
        if token_b_seller_account.owner != *ctx.accounts.seller.key {
            return Err(error!(ErrorCode::MisMatchBuyerOwner));
        }
        otc.buy_asset.push(Asset {
            asset_mint: *token_b_mint.key,
            asset_vault: token_b_seller_account.key(),
            amount: amount_b[n] as u128,
        });
    }
    Ok(())
}
