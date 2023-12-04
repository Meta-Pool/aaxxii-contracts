use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, PromiseOrValue};
use near_sdk::serde_json;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[near_bindgen]
impl FungibleTokenReceiver for StakingPositionContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount = amount.0;

        // deposit for-claims, msg == "for-claims" means tokens to be later distributed to voters
        if msg.len() >= 11 && &msg[..11] == "for-claims:" {

            // Leaving 2 digits for the multiplier
            let multiplier = msg[11..13].parse::<u8>().expect("Err parsing multiplier.");
            match serde_json::from_str(&msg[13..]) {
                Ok(distribute_info) => self.distribute_ft_claims(
                    amount,
                    multiplier,
                    distribute_info
                ),
                Err(_) => panic!("Err parsing msg for-claims"),
            };
        }

        // else, user deposit underlying asset to lock.
        else {
            let locking_period = msg.parse::<Days>()
                .expect("Err parsing locking_period from msg. Must be u16.");

            assert_eq!(
                env::predecessor_account_id(),
                self.underlying_token_contract_address,
                "This contract only works with {}",
                self.underlying_token_contract_address
            );

            self.assert_min_deposit_amount(amount);
            log!("DEPOSIT: {} deposited from {}", amount, &sender_id,);
            let mut staker = self.internal_get_staker(sender_id);
            self.deposit_locking_position(amount, locking_period, &mut staker);
        }

        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

impl StakingPositionContract {
    // called from ft_on_transfer
    fn distribute_ft_claims(
        &mut self,
        total_amount: u128,
        multiplier: u8,
        distribute_info: Vec<(String, u64)>,
    ) {
        let mut total_distributed = 0;
        let token_address = env::predecessor_account_id();

        require!(self.is_ft_available(&token_address), "Unknown token address.");
        for (owner, pre_amount) in distribute_info {
            let amount = pre_amount as u128 * 10u128.pow(multiplier.into());
            self.add_claimable_ft(
                &AccountId::new_unchecked(owner),
                &token_address,
                amount
            );
            total_distributed += amount;
        }

        assert!(
            total_distributed == total_amount,
            "total to distribute {} != total_amount sent {}",
            total_distributed,
            total_amount
        );
    }

    pub(crate) fn distribute_near_claims(
        &mut self,
        total_amount: u128,
        distribute_info: Vec<(AccountId, U128)>
    ) {
        let mut total_distributed = 0;

        for (owner, amount) in distribute_info {
            let amount = amount.0;
            self.add_claimable_near(&owner, amount);
            total_distributed += amount;
        }

        assert!(
            total_distributed == total_amount,
            "total to distribute {} != total_amount sent {}",
            total_distributed,
            total_amount
        );
    }
}
