use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    require, env, assert_one_yocto, near_bindgen, AccountId, Balance,
    PanicOnDefault, PromiseResult, Promise
};
use std::convert::TryInto;

use crate::buyer::*;
use crate::constants::*;
use crate::sale::*;
use crate::types::*;
use crate::utils::*;
use crate::interface::*;

mod buyer;
pub mod constants;
mod deposit;
mod interface;
mod internal;
mod sale;
mod types;
mod utils;
mod withdraw;

/// There are 5 possible stages of a Sale depending on the date:
///
///      open                close              release
///       |-------------------|-------------------|
/// Stages:
///   0            1                    2            3
///
/// We'll be using the stage number as convention.

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
    pub treasury_id: AccountId,
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
        treasury_id: AccountId,
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
            treasury_id,
            sale_fee,
        }
    }

    // ****************************
    // * Update contract settings *
    // ****************************

    #[payable]
    pub fn update_owner_id(&mut self, new_value: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.owner_id = new_value;
    }

    #[payable]
    pub fn update_treasury_id(&mut self, new_value: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.treasury_id = new_value;
    }

    /// This update will only affects the next sales, not currents.
    #[payable]
    pub fn update_min_deposit_amount_in_near(&mut self, new_value: U128) {
        assert_one_yocto();
        self.assert_only_owner();
        self.min_deposit_amount_in_near = new_value.0;
    }

    /// This update will only affects the next sales, not currents.
    #[payable]
    pub fn update_min_deposit_amount_in_payment_token(&mut self, new_value: U128) {
        assert_one_yocto();
        self.assert_only_owner();
        self.min_deposit_amount_in_payment_token = new_value.0;
    }

    /// This update will only affects the next sales, not currents.
    #[payable]
    pub fn update_payment_token_contract_address(&mut self, new_value: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.payment_token_contract_address = new_value;
    }

    /// This update will only affects the next sales, not currents.
    #[payable]
    pub fn update_payment_token_unit(&mut self, new_value: U128) {
        assert_one_yocto();
        self.assert_only_owner();
        self.payment_token_unit = new_value.0;
    }

    /// This update will only affects the next sales, not currents.
    #[payable]
    pub fn update_default_sales_fee(&mut self, new_value: BasisPoints) {
        assert_one_yocto();
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

    /// Buyers can purchase tokens only within the funding period.
    /// Same for buyers ft payment token deposit [deposit.rs]
    /// Only callable during `stage 1`.
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

    /// When a buyer withdraw form a sale ALL the claimable tokens are send to
    /// the buyer, and the deposit is removed from `sale.deposits`.
    /// Only callable during `stage 3`.
    pub fn withdraw_tokens(&mut self, sale_id: u32) -> Promise {
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_release_period();

        let buyer_id = env::predecessor_account_id();
        // Important: Claimable tokens and buyer deposit are removed.
        let claimable = sale
            .claimable_sold_token_for_buyers
            .remove(&buyer_id)
            .expect("No claimable tokens.");
        let deposit = sale.deposits.remove(&buyer_id).expect("No deposit.");
        require!(claimable > 0 && deposit > 0);

        let mut buyer = self.internal_get_buyer(&buyer_id);
        buyer.supporting_sales.remove(&sale.id);
        self.buyers.insert(&buyer_id, &buyer);

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

    /// Only callable during `stage 2 and 3`, only if sold tokens are covered.
    /// Payments are being send to the `treasury_id`.
    pub fn collect_payments(&mut self, sale_id: u32) -> Promise {
        self.assert_only_owner();
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_close_period();
        self.remove_sale_from_active_list(sale_id);
        require!(sale.total_payment_token > 0, "Nothing to collect.");
        require!(sale.are_sold_tokens_covered(), "Deposit all the sold tokens.");

        self.internal_collect_payments(&mut sale)
    }

    /// Only callable after the owner raw `collect_payments`.
    /// Fees are being send to the `treasury_id`.
    pub fn collect_fees(&mut self, sale_id: u32) -> Promise {
        self.assert_only_owner();
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_close_period();
        require!(sale.total_fees > 0, "Nothing to collect.");

        self.internal_collect_fees(&mut sale)
    }

    pub fn withdraw_excess_sold_tokens(&mut self, sale_id: u32) -> Promise {
        self.assert_only_owner();
        let mut sale = self.internal_get_sale(sale_id);
        sale.assert_after_close_period();
        self.remove_sale_from_active_list(sale_id);
        
        let excess = if sale.are_sold_tokens_covered() {
            // Check if sale has more tokens than what it needs to cover deposits.
            // Return excess.
            sale.sold_tokens_for_buyers - sale.required_sold_token
        } else {
            // IMPORTANT: If we're in stage 2, then seller can still deposit more sold
            // tokens to cover the deposits.
            // Note how the `excess` here are the remaining amount of sold tokens.
            sale.assert_after_release_period();
            sale.sold_tokens_for_buyers
        };

        require!(excess > 0, "No excess amount of sold tokens.");
        self.seller_withdraw_excess_sold_tokens(excess, &mut sale)
    }

    // ********
    // * View *
    // ********

    pub fn get_sale_fee(&self, sale_id: u32) -> U128 {
        let sale = self.internal_get_sale(sale_id);
        U128::from(sale.total_fees)
    }

    pub fn get_active_sales(
        &self,
        from_index: u32,
        limit: u32
    ) -> Vec<SaleJSON> {
        let sales = self.active_sales.to_vec();
        let sales_len = sales.len() as u32;
        let mut result = Vec::<SaleJSON>::new();
        if from_index >= sales_len { return result; }
        for index in from_index..std::cmp::min(from_index + limit, sales_len) {
            let sale_id = sales.get(index as usize).expect("Out of index!");
            let sale = self.internal_get_sale(*sale_id);
            // Even if the sale is in the `self.active_sales` list it might be inactive.
            if sale.is_active() {
                result.push(sale.to_json());
            }
        }
        result
    }

    pub fn get_sale(&self, sale_id: u32) -> SaleJSON {
        self.internal_get_sale(sale_id).to_json()
    }

    pub fn get_sales(&self, from_index: u32, limit: u32) -> Vec<SaleJSON> {
        let sales_len = self.get_number_of_sales();
        let mut result = Vec::<SaleJSON>::new();
        if from_index >= sales_len { return result; }
        for index in from_index..std::cmp::min(from_index + limit, sales_len) {
            let sale = self.sales.get(index as u64).expect("Out of index!");
            result.push(sale.to_json());
        }
        result
    }

    pub fn get_sale_id_from_slug(&self, slug: String) -> u32 {
        match self.sale_id_by_slug.get(&slug) {
            Some(id) => id,
            None => panic!("Nonexistent slug!"),
        }
    }

    pub fn get_buyer_sales(&self, buyer_id: AccountId) -> Vec<u32> {
        let buyer = self.internal_get_buyer(&buyer_id);
        buyer.supporting_sales.to_vec()
    }

    pub fn get_buyer_sales_list(
        &self,
        buyer_id: AccountId,
        from_index: u32,
        limit: u32,
    ) -> Vec<SaleJSON> {
        let buyer = self.internal_get_buyer(&buyer_id);
        let sales = buyer.supporting_sales.to_vec();
        let sales_len = sales.len() as u32;
        let mut result = Vec::<SaleJSON>::new();
        if from_index >= sales_len { return result; }
        for index in from_index..std::cmp::min(from_index + limit, sales_len) {
            let sale_id = sales.get(index as usize).expect("Out of index!");
            let sale = self.internal_get_sale(*sale_id);
            result.push(sale.to_json());
        }
        result
    }

    pub fn get_buyers(
        &self,
        from_index: u32,
        limit: u32
    ) -> Vec<AccountId> {
        let keys = self.buyers.keys_as_vector();
        let to = std::cmp::min(from_index + limit, keys.len().try_into().unwrap());
        return (from_index..to).map(|index| keys.get(index as u64).unwrap()).collect();
    }

    pub fn get_number_of_sales(&self) -> u32 {
        self.sales.len().try_into().unwrap()
    }

    pub fn get_number_of_buyers(&self) -> u32 {
        self.sales.len().try_into().unwrap()
    }

    pub fn get_number_of_buyers_for_sale(&self, sale_id: u32) -> u32 {
        let sale = self.internal_get_sale(sale_id);
        sale.deposits.len().try_into().unwrap()
    }

    pub fn get_buyer_claimable_sold_token(
        &self,
        buyer_id: AccountId,
        sale_id: u32
    ) -> U128 {
        let sale = self.internal_get_sale(sale_id);
        U128::from(sale.get_buyer_claimable_sold_token(&buyer_id))
    }

    pub fn get_buyer_deposit(&self, buyer_id: AccountId, sale_id: u32) -> U128 {
        let sale = self.internal_get_sale(sale_id);
        U128::from(sale.get_buyer_deposit(&buyer_id))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;