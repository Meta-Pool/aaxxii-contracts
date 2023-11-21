pub mod now;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, PanicOnDefault};

mod types;
use crate::types::EpochMillis;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self { Self {} }

    /// Use the fail function to force a failure/resolve flow in a cross-contract-call.
    pub fn fail(&self) {
        env::panic_str("Forcing a panic!");
    }

    pub fn get_now(&self) -> EpochMillis {
        Self::get_current_epoch_millis()
    }

    pub fn get_future(&self, mins: u32) -> EpochMillis {
        Self::get_current_epoch_millis() + (mins * 60 * 1_000) as u64
    }

    #[inline]
    pub fn get_current_epoch_millis() -> EpochMillis {
        return env::block_timestamp() / 1_000_000;
    }
}
