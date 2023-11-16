use near_sdk::{BorshStorageKey, CryptoHash};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;

pub type BasisPoints = u32;
pub type EpochMillis = u64;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    ClaimableSoldTokens { hash_id: CryptoHash },
    BuyerActiveSales { hash_id: CryptoHash },
    Deposits { hash_id: CryptoHash },
    ActivesSales,
    Buyers,
    Sales,
    SalesById,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleJSON {
    pub id: u32,
    pub slug: String,
    pub sold_token_contract_address: AccountId,
    pub max_available_sold_token: U128,
    pub required_sold_token: U128,
    pub total_payment_token: U128,
    pub one_payment_token_purchase_rate: U128,
    pub open_date_timestamp: U64,
    pub close_date_timestamp: U64,
    pub release_date_timestamp: U64,
    pub sold_tokens_for_buyers: U128,
    
    pub min_deposit_amount: U128,
    pub payment_token_contract_address: Option<AccountId>,
    pub payment_token_unit: U128,
    pub sale_fee: BasisPoints,
    pub total_fees: U128,
    
    pub is_in_near: bool,
    pub is_active: bool,
}
