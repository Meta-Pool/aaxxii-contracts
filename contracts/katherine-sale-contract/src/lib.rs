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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct KatherineSale {
    pub owner_id: AccountId,
    pub buyers: UnorderedMap<AccountId, Buyer>,
    pub sales: Vector<Sale>,
    pub sale_id_by_slug: UnorderedMap<String, u32>,

    /// Min amount accepted for sale
    pub min_deposit_amount: Balance,

    // Active kickstarter projects.
    pub active_sales: UnorderedSet<u32>,
}