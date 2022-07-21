use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program_option::COption, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("2ocbQycEn49nySkwoVN4fwiJTPR8zynjA77DbFj7qy1H");

#[program]
pub mod nftotc {
    use super::*;

    pub fn create_otc(ctx: Context<CreateOTC>, otc_nonce, amount_a: Vec<u128>, amount_b: Vec<u128>, expires: u128) -> Result<()> {
        let otc = &mut ctx.accounts.otc;
        otc.seller = ctx.accounts.seller.key();
        otc.nonce = otc_nonce;
        // if no-specific buyer asigned, buyer == seller pubkey
        otc.buyer = ctx.accounts.buyer.key;
        // if no expiration time, expires == 0 
        otc.expires = expires;
        otc.executed = false;

        for x in 1..5 {
            if ctx.accounts.token_a_mint[x].key != ctx.accounts.seller.key {
                if ctx.accounts.token_a_vault[x].owner == ctx.accounts.seller.key {
                    if ctx.accounts.token_a_otc_vault[x].owner == ctx.accounts.otc_signer.key {
                        // Transfer tokens into the stake vault.
                        {
                            let cpi_ctx = CpiContext::new(
                                ctx.accounts.token_program.to_account_info(),
                                token::Transfer {
                                    from: ctx.accounts.token_a_vault[x].to_account_info(),
                                    to: ctx.accounts.token_a_otc_vault[x].to_account_info(),
                                    authority: ctx.accounts.seller.to_account_info(),
                                },
                            );
                            token::transfer(cpi_ctx, amount_a[x])?;
                        }

                        if ctx.accounts.token_b_mint[x].key != ctx.accounts.seller.key {
                            if ctx.accounts.token_b_vault[x].owner == ctx.accounts.seller.key {
                                otc.buy_asset.push(
                                    Asset {
                                        asset_mint: ctx.accounts.token_a_mint[x].key,
                                        asset_vault: ctx.accounts.token_a_otc_vault[x].key,
                                        amount: amount_a[x],
                                    }
                                );

                                otc.sell_asset.push(Asset {
                                    asset_mint: ctx.accounts.token_b_mint[x].key,
                                    asset_vault: ctx.accounts.token_b_vault[x].key,
                                    amount: amount_b[x],
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn execute_otc(ctx: Context<ExecuteOTC>) -> Result<()> {
        let otc = &mut ctx.accounts.otc;
        let c = clock::Clock::get().unwrap();

        require!(c.unix_timestamp < otc.expires || otc.expires == 0, ErrorCode::Expired);
        for x in 1..5 {
            if otc.buy_asset[x].asset_mint != ctx.accounts.seller.key {
                if otc.buy_asset[x].asset_vault == ctx.accounts.token_b_seller_vault[x].key {
                    {
                        let cpi_ctx = CpiContext::new(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.token_b_vault[x].to_account_info(),
                                to: ctx.accounts.token_b_seller_vault[x].to_account_info(),
                                authority: ctx.accounts.buyer.to_account_info(), 
                            },
                        );
                        token::transfer(cpi_ctx, buy_asset[x].amount)?;
                    }
                }
            }

            if otc.sell_asset[x].asset_mint != ctx.accounts.seller.key {
                if otc.sell_asset[x].asset_vault == ctx.accounts.token_a_otc_vault[x].key {
                    {
                        let seeds = &[
                            ctx.accounts.otc.to_account_info().key.as_ref(),
                            &[ctx.accounts.otc.nonce],
                        ];
                        let otc_signer = &[&seeds[..]];
            
                        let cpi_ctx = CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            token::Transfer {
                                from: ctx.accounts.token_a_otc_vault[x].to_account_info(),
                                to: ctx.accounts.token_a_buyer_vault[x].to_account_info(),
                                authority: ctx.accounts.otc_signer.to_account_info(),
                            },
                            otc_signer,
                        );
                        token::transfer(cpi_ctx, ctx.accounts.otc.sell_asset[x].amount as u64)?;
                    }
                }
            }
        }

        Ok(())
    }

    // pub fn cancel_otc(ctx: Context<CancelOTC>) -> Result<()> {
    //     let otc = &mut ctx.accounts.otc;
    //     let c = clock::Clock::get().unwrap();

    //     require!(c.unix_timestamp < otc.expires || otc.expires == 0, ErrorCode::Expired);
    //     for x in 1..5 {
    //         if otc.buy_asset[x].asset_mint != ctx.accounts.seller.key {
    //             if otc.buy_asset[x].asset_vault == ctx.accounts.token_b_seller_vault[x].key {
    //                 {
    //                     let cpi_ctx = CpiContext::new(
    //                         ctx.accounts.token_program.to_account_info(),
    //                         token::Transfer {
    //                             from: ctx.accounts.token_b_vault[x].to_account_info(),
    //                             to: ctx.accounts.token_b_seller_vault[x].to_account_info(),
    //                             authority: ctx.accounts.buyer.to_account_info(), 
    //                         },
    //                     );
    //                     token::transfer(cpi_ctx, buy_asset[x].amount)?;
    //                 }
    //             }
    //         }

    //         if otc.sell_asset[x].asset_mint != ctx.accounts.seller.key {
    //             if otc.sell_asset[x].asset_vault == ctx.accounts.token_a_otc_vault[x].key {
    //                 {
    //                     let seeds = &[
    //                         ctx.accounts.otc.to_account_info().key.as_ref(),
    //                         &[ctx.accounts.otc.nonce],
    //                     ];
    //                     let otc_signer = &[&seeds[..]];
            
    //                     let cpi_ctx = CpiContext::new_with_signer(
    //                         ctx.accounts.token_program.to_account_info(),
    //                         token::Transfer {
    //                             from: ctx.accounts.token_a_otc_vault[x].to_account_info(),
    //                             to: ctx.accounts.token_a_buyer_vault[x].to_account_info(),
    //                             authority: ctx.accounts.otc_signer.to_account_info(),
    //                         },
    //                         otc_signer,
    //                     );
    //                     token::transfer(cpi_ctx, ctx.accounts.otc.sell_asset[x].amount as u64)?;
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}

#[derive(Accounts)]
#[instruction(otc_nonce: u8)]
pub struct CreateOTC {
    pub buyer: Signer<'info>,

    pub seller: UncheckedAccount<'info>,

    pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    pub token_b_mint: Vec<Box<Account<'info, Mint>>>,
    pub token_b_vault: Vec<Box<Account<'info, TokenAccount>>>,

    #[account(zero)]
    pub otc: Box<Account<'info, OTC>>,

    #[account(
        seeds = [
            otc.to_account_info().key.as_ref(),
        ],
        bump = otc_nonce
    )]
    pub otc_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExecuteOTC {
    #[account(
        constraint = otc.buyer == buyer.key
    )]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        constraint = otc.seller == seller.key
    )]
    pub seller: Signer<'info>,

    pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    pub token_a_buyer_vault: Vec<Box<Account<'info, TokenAccount>>>,
    pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    pub token_b_mint: Vec<Box<Account<'info, Mint>>>,
    pub token_b_vault: Vec<Box<Account<'info, TokenAccount>>>,
    pub token_b_seller_vault: Vec<Box<Account<'info, TokenAccount>>>,

    #[account(mut)]
    pub otc: Box<Account<'info, OTC>>,

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

#[derive(Accounts)]
pub struct CancelOTC {
    #[account(
        constraint = otc.buyer == buyer.key
    )]
    pub buyer: Signer<'info>,

    #[account(
        constraint = otc.seller == seller.key
    )]
    pub seller: UncheckedAccount<'info>,

    pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    #[account(mut)]
    pub otc: Box<Account<'info, OTC>>,

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

#[account]
pub struct OTC {
    /// Seller 
    pub seller: Pubkey,
    /// Buyer (if any)
    pub buyer: Pubkey,

    pub sell_asset: Vec<Asset>,
    pub buy_asset: Vec<Asset>,

    pub expires: u128,

    pub executed: bool,

    pub nonce: u8,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Asset {
    pub asset_mint: Pubkey,
    pub asset_vault: Pubkey,
    pub amount: u128,
}

#[error_code]
pub struct ErrorCode {
    #[msg("Expired")]
    Expired,
}