use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::{self, Token, TokenAccount};

#[derive(Accounts)]
pub struct ExecuteOTC<'info> {
    pub buyer: Signer<'info>,

    /// CHECK: TODO
    #[account(
        constraint = otc.seller == *seller.key
    )]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = otc.seller == *seller.key,
    )]
    pub otc: Box<Account<'info, Otc>>,

    /// CHECK: TODO
    #[account(
        seeds = [
            otc.to_account_info().key.as_ref(),
            seller.to_account_info().key.as_ref(),
        ],
        bump = otc.nonce
    )]
    pub otc_signer: UncheckedAccount<'info>,

    // Misc
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteOTC<'info>>) -> Result<()> {
    let otc = &mut ctx.accounts.otc;

    require!(otc.executed == false, ErrorCode::AlreadyExecuted);
    msg!("{}", otc.sell_asset.len());
    let sell_asset_count = otc.sell_asset.len();
    let mut remaining_accounts_counter: usize = 0;

    if token::ID != *ctx.accounts.token_program.key {
        return Err(error!(ErrorCode::InvalidTokenId));
    }

    if otc.buyer == ctx.accounts.seller.key() {
        msg!("NICE!");
    } else if otc.buyer != ctx.accounts.seller.key() {
        return Err(error!(ErrorCode::InvalidBuyer));
    } else if ctx.accounts.buyer.key() != otc.buyer {
        return Err(error!(ErrorCode::InvalidBuyer));
    }

    for n in 0..sell_asset_count {
        let token_a_mint = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;

        if token_a_mint.key() != ctx.accounts.seller.key() && token_a_mint.key() == otc.sell_asset[n].asset_mint {
            let token_a_buyer_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            let token_a_seller_vault_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            if otc.sell_asset[n].asset_vault != token_a_seller_vault_info.key() {
                return Err(error!(ErrorCode::MintAccountWrong));
            }

            let token_a_buyer_account = Account::<TokenAccount>::try_from(token_a_buyer_account_info)?;        
            let token_a_seller_account = Account::<TokenAccount>::try_from(token_a_seller_vault_info)?;

            if token_a_buyer_account.owner != ctx.accounts.buyer.key() && token_a_seller_account.owner != ctx.accounts.seller.key() {
                return Err(error!(ErrorCode::InvalidOwner));
            }

            let seeds = &[otc.to_account_info().key.as_ref(), ctx.accounts.seller.key.as_ref(), &[otc.nonce]];
            let otc_signer = &[&seeds[..]];

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: token_a_seller_vault_info.to_account_info(),
                    to: token_a_buyer_account_info.to_account_info(),
                    authority: ctx.accounts.otc_signer.to_account_info(),
                },
                otc_signer,
            );
            token::transfer(cpi_ctx, otc.sell_asset[n].amount as u64)?;
        } else {
            remaining_accounts_counter += 2;
        }
    
        
        let token_b_mint_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;

        if token_b_mint_info.key() != ctx.accounts.seller.key() && token_b_mint_info.key() == otc.buy_asset[n].asset_mint {
            let token_b_token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            // let token_b_token_account = Account::<TokenAccount>::try_from(token_b_token_account_info)?;
            let token_b_seller_token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            let token_b_seller_token_account = 
                Account::<TokenAccount>::try_from(token_b_seller_token_account_info)?;
            let token_b_token_account = 
                Account::<TokenAccount>::try_from(token_b_token_account_info)?;

            if otc.buy_asset[n].asset_vault != token_b_seller_token_account_info.key()
                && token_b_seller_token_account.owner != ctx.accounts.seller.key()
                && token_b_token_account.owner != ctx.accounts.buyer.key()
            {
                return Err(error!(ErrorCode::MintAccountWrong));
            }

            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: token_b_token_account_info.to_account_info(),
                    to: token_b_seller_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            );
            token::transfer(cpi_ctx, otc.buy_asset[n].amount as u64)?;
        } else {
            remaining_accounts_counter += 2;
        }
        
    }

    otc.executed = true;

    // Details:
    // token_a_mint - sell asset mint (0, 6, 12, 18, 24)
    // token_a_buyer_vault - buyer's vault to hold the token_a asset (1,7,13, 19, 25)
    // token_a_otc_vault - program holding the token_a asset (2,8,14, 20, 26)
    // token_b_mint - buy asset mint (3, 9, 15, 21, 27)
    // token_b_vault - buyer's vault holding the tokens (4, 10, 16, 22, 28)
    // token_b_seller_vault - seller's vault for holding the tokens (5, 11, 17, 23, 29)

    Ok(())
}
