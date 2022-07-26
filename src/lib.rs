use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program_option::COption, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("2ocbQycEn49nySkwoVN4fwiJTPR8zynjA77DbFj7qy1H");

#[program]
pub mod nftotc {
    use super::*;

    pub fn create_otc<'info>(ctx: Context<'_, '_, '_, 'info, CreateOTC<'info>>, otc_nonce: u8, amount_a: Vec<u128>, amount_b: Vec<u128>, expires: u128) -> Result<()> {
        let otc = &mut ctx.accounts.otc;

        otc.seller = ctx.accounts.seller.key();
        otc.nonce = otc_nonce;
        // if no-specific buyer asigned, buyer == seller pubkey
        otc.buyer = ctx.accounts.buyer.key();
        // if no expiration time, expires == 0 
        otc.expires = expires;
        otc.executed = false;

        let mut x = 0;
        let mut y = 0;
        let account_info = &mut ctx.remaining_accounts.iter();
        loop {
            if x == 0 || x == 5 || x == 10 || x == 15 || x == 20 {
                let token_a_mint = next_account_info(account_info)?;
                x = x+1;

                if token_a_mint.key() != ctx.accounts.seller.key() {
                    let new_account_info = next_account_info(account_info).unwrap();
                    let mut data: &[u8] =  &new_account_info.try_borrow_data()?;
                    let token_a_vault = TokenAccount::try_deserialize(&mut data)?;
                    x = x+1;
                    if token_a_vault.owner == *ctx.accounts.seller.key && token_a_vault.mint == token_a_mint.key() {
                        let nnew_account_info = next_account_info(account_info).unwrap();
                        let mut data_new: &[u8] = &nnew_account_info.try_borrow_data()?;
                        let token_a_otc_vault = TokenAccount::try_deserialize(&mut data_new)?;
                        x = x+1;

                        if token_a_otc_vault.owner == *ctx.accounts.otc_signer.key {
                            // Transfer tokens into the stake vault.
                            {
                                let cpi_ctx = CpiContext::new(
                                    ctx.accounts.token_program.to_account_info(),
                                    token::Transfer {
                                        from: new_account_info.to_account_info(),
                                        to: nnew_account_info.to_account_info(),
                                        authority: ctx.accounts.seller.to_account_info(),
                                    },
                                );
                                token::transfer(cpi_ctx, amount_a[y] as u64)?;
                            }

                            otc.sell_asset.push(
                                Asset {
                                    asset_mint: *token_a_mint.key,
                                    asset_vault: nnew_account_info.key(), // token_a_otc_vault
                                    amount: amount_a[y] as u128,
                                }
                            );

                            let token_b_mint = next_account_info(account_info)?;                            
                            x = x+1;
                            let nnnew_account_info = next_account_info(account_info).unwrap();
                            let mut data_nnew: &[u8] = &nnnew_account_info.try_borrow_data()?;
                            let token_b_vault = TokenAccount::try_deserialize(&mut data_nnew)?;
                            x = x+1;

                            if token_b_mint.key != ctx.accounts.seller.key {
                                if token_b_vault.owner == *ctx.accounts.seller.key {
    
                                    otc.buy_asset.push(Asset {
                                        asset_mint: *token_b_mint.key,
                                        asset_vault: *nnnew_account_info.key,
                                        amount: amount_b[y] as u128,
                                    });
                                    y = y+1;
                                }
                            } else {
                                y  = y+1;
                            }
                        }
                    
                    }
                }
            } else {
                break;
            }
        } 
        Ok(())
    }

    pub fn execute_otc<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteOTC<'info>>) -> Result<()> {
        let otc = &mut ctx.accounts.otc;
        let c = clock::Clock::get().unwrap();

        require!((c.unix_timestamp as u128) < (otc.expires as u128) || otc.expires == 0, ErrorCode::Expired);
        require!(otc.executed == false, ErrorCode::AlreadyExecuted);

        let mut x = 0;
        let mut y = 0;
        let account_info  = &mut ctx.remaining_accounts.iter();

        // Details:
        // token_a_mint - sell asset mint (0, 6, 12, 18, 24)
        // token_a_buyer_vault - buyer's vault to hold the token_a asset (1,7,13, 19, 25)
        // token_a_otc_vault - program holding the token_a asset (2,8,14, 20, 26)
        // token_b_mint - buy asset mint (3, 9, 15, 21, 27)
        // token_b_vault - buyer's vault holding the tokens (4, 10, 16, 22, 28)
        // token_b_seller_vault - seller's vault for holding the tokens (5, 11, 17, 23, 29)
        
        loop {
            if x == 0 || x == 6 || x == 12 || x ==  18 || x == 24 {
                let token_a_mint = next_account_info(account_info)?;
                x = x + 1;
                
                if token_a_mint.key() == otc.sell_asset[y].asset_mint && token_a_mint.key() != ctx.accounts.seller.key() {
                    let token_a_buyer_vault = next_account_info(account_info)?;
                    let token_a_otc_vault = next_account_info(account_info)?;

                    x = x + 2;
                    
                    if otc.sell_asset[y].asset_vault == token_a_otc_vault.key() {
                        {
                            let seeds = &[
                                otc.to_account_info().key.as_ref(),
                                &[otc.nonce],
                            ];
                            let otc_signer = &[&seeds[..]];
                
                            let cpi_ctx = CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                token::Transfer {
                                    from: token_a_otc_vault.to_account_info(),
                                    to: token_a_buyer_vault.to_account_info(),
                                    authority: ctx.accounts.otc_signer.to_account_info(),
                                },
                                otc_signer,
                            );
                            token::transfer(cpi_ctx, otc.sell_asset[x].amount as u64)?;
                        }
                    }
                } 
                let token_b_mint = next_account_info(account_info)?;
                x = x+1;
                if token_b_mint.key() == otc.buy_asset[y].asset_mint && token_b_mint.key() != ctx.accounts.seller.key() {
                    let token_b_vault = next_account_info(account_info)?;
                    let new_account_info = next_account_info(account_info)?;
                    let mut data: &[u8] = &new_account_info.try_borrow_data()?;
                    let token_b_seller_vault = TokenAccount::try_deserialize(&mut data)?;

                    x = x + 2;
                    
                    if otc.buy_asset[y].asset_vault == new_account_info.key() && token_b_seller_vault.owner == ctx.accounts.seller.key() {
                        {
                            let cpi_ctx = CpiContext::new(
                                ctx.accounts.token_program.to_account_info(),
                                token::Transfer {
                                    from: token_b_vault.to_account_info(),
                                    to: new_account_info.to_account_info(),
                                    authority: ctx.accounts.buyer.to_account_info(), 
                                },
                            );
                            token::transfer(cpi_ctx, otc.buy_asset[y].amount as u64)?;
                        }
                        y = y+1;
                    }
                } else {
                    y = y + 1;
                }

            } else {
                otc.executed = true;
                break;
            }
        }

        Ok(())
    }


    // Details
    // token_a_mint - token_a asset mint (0, 3, 6, 9, 12)
    // token_a_vault - seller's vault holding asset a (1, 4, 7, 10, 13)
    // token_a_otc_vault - program vault holding the asset (2, 5, 8, 11, 14)
    pub fn cancel_otc<'info>(ctx: Context<'_, '_, '_, 'info, CancelOTC<'info>>) -> Result<()> {
        let otc = &mut ctx.accounts.otc;
        let c = clock::Clock::get().unwrap();

        require!(otc.executed == false, ErrorCode::AlreadyExecuted);

        let mut x = 0;
        let mut y = 0;
        let account_info = &mut ctx.remaining_accounts.iter();

        loop {
            if x == 0 || x == 3 || x == 6 || x == 9 || x == 12 {
                let token_a_mint = next_account_info(account_info)?;
                x = x + 1;
                
                if token_a_mint.key() == otc.sell_asset[y].asset_mint && token_a_mint.key() != ctx.accounts.seller.key() {
                    let token_a_vault = next_account_info(account_info)?;
                    let token_a_otc_vault = next_account_info(account_info)?;

                    x = x + 2;
                    
                    if otc.sell_asset[y].asset_vault == token_a_otc_vault.key() {
                        {
                            let seeds = &[
                                otc.to_account_info().key.as_ref(),
                                &[otc.nonce],
                            ];
                            let otc_signer = &[&seeds[..]];
                
                            let cpi_ctx = CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                token::Transfer {
                                    from: token_a_otc_vault.to_account_info(),
                                    to: token_a_vault.to_account_info(),
                                    authority: ctx.accounts.otc_signer.to_account_info(),
                                },
                                otc_signer,
                            );
                            token::transfer(cpi_ctx, otc.sell_asset[y].amount as u64)?;
                           
                        }
                        y = y+1;
                    }
                } 
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(otc_nonce: u8)]
pub struct CreateOTC<'info> {
    pub buyer: UncheckedAccount<'info>,

    pub seller: Signer<'info>,

    // pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    // pub token_b_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_b_vault: Vec<Box<Account<'info, TokenAccount>>>,

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
pub struct ExecuteOTC<'info> {
    #[account(
        constraint = otc.buyer == *buyer.key
    )]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        constraint = otc.seller == *seller.key
    )]
    pub seller: Signer<'info>,

    // pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_a_buyer_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

    // pub token_b_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_b_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_b_seller_vault: Vec<Box<Account<'info, TokenAccount>>>,

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
pub struct CancelOTC<'info> {
    #[account(
        constraint = otc.seller == *seller.key
    )]
    pub seller: Signer<'info>,

    #[account(
        constraint = otc.buyer == *buyer.key
    )]
    pub buyer: UncheckedAccount<'info>,

    // pub token_a_mint: Vec<Box<Account<'info, Mint>>>,
    // pub token_a_vault: Vec<Box<Account<'info, TokenAccount>>>,
    // pub token_a_otc_vault: Vec<Box<Account<'info, TokenAccount>>>,

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
pub enum ErrorCode {
    #[msg("Expired")]
    Expired,
    #[msg("Already executed")]
    AlreadyExecuted,
}
