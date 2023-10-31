use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseResult};
use std::convert::TryInto;

use crate::sale::*;
use crate::buyer::*;
use crate::types::*;
use crate::utils::*;

mod sale;
mod buyer;
mod types;
mod utils;
mod internal;
mod deposit;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineSale {
    pub owner_id: AccountId,
    pub buyers: UnorderedMap<AccountId, Buyer>,
    pub sales: Vector<Sale>,
    pub sale_id_by_slug: UnorderedMap<String, u32>,

    /// Min amount accepted for sale
    pub min_deposit_amount: Balance,

    pub active_sales: UnorderedSet<u32>,

    pub default_sales_fee: BasisPoints,
}

#[near_bindgen]
impl KatherineSale {
    #[init]
    pub fn new(
        owner_id: AccountId,
        min_deposit_amount: U128,
        default_sales_fee: BasisPoints,
    ) -> Self {
        check_basis_points(default_sales_fee);
        Self {
            owner_id,
            buyers: UnorderedMap::new(StorageKey::Buyers),
            sales: Vector::new(StorageKey::Sales),
            sale_id_by_slug: UnorderedMap::new(StorageKey::SalesById),
            min_deposit_amount: min_deposit_amount.0,
            active_sales: UnorderedSet::new(StorageKey::ActivesSales),
            default_sales_fee,
        }
    }

    // ****************************
    // * Update contract settings *
    // ****************************

    pub fn update_owner_id(&mut self, new_owner_id: AccountId) {
        self.assert_only_owner();
        self.owner_id = new_owner_id;
    }

    pub fn update_min_deposit_amount(&mut self, new_value: U128) {
        self.assert_only_owner();
        self.min_deposit_amount = new_value.0;
    }

    pub fn update_default_sales_fee(&mut self, new_value: BasisPoints) {
        self.assert_only_owner();
        check_basis_points(new_value);
        self.default_sales_fee = new_value;
    }

    // *******************
    // * Sales Operation *
    // *******************

    pub fn create_sale(
        &mut self,
        slug: String,
        sold_token_contract_address: AccountId,
        payment_token_contract_address: Option<AccountId>,
        one_payment_token_purchase_rate: u128,
        max_available_sold_token: Balance,
        open_date_timestamp: EpochMillis,
        close_date_timestamp: EpochMillis,
        release_date_timestamp: EpochMillis,
        override_sale_fee: Option<BasisPoints>,
    ) -> u32 {
        self.assert_only_owner();
        self.assert_unique_slug(&slug);
        let id = self.sales.len() as u32;

        let sale = Sale::new(
            id,
            slug,
            sold_token_contract_address,
            payment_token_contract_address,
            one_payment_token_purchase_rate,
            max_available_sold_token,
            open_date_timestamp,
            close_date_timestamp,
            release_date_timestamp,
            override_sale_fee.unwrap_or(self.default_sales_fee),
        );

        sale.assert_input_timestamps();
        self.sales.push(&sale);
        self.sale_id_by_slug
            .insert(&sale.slug, &sale.id);
        self.active_sales.insert(&sale.id);
        sale.id.into()
    }
}