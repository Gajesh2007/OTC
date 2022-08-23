use anchor_lang::prelude::*;

#[account]
pub struct Otc {
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
