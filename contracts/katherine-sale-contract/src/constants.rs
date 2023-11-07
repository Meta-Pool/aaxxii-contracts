use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{BorshStorageKey, Gas, Balance, CryptoHash, StorageUsage};

pub const NEAR: Balance = 1_000_000_000_000_000_000_000_000;

pub const STORAGE_PER_SALE: u128 = NEAR / 2;