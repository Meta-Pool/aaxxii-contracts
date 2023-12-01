use near_sdk::collections::UnorderedMap;
use uint::construct_uint;
use near_sdk::{AccountId, Balance};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::{BorshStorageKey, Gas, CryptoHash};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::utils::generate_hash_id;

pub type VoterId = AccountId;
pub type VotingPower = u128;
pub type Days = u16;
pub type ContractAddress = AccountId;
pub type VotableObjId = String;
pub type EpochMillis = u64;
pub type PositionIndex = u16;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FtDetails {
    pub owners: UnorderedMap<AccountId, u128>,
    pub accum_ft_distributed_for_claims: u128,
    pub total_unclaimed_ft: u128,
}

impl FtDetails {
    pub fn new(token_address: &AccountId) -> Self {
        Self {
            owners: UnorderedMap::new(StorageKey::OwnersFt {
                hash_id: generate_hash_id(token_address.to_string())
            }),
            accum_ft_distributed_for_claims: 0,
            total_unclaimed_ft: 0,
        }
    }
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    LockingPosition { hash_id: CryptoHash },
    VotePosition { hash_id: CryptoHash },
    Stakers,
    Votes,
    ContractVotes { hash_id: CryptoHash },
    VoterVotes { hash_id: CryptoHash },
    OwnersFt { hash_id: CryptoHash },

    ClaimableNear,
    ClaimableFt,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LockingPositionJSON {
    pub index: Option<PositionIndex>,
    pub amount: U128,
    pub locking_period: Days,
    pub voting_power: U128,
    pub unlocking_started_at: Option<EpochMillis>,
    pub is_unlocked: bool,
    pub is_unlocking: bool,
    pub is_locked: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VotableObjectJSON {
    pub votable_contract: String,
    pub id: VotableObjId,
    pub current_votes: U128
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VotePositionJSON {
    pub votable_address: AccountId,
    pub votable_object_id: String,
    pub voting_power: U128
}
