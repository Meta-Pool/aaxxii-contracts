use crate::*;
use near_sdk::require;

impl KatherineSaleContract {
    #[inline]
    pub(crate) fn assert_only_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id, "Only owner.");
    }
}