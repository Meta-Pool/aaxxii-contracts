use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, PromiseOrValue, require};

#[near_bindgen]
impl FungibleTokenReceiver for KatherineSale {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let sale_id = match msg.parse::<u32>() {
            Ok(_id) => _id,
            Err(_) => panic!("Invalid sale id."),
        };
        let mut sale = self.internal_get_sale(sale_id);
        let amount = amount.0;

        // Deposit of the sold tokens.
        if env::predecessor_account_id() == sale.sold_token_contract_address {
            self.process_sold_tokens_deposit(amount, &mut sale);
            log!(
                "DEPOSIT: {} sold tokens deposited from {} to sale {}",
                amount,
                &sender_id,
                &msg
            );

        // Deposit of a payment token.
        } else if sale.payment_token_contract_address.is_some()
                && sale.payment_token_contract_address.unwrap() == env::predecessor_account_id() {
            self.assert_min_deposit_amount(amount);
        // self.process_sold_tokens_deposit(sender_id.as_ref(), &amount, &mut kickstarter);
            log!(
                "DEPOSIT: {} payment tokens deposited from {} to sale {}",
                amount,
                &sender_id,
                &msg
            );

        } else {
            panic!("Unknown token {} for sale {}", env::predecessor_account_id(), &msg);
        }
            
            
            
        //     if env::predecessor_account_id() == sale.sold_token_contract_address {
        //     // self.assert_min_deposit_amount(amount);
        //     log!(
        //         "DEPOSIT: {} sold tokens deposited from {} to sale {}",
        //         amount,
        //         &sender_id,
        //         &msg
        //     );
        //     // self.process_kickstarter_deposit(amount, &mut kickstarter);
        // } else {
        //     panic!("Unknown token {} for sale {}", env::predecessor_account_id(), &msg);
        // }
        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

impl KatherineSale {
    fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }

    // /// Process a stNEAR deposit to Katherine Contract.
    // fn process_supporter_deposit(
    //     &mut self,
    //     supporter_id: &AccountId,
    //     amount: &Balance,
    //     kickstarter: &mut Kickstarter,
    // ) {
    //     // Update Kickstarter
    //     kickstarter.assert_within_funding_period();
    //     kickstarter.assert_enough_reward_tokens();

    //     let new_total_deposited = kickstarter.total_deposited + amount;
    //     assert!(
    //         new_total_deposited <= kickstarter.deposits_hard_cap,
    //         "The deposits hard cap cannot be exceeded!"
    //     );
    //     kickstarter.total_deposited = new_total_deposited;
    //     kickstarter.update_supporter_deposits(&supporter_id, amount);
    //     self.kickstarters
    //         .replace(kickstarter.id as u64, &kickstarter);

    //     // Update Supporter.
    //     let mut supporter = self.internal_get_supporter(&supporter_id);
    //     supporter.supported_projects.insert(&kickstarter.id);
    //     self.supporters.insert(&supporter_id, &supporter);
    // }

    /// Process the tokens that are going to be sold.
    fn process_sold_tokens_deposit(
        &mut self,
        amount: Balance,
        sale: &mut Sale,
    ) {
        require!(
            get_current_epoch_millis() < sale.release_date_timestamp,
            "Too late. Sale is over."
        );
        // let amount = kickstarter.less_to_24_decimals(amount);
        // let max_tokens_to_release = self.calculate_max_tokens_to_release(&kickstarter);
        // let min_tokens_to_allow_support = max_tokens_to_release
        //     + self.calculate_katherine_fee(max_tokens_to_release);
        sale.sold_tokens_for_buyers += amount;
        // kickstarter.enough_reward_tokens = {
        //     kickstarter.available_reward_tokens >= min_tokens_to_allow_support
        // };
        self.sales
            .replace(sale.id as u64, &sale);
    }
}
