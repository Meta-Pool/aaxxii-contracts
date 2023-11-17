use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{BorshStorageKey, Gas, Balance, CryptoHash, StorageUsage};

use crate::types::BasisPoints;

pub const NEAR: Balance = 1_000_000_000_000_000_000_000_000;
pub const BASIS_POINT: BasisPoints = 10_000;

pub const STORAGE_PER_SALE: u128 = NEAR / 100;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(47 * TGAS);
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(11 * TGAS);