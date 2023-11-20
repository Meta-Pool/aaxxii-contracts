use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{AccountId, require};

use crate::constants::STORAGE_PER_SALE;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PaymentConfig {
    pub min_deposit_amount: Balance,
    // If None, the payment token is in NEAR.
    pub payment_token_contract_address: Option<AccountId>,
    pub payment_token_unit: u128,
    pub sale_fee: BasisPoints,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sale {
    /// Unique ID identifier.
    pub id: u32,

    /// The slug is a unique string for the sale to recover the id.
    pub slug: String,

    /// The address of the token to be sold.
    pub sold_token_contract_address: AccountId,

    /// For the **buyers**
    pub max_available_sold_token: Balance, // Remains constant.
    pub required_sold_token: Balance,

    /// For the **seller**
    pub total_payment_token: Balance,

    /// Conversion rates
    /// How many sold tokens can be purchase using one payment token or NEAR.
    /// CAUTION: Include in the conversion rate the token DECIMALS.
    /// "one" in NEAR is 1e24 // "one" in USDT is 1_000_000 [see payment_token_unit].
    pub one_payment_token_purchase_rate: u128,

    /// Opening date for Sale.
    pub open_date_timestamp: EpochMillis,
    /// Closing date for Sale.
    pub close_date_timestamp: EpochMillis,

    /// Date when the sold tokens will be released for claim.
    /// IMPORTANT: Limit date for depositing the `sold_tokens_for_buyers`.
    pub release_date_timestamp: EpochMillis,

    /// If `sold_tokens_for_buyers` < `required_sold_token`, by release_date
    /// then buyers can claim the payment tokens.
    pub sold_tokens_for_buyers: Balance,

    /// Consider that the sold token was converted form NEAR or payment token.
    /// The sum of the balances should be equal to `required_sold_token`.
    pub claimable_sold_token_for_buyers: UnorderedMap<AccountId, Balance>,
    /// The sum of the balances should be equal to `total_payment_token`.
    pub deposits: UnorderedMap<AccountId, Balance>,

    /// The payment config is inherit when sale is created.
    pub payment_config: PaymentConfig,

    pub total_fees: u128,
}

impl Sale {
    pub(crate) fn new(
        id: u32,
        slug: String,
        sold_token_contract_address: AccountId,
        one_payment_token_purchase_rate: u128,
        max_available_sold_token: Balance,
        open_date_timestamp: EpochMillis,
        close_date_timestamp: EpochMillis,
        release_date_timestamp: EpochMillis,
        // Create payment config.
        min_deposit_amount: Balance,
        payment_token_contract_address: Option<AccountId>,
        payment_token_unit: u128,
        sale_fee: BasisPoints,
    ) -> Self {
        Sale {
            id,
            slug,
            sold_token_contract_address,
            one_payment_token_purchase_rate,
            max_available_sold_token,
            required_sold_token: 0,
            total_payment_token: 0,
            open_date_timestamp,
            close_date_timestamp,
            release_date_timestamp,
            sold_tokens_for_buyers: 0,
            claimable_sold_token_for_buyers: UnorderedMap::new(
                StorageKey::ClaimableSoldTokens {
                    hash_id: generate_hash_id(id.to_string())
                }
            ),
            deposits: UnorderedMap::new(
                StorageKey::Deposits {
                    hash_id: generate_hash_id(id.to_string())
                }
            ),
            payment_config: PaymentConfig {
                min_deposit_amount,
                payment_token_contract_address,
                payment_token_unit,
                sale_fee,
            },
            total_fees: 0,
        }
    }

    /// The sale will only have a payment token if it's not denominated in NEAR.
    pub(crate) fn get_payment_token(&self) -> AccountId {
        self.payment_config.payment_token_contract_address.clone().unwrap()
    }

    pub(crate) fn get_sold_token(&self) -> AccountId {
        self.sold_token_contract_address.clone()
    }

    pub(crate) fn assert_storage_is_covered() {
        assert!(
            env::attached_deposit() >= STORAGE_PER_SALE,
            "The required NEAR to create a sale is {}",
            STORAGE_PER_SALE
        );
    }

    pub(crate) fn get_buyer_claimable_sold_token(
        &self,
        buyer_id: &AccountId
    ) -> Balance {
        match self.claimable_sold_token_for_buyers.get(buyer_id) {
            Some(amount) => amount,
            None => 0,
        }
    }

    pub(crate) fn get_buyer_deposit(
        &self,
        buyer_id: &AccountId
    ) -> Balance {
        match self.deposits.get(buyer_id) {
            Some(amount) => amount,
            None => 0,
        }
    }

    pub(crate) fn from_payment_to_sold_token(&self, amount: u128) -> u128 {
        proportional(
            amount,
            self.one_payment_token_purchase_rate,
            self.payment_config.payment_token_unit
        )
    }

    #[inline]
    pub(crate) fn is_near_accepted(&self) -> bool {
        self.payment_config.payment_token_contract_address.is_none()
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        get_current_epoch_millis() < self.close_date_timestamp
    }
    
    pub(crate) fn is_within_funding_period(&self) -> bool {
        let now = get_current_epoch_millis();
        now < self.close_date_timestamp && now >= self.open_date_timestamp
    }

    pub(crate) fn are_sold_tokens_covered(&self) -> bool {
        self.required_sold_token <= self.sold_tokens_for_buyers
    }

    // **************** 
    // * Sale Asserts *
    // **************** 

    #[inline]
    pub(crate) fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.payment_config.min_deposit_amount,
            "minimum deposit amount is {}",
            self.payment_config.min_deposit_amount
        );
    }

    #[inline]
    pub(crate) fn assert_input_timestamps(&self) {
        require!(
            self.open_date_timestamp > get_current_epoch_millis()
                && self.close_date_timestamp > self.open_date_timestamp 
                && self.release_date_timestamp > self.close_date_timestamp,
            "Incorrect sale dates."
        );
    }

    #[inline]
    pub(crate) fn assert_within_funding_period(&self) {
        require!(
            self.is_within_funding_period(),
            "Not within the funding period."
        );
    }

    #[inline]
    pub(crate) fn assert_after_release_period(&self) {
        require!(
            get_current_epoch_millis() >= self.release_date_timestamp,
            "Only after release period."
        );
    }

    #[inline]
    pub(crate) fn assert_after_close_period(&self) {
        require!(
            get_current_epoch_millis() > self.close_date_timestamp,
            "Only after close period."
        );
    }

    pub(crate) fn to_json(&self) -> SaleJSON {
        SaleJSON {
            id: self.id,
            slug: self.slug.clone(),
            sold_token_contract_address: self.sold_token_contract_address.clone(),
            max_available_sold_token: U128::from(self.max_available_sold_token),
            required_sold_token: U128::from(self.required_sold_token),
            total_payment_token: U128::from(self.total_payment_token),
            one_payment_token_purchase_rate: U128::from(self.one_payment_token_purchase_rate),
            open_date_timestamp: U64::from(self.open_date_timestamp),
            close_date_timestamp: U64::from(self.close_date_timestamp),
            release_date_timestamp: U64::from(self.release_date_timestamp),
            sold_tokens_for_buyers: U128::from(self.sold_tokens_for_buyers),
            min_deposit_amount: U128::from(self.payment_config.min_deposit_amount),
            payment_token_contract_address: self.payment_config.payment_token_contract_address.clone(),
            payment_token_unit: U128::from(self.payment_config.payment_token_unit),
            sale_fee: self.payment_config.sale_fee,
            total_fees: U128::from(self.total_fees),
            is_in_near: self.is_near_accepted(),
            is_active: self.is_active()
        }
    }
}

impl KatherineSaleContract {
    pub(crate) fn assert_unique_slug(&self, slug: &String) {
        assert!(
            self.sale_id_by_slug.get(slug).is_none(),
            "Slug already exists. Choose a different one!"
        );
    }

    pub(crate) fn internal_get_sale(&self, sale_id: u32) -> Sale {
        self.sales.get(sale_id as u64).expect("Unknown sale.")
    }

    pub(crate) fn remove_sale_from_active_list(&mut self, sale_id: u32) {
        self.active_sales.remove(&sale_id);
    }
}