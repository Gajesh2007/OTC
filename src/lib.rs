use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod state;
pub use errors::ErrorCode;

pub use instructions::*;
pub use state::*;

declare_id!("2qtPWREXxQbTZidekTeU7e5nphygjoJuZRMJwwYL8pq5");

#[program]
pub mod nftotc {
    use super::*;

    pub fn create_otc<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateOTC<'info>>,
        otc_nonce: u8,
        amount_a: Vec<u128>,
        amount_b: Vec<u128>,
        expires: u128,
    ) -> Result<()> {
        create_otc::handler(ctx, otc_nonce, amount_a, amount_b, expires)
    }

    pub fn execute_otc<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteOTC<'info>>) -> Result<()> {
        execute_otc::handler(ctx)
    }

    pub fn cancel_otc<'info>(ctx: Context<'_, '_, '_, 'info, CancelOTC<'info>>) -> Result<()> {
        cancel_otc::handler(ctx)
    }
}
