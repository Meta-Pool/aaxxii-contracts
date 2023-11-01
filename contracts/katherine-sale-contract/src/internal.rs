use crate::*;
// use crate::interface::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, require};

impl KatherineSaleContract {
    #[inline]
    pub(crate) fn assert_only_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id, "Only owner.");
    }
}