use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{require, env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult};
use std::convert::TryInto;

use crate::buyer::*;
use crate::constants::*;
use crate::sale::*;
use crate::types::*;
use crate::utils::*;

mod buyer;
mod constants;
mod deposit;
mod internal;
mod sale;
mod types;
mod utils;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineSaleContract {
    pub owner_id: AccountId,
    pub buyers: UnorderedMap<AccountId, Buyer>,
    pub sales: Vector<Sale>,
    pub sale_id_by_slug: UnorderedMap<String, u32>,
    pub active_sales: UnorderedSet<u32>,

    /// ******** Default sales parameters *********
    /// * If updated, only new sales are affected *
    /// *******************************************
    /// Min amount accepted for sale in NEAR.
    pub min_deposit_amount_in_near: Balance,
    /// Min amount accepted for sale in payment token.
    pub min_deposit_amount_in_payment_token: Balance,
    /// The address of the payment token for the sale.
    pub payment_token_contract_address: AccountId,
    /// e.g. 1.0 USDT == 1_000_000 unit.
    pub payment_token_unit: u128,
    /// % of the total sale for the owner_id.
    pub sale_fee: BasisPoints,
}

#[near_bindgen]
impl KatherineSaleContract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        min_deposit_amount_in_near: U128,
        min_deposit_amount_in_payment_token: U128,
        payment_token_contract_address: AccountId,
        payment_token_unit: U128,
        sale_fee: BasisPoints,
    ) -> Self {
        check_basis_points(sale_fee);
        Self {
            owner_id,
            buyers: UnorderedMap::new(StorageKey::Buyers),
            sales: Vector::new(StorageKey::Sales),
            sale_id_by_slug: UnorderedMap::new(StorageKey::SalesById),
            min_deposit_amount_in_near: min_deposit_amount_in_near.0,
            active_sales: UnorderedSet::new(StorageKey::ActivesSales),
            min_deposit_amount_in_payment_token: min_deposit_amount_in_payment_token.0,
            payment_token_contract_address,
            payment_token_unit: payment_token_unit.0,
            sale_fee,
        }
    }

    // ****************************
    // * Update contract settings *
    // ****************************

    pub fn update_owner_id(&mut self, new_owner_id: AccountId) {
        self.assert_only_owner();
        self.owner_id = new_owner_id;
    }

    pub fn update_min_deposit_amount_in_near(&mut self, new_value: U128) {
        self.assert_only_owner();
        self.min_deposit_amount_in_near = new_value.0;
    }

    pub fn update_min_deposit_amount_in_payment_token(&mut self, new_value: U128) {
        self.assert_only_owner();
        self.min_deposit_amount_in_payment_token = new_value.0;
    }

    pub fn update_payment_token_contract_address(&mut self, new_value: AccountId) {
        self.assert_only_owner();
        self.payment_token_contract_address = new_value;
    }

    pub fn update_payment_token_unit(&mut self, new_value: U128) {
        self.assert_only_owner();
        self.payment_token_unit = new_value.0;
    }

    pub fn update_default_sales_fee(&mut self, new_value: BasisPoints) {
        self.assert_only_owner();
        check_basis_points(new_value);
        self.sale_fee = new_value;
    }

    // *******************
    // * Sales operation *
    // *******************

    #[payable]
    pub fn create_sale(
        &mut self,
        slug: String,
        is_in_near: bool,
        sold_token_contract_address: AccountId,
        // "one" references the payment token UNIT
        one_payment_token_purchase_rate: u128,
        max_available_sold_token: Balance,
        open_date_timestamp: EpochMillis,
        close_date_timestamp: EpochMillis,
        release_date_timestamp: EpochMillis,
    ) -> u32 {
        self.assert_only_owner();
        self.assert_unique_slug(&slug);
        Sale::assert_storage_is_covered();
        let id = self.sales.len() as u32;

        let (
            min_deposit_amount,
            payment_token_contract_address,
            payment_token_unit
        ) = if is_in_near {
            (self.min_deposit_amount_in_near, None, NEAR)
        } else {
            (
                self.min_deposit_amount_in_payment_token,
                Some(self.payment_token_contract_address.clone()),
                self.payment_token_unit
            )
        };

        let sale = Sale::new(
            id,
            slug,
            sold_token_contract_address,
            one_payment_token_purchase_rate,
            max_available_sold_token,
            open_date_timestamp,
            close_date_timestamp,
            release_date_timestamp,
            min_deposit_amount,
            payment_token_contract_address,
            payment_token_unit,
            self.sale_fee,
        );

        sale.assert_input_timestamps();
        self.sales.push(&sale);
        self.sale_id_by_slug
            .insert(&sale.slug, &sale.id);
        self.active_sales.insert(&sale.id);
        sale.id.into()
    }

    // ***********************
    // * Payments using NEAR *
    // ***********************

    #[payable]
    pub fn purchase_token_with_near(&mut self, sale_id: u32) {
        let mut sale = self.internal_get_sale(sale_id);
        let amount = env::attached_deposit();
        let buyer_id = env::predecessor_account_id();

        require!(sale.is_near_accepted());
        self.process_payment_tokens_deposit(&buyer_id, amount, &mut sale);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;