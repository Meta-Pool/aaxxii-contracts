use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, PromiseOrValue, require};

#[near_bindgen]
impl FungibleTokenReceiver for KatherineSaleContract {

    // *********************
    // * Payments using FT *
    // *********************

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
        } else if sale.payment_config.payment_token_contract_address.is_some()
                && sale.payment_config.payment_token_contract_address.as_ref().unwrap() == &env::predecessor_account_id() {
            self.process_payment_tokens_deposit(&sender_id, amount, &mut sale);
            log!(
                "DEPOSIT: {} payment tokens deposited from {} to sale {}",
                amount,
                &sender_id,
                &msg
            );

        } else {
            panic!("Unknown token {} for sale {}", env::predecessor_account_id(), &msg);
        }

        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

impl KatherineSaleContract {
    /// Process a payment deposit.
    /// This function should be able to process NEAR and payment token.
    pub(crate) fn process_payment_tokens_deposit(
        &mut self,
        buyer_id: &AccountId,
        amount: Balance,
        sale: &mut Sale,
    ) {
        sale.assert_min_deposit_amount(amount);
        sale.assert_within_funding_period();

        // For the payment token.
        sale.total_payment_token += amount;
        let sold_tokens = sale.from_payment_to_sold_token(amount);

        // For the sold token.
        let new_amount = sale.get_claimable_sold_token_for_buyers(buyer_id) + sold_tokens;
        sale.claimable_sold_token_for_buyers.insert(buyer_id, &new_amount);
        sale.required_sold_token += new_amount;
        require!(
            sale.required_sold_token <= sale.max_available_sold_token,
            "Not enough token for sale."
        );

        // Update Sale and Buyer
        self.sales.replace(sale.id as u64, &sale);
        let mut buyer = self.internal_get_buyer(&buyer_id);
        buyer.active_sales.insert(&sale.id);
        self.buyers.insert(&buyer_id, &buyer);
    }

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
        self.sales.replace(sale.id as u64, &sale);
    }
}
