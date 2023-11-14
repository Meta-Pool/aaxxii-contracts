use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{require, env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult, Promise};
use std::convert::TryInto;

use crate::buyer::*;
use crate::constants::*;
use crate::sale::*;
use crate::types::*;
use crate::utils::*;
use crate::interface::*;

mod buyer;
mod constants;
mod deposit;
mod interface;
mod internal;
mod sale;
mod types;
mod utils;
mod withdraw;

/// Time in this contract is measured in Milliseconds.

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
        one_payment_token_purchase_rate: U128,
        max_available_sold_token: U128,
        open_date_timestamp: U64,
        close_date_timestamp: U64,
        release_date_timestamp: U64,
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
            one_payment_token_purchase_rate.0,
            max_available_sold_token.0,
            open_date_timestamp.0,
            close_date_timestamp.0,
            release_date_timestamp.0,
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

    // *******************
    // * Buyers Withdraw *
    // *******************

    pub fn withdraw_tokens(&mut self, sale_id: u32) -> Promise {
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_release_period();

        let buyer_id = env::predecessor_account_id();
        let claimable = sale.claimable_sold_token_for_buyers.remove(&buyer_id).expect("No claimable tokens.");
        let deposit = sale.deposits.remove(&buyer_id).expect("No deposit.");
        require!(claimable > 0 && deposit > 0);

        if sale.are_sold_tokens_covered() {
            self.internal_buyer_withdraw_sold_tokens(
                buyer_id,
                claimable,
                deposit,
                &mut sale
            )
        } else {
            self.internal_buyer_withdraw_payment_token(
                buyer_id,
                claimable,
                deposit,
                &mut sale
            )
        }
    }

    // *******************
    // * Seller Withdraw *
    // *******************

    pub fn collect_tokens(&mut self, sale_id: u32) -> Promise {
        self.assert_only_owner();
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_close_period();
        require!(sale.are_sold_tokens_covered(), "Deposit all the sold tokens");

        if sale.is_near_accepted() {
            self.internal_seller_withdraw_near()
        } else {
            self.internal_seller_withdraw_payment_token()
        }
    }

    pub fn withdraw_excess_sold_tokens(&mut self, sale_id: u32) -> Promise {
        // Only after the close date.
        unimplemented!();
    }

    // ********
    // * View *
    // ********

    pub fn get_number_of_sales(&self) -> u32 {
        self.sales.len().try_into().unwrap()
    }

    pub fn get_claimable_sold_token_for_buyers(
        &self,
        buyer_id: AccountId,
        sale_id: u32
    ) -> U128 {
        let sale = self.internal_get_sale(sale_id);
        U128::from(sale.get_claimable_sold_token_for_buyers(&buyer_id))
    }

    pub fn get_deposits(&self, buyer_id: AccountId, sale_id: u32) -> U128 {
        let sale = self.internal_get_sale(sale_id);
        U128::from(sale.get_deposits(&buyer_id))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;