use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, PromiseOrValue};
use near_sdk::{serde_json, ONE_NEAR};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

const E24: u128 = ONE_NEAR;
const E20: Balance = 100_000_000_000_000_000_000;

#[near_bindgen]
impl FungibleTokenReceiver for StakingPositionContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount = amount.0;

        // deposit for-claims, msg == "for-claims" means META to be later distributed to voters
        if msg.len() >= 11 && &msg[..11] == "for-claims:" {
            match serde_json::from_str(&msg[11..]) {
                Ok(info) => self.distribute_for_claims(amount, &info),
                Err(_) => panic!("Err parsing msg for-claims"),
            };
        }
        // else, user deposit to lock
        else {
            let locking_period = match msg.parse::<Days>() {
                Ok(days) => days,
                Err(_) => panic!("Err parsing locking_period from msg. Must be u16"),
            };

            assert_eq!(
                env::predecessor_account_id(),
                self.aaxxii_token_contract_address,
                "This contract only works with META from {}",
                self.aaxxii_token_contract_address
            );

            self.assert_min_deposit_amount(amount);
            log!("DEPOSIT: {} META deposited from {}", amount, &sender_id,);
            let mut staker = self.internal_get_staker(sender_id);
            self.deposit_locking_position(amount, locking_period, &mut staker);
        }
        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

impl StakingPositionContract {
    // distributes meta from self.meta_to_distribute between existent voters
    // called from ft_on_transfer
    pub(crate) fn distribute_for_claims(
        &mut self,
        total_amount: u128,
        distribute_info: &Vec<(String, u64)>,
    ) {
        let mut total_distributed = 0;
        let token_address = env::predecessor_account_id();

        // Meta Token
        // TODO: if the contract is aaxxii token, send it directly to the staker.balance.
        // if token_address == self.meta_token_contract_address {
        if token_address == self.aaxxii_token_contract_address {
            for item in distribute_info {
                // in case of META, item.1 is integer META
                let amount = item.1 as u128 * E24;
                self.add_claimable_meta(&AccountId::new_unchecked(item.0.clone()), amount);
                total_distributed += amount;
            }
            // self.accum_ft_distributed_for_claims += total_distributed;

        // stNear Token
        // } else if token_address == self.stnear_token_contract_address {
        // TODO: something is WEIRD in here, please review.  <======================
        } else if token_address == self.aaxxii_token_contract_address {
            for item in distribute_info {
                // in case of stNEAR, item.1 is stNEAR amount * 1e4 (4 decimal places)
                // so we multiply by 1e20 to get yocto-stNEAR
                let amount = item.1 as u128 * E20;
                self.add_claimable_stnear(&AccountId::new_unchecked(item.0.clone()), amount);
                total_distributed += amount;
            }
            // self.accum_distributed_stnear_for_claims += total_distributed;
        } else {
            panic!("Unknown token address: {}", token_address);
        }

        unimplemented!();

        assert!(
            total_distributed == total_amount,
            "total to distribute {} != total_amount sent {}",
            total_distributed,
            total_amount
        );
}
}
