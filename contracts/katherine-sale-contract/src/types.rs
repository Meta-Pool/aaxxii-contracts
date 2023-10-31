use near_sdk::{BorshStorageKey, CryptoHash};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use uint::construct_uint;

pub type BasisPoints = u32;
pub type EpochMillis = u64;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    ClaimableSoldTokens { hash_id: CryptoHash },
    Buyer { hash_id: CryptoHash }
}