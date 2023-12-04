use near_sdk::Gas;

pub const E24: u128 = 1_000_000_000_000_000_000_000_000;
pub const YOCTO_UNITS: u128 = E24;
pub const TGAS: u64 = 1_000_000_000_000;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(47 * TGAS);
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(11 * TGAS);
