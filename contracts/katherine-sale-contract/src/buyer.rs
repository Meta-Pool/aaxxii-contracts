use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{AccountId, require};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Buyer {
    pub active_sales: UnorderedSet<u32>
}

impl Buyer {
    pub fn new(id: &AccountId) -> Self {
        Self {
            active_sales: UnorderedSet::new(
                StorageKey::BuyerActiveSales {
                    hash_id: generate_hash_id(id.to_string())
                }
            ),
        }
    }

    pub fn in_sale(&self, sale_id: u32) -> bool {
        return self.active_sales.contains(&sale_id)
    }

    /// When the buyer.is_empty() it will be removed.
    pub fn is_empty(&self) -> bool {
        return self.active_sales.is_empty();
    }
}

impl KatherineSale {
    /// Inner method to get the given buyer or a new default value buyer.
    pub(crate) fn internal_get_buyer(
        &self,
        buyer_id: &AccountId
    ) -> Buyer {
        self.buyers.get(buyer_id).unwrap_or(Buyer::new(buyer_id))
    }
}
