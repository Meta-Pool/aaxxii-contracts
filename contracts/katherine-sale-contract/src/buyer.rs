use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Buyer {
    pub supporting_sales: UnorderedSet<u32>
}

impl Buyer {
    pub(crate) fn new(id: &AccountId) -> Self {
        Self {
            supporting_sales: UnorderedSet::new(
                StorageKey::BuyerActiveSales {
                    hash_id: generate_hash_id(id.to_string())
                }
            ),
        }
    }

    /// When the buyer.is_empty() it will be removed.
    pub(crate) fn is_empty(&self) -> bool {
        return self.supporting_sales.is_empty();
    }
}

impl KatherineSaleContract {
    /// Inner method to get the given buyer or a new default value buyer.
    pub(crate) fn internal_get_buyer(
        &self,
        buyer_id: &AccountId
    ) -> Buyer {
        self.buyers.get(buyer_id).unwrap_or(Buyer::new(buyer_id))
    }
}
