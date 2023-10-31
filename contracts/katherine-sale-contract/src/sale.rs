use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{AccountId, require};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sale {
    /// Unique ID identifier.
    pub id: u32,

    /// The slug is a unique string for the sale to recover the id.
    pub slug: String,

    /// The address of the token to be sold.
    pub sold_token_contract_address: AccountId,

    /// The address of the payment token for the sale.
    /// If the payment token is None, then only NEAR base asset is used.
    pub payment_token_contract_address: Option<AccountId>,

    /// Conversion rates
    /// How many sold tokens can be purchase using one payment token or NEAR.
    /// CAUTION: Include in the conversion rate the token DECIMALS.
    pub one_payment_token_purchase_rate: u128,

    /// For the **buyers**
    pub max_available_sold_token: Balance,
    pub required_sold_token: Balance,

    /// For the **seller**
    pub total_payment_token: Balance,

    /// Opening date for Sale.
    pub open_date_timestamp: EpochMillis,
    /// Closing date for Sale.
    pub close_date_timestamp: EpochMillis,

    /// Date when the sold tokens will be released for claim.
    /// IMPORTANT: Limit date for depositing the sold_tokens_for_buyers
    pub release_date_timestamp: EpochMillis,

    /// If sold_tokens_for_buyers < required_sold_token, by release_date
    /// then buyers can claim the payment tokens.
    pub sold_tokens_for_buyers: Balance,

    /// For the sale_owner. A percentage of the payment tokens.
    pub sale_fee: BasisPoints,

    /// Consider that the sold token was converted form NEAR or payment token.
    /// The sum of the balances should be equal to required_sold_token.
    pub claimable_sold_token_for_buyers: UnorderedMap<AccountId, Balance>,
}

impl Sale {
    pub(crate) fn new(
        id: u32,
        slug: String,
        sold_token_contract_address: AccountId,
        payment_token_contract_address: Option<AccountId>,
        one_payment_token_purchase_rate: u128,
        max_available_sold_token: Balance,
        open_date_timestamp: EpochMillis,
        close_date_timestamp: EpochMillis,
        release_date_timestamp: EpochMillis,
        sale_fee: BasisPoints,
    ) -> Self {
        Sale {
            id,
            slug,
            sold_token_contract_address,
            payment_token_contract_address,
            one_payment_token_purchase_rate,
            max_available_sold_token,
            required_sold_token: 0,
            total_payment_token: 0,
            open_date_timestamp,
            close_date_timestamp,
            release_date_timestamp,
            sold_tokens_for_buyers: 0,
            sale_fee,
            claimable_sold_token_for_buyers: UnorderedMap::new(
                StorageKey::ClaimableSoldTokens {
                    hash_id: generate_hash_id(id.to_string())
                }
            ),
        }
    }

    pub fn is_near_accepted(&self) -> bool {
        self.payment_token_contract_address.is_none()
    }

    pub(crate) fn assert_input_timestamps(&self) {
        require!(
            self.open_date_timestamp > get_current_epoch_millis()
                && self.close_date_timestamp > self.open_date_timestamp 
                && self.release_date_timestamp > self.close_date_timestamp,
            "Incorrect sale dates."
        );

    }
}

// #[near_bindgen]
impl KatherineSale {
    pub(crate) fn assert_unique_slug(&self, slug: &String) {
        assert!(
            self.sale_id_by_slug.get(slug).is_none(),
            "Slug already exists. Choose a different one!"
        );
    }

    pub(crate) fn internal_get_sale(&self, sale_id: u32) -> Sale {
        self.sales
            .get(sale_id as u64)
            .expect("Unknown sale.")
    }
}