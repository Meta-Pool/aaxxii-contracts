use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{Promise, env, log, near_bindgen};

/// Notice:
/// There are three sales variables, that work as reservoirs, and will be affected
/// by withdraws:
///     - sale.sold_tokens_for_buyers
///     - sale.required_sold_token
///     - sale.total_payment_token

#[near_bindgen]
impl KatherineSaleContract {

    // **************************
    // * Successful sale: Buyer *
    // **************************

    pub(crate) fn internal_buyer_withdraw_sold_tokens(
        &mut self,
        buyer_id: AccountId,
        claimable: u128,
        deposit: u128,
        sale: &mut Sale
    ) -> Promise {
        // Removing claimable tokens. `total_payment_token` stays the same.
        sale.sold_tokens_for_buyers -= claimable;
        sale.required_sold_token -= claimable;
        self.sales.replace(sale.id as u64, &sale);

        let claimable = U128::from(claimable);
        let deposit = U128::from(deposit);
        let token_id = sale.get_sold_token();
        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(buyer_id.clone(), claimable, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .buyer_withdraw_sold_tokens_resolve(
                        &buyer_id,
                        &token_id,
                        claimable,
                        deposit,
                        sale.id
                    )
            )
    }

    #[private]
    pub fn buyer_withdraw_sold_tokens_resolve(
        &mut self,
        buyer_id: &AccountId,
        token_id: &AccountId,
        claimable: U128,
        deposit: U128,
        sale_id: u32
    ) {
        let claimable = claimable.0;
        let mut buyer = self.internal_get_buyer(&buyer_id);

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                if buyer.is_empty() {
                    self.buyers.remove(&buyer_id);
                    log!("GODSPEED: {} is no longer part of Katherine!", &buyer_id);
                }
                log!(
                    "WITHDRAW: {} tokens of sold-token {} transferred to {}",
                    claimable, token_id, buyer_id
                );
            },
            PromiseResult::Failed => {
                buyer.supporting_sales.insert(&sale_id);
                self.buyers.insert(&buyer_id, &buyer);

                let mut sale = self.internal_get_sale(sale_id);
                // Important: Recover the claimable tokens and deposit from user.
                sale.deposits.insert(buyer_id, &deposit.0);
                sale.claimable_sold_token_for_buyers.insert(&buyer_id, &claimable);
                sale.sold_tokens_for_buyers += claimable;
                sale.required_sold_token += claimable;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    claimable, sale_id
                );
            }
        };
    }

    // ****************************
    // * Unsuccessful sale: Buyer *
    // ****************************

    /// Payment tokens will only be returned to the buyer if the seller never
    /// deposited the full `required_sold_token` before the release date.
    pub(crate) fn internal_buyer_withdraw_payment_token(
        &mut self,
        buyer_id: AccountId,
        claimable: u128,
        deposit: u128,
        sale: &mut Sale
    ) -> Promise {
        // Removing claimable tokens and returning the deposit to the buyer.
        // `sold_tokens_for_buyers` stays the same for the seller to reclaim.
        sale.required_sold_token -= claimable;
        sale.total_payment_token -= deposit;
        self.sales.replace(sale.id as u64, &sale);

        if sale.is_near_accepted() {
            log!(
                "WITHDRAW: {} NEAR transferred back to {}",
                claimable, buyer_id
            );
            Promise::new(buyer_id).transfer(deposit)
        } else {
            // If sale is not in near then expect a token address.
            let token_id = sale.get_payment_token();
            self.buyer_withdraw_ft_payment_token(
                buyer_id,
                claimable,
                deposit,
                token_id,
                sale.id
            )
        }
    }

    fn buyer_withdraw_ft_payment_token(
        &mut self,
        buyer_id: AccountId,
        claimable: u128,
        deposit: u128,
        token_id: AccountId,
        sale_id: u32
    ) -> Promise {
        let claimable = U128::from(claimable);
        let deposit = U128::from(deposit);

        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(buyer_id.clone(), deposit, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .buyer_withdraw_payment_tokens_resolve(
                        &buyer_id,
                        &token_id,
                        claimable,
                        deposit,
                        sale_id
                    )
            )
    }

    #[private]
    pub fn buyer_withdraw_payment_tokens_resolve(
        &mut self,
        buyer_id: &AccountId,
        token_id: &AccountId,
        claimable: U128,
        deposit: U128,
        sale_id: u32
    ) {
        let claimable = claimable.0;
        let mut buyer = self.internal_get_buyer(&buyer_id);

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                if buyer.is_empty() {
                    self.buyers.remove(&buyer_id);
                    log!("GODSPEED: {} is no longer part of Katherine!", &buyer_id);
                }
                log!(
                    "WITHDRAW: {} tokens of payment-token {} transferred back to {}",
                    claimable, token_id, buyer_id
                );
            },
            PromiseResult::Failed => {
                buyer.supporting_sales.insert(&sale_id);
                self.buyers.insert(&buyer_id, &buyer);

                let deposit = deposit.0;
                let mut sale = self.internal_get_sale(sale_id);
                // Important: Recover the claimable tokens and deposit from user.
                sale.deposits.insert(buyer_id, &deposit);
                sale.claimable_sold_token_for_buyers.insert(&buyer_id, &claimable);
                sale.required_sold_token += claimable;
                sale.total_payment_token += deposit;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    claimable, sale_id
                );
            }
        };
    }

    // ***********************************
    // * Collect payments & fees: Seller *
    // ***********************************

    pub(crate) fn internal_collect_payments(&mut self, sale: &mut Sale) -> Promise {
        let fee = proportional(
            sale.total_payment_token,
            sale.payment_config.sale_fee as u128,
            BASIS_POINT as u128
        );
        let to_send = sale.total_payment_token - fee;
        sale.total_payment_token = 0;
        sale.total_fees = fee;
        self.sales.replace(sale.id as u64, &sale);

        if sale.is_near_accepted() {
            Promise::new(self.treasury_id.clone()).transfer(to_send)
        } else {
            // If sale is not in near then expect a token address.
            let token_id = sale.get_payment_token();
            self.internal_seller_withdraw_payment_token(
                token_id,
                to_send,
                sale.id
            )
        }
    }

    fn internal_seller_withdraw_payment_token(
        &mut self,
        token_id: AccountId,
        amount: u128,
        sale_id: u32
    ) -> Promise {
        let amount = U128::from(amount);

        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(self.treasury_id.clone(), amount, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .seller_withdraw_payment_tokens_resolve(
                        &token_id,
                        amount,
                        sale_id
                    )
            )
    }

    #[private]
    pub fn seller_withdraw_payment_tokens_resolve(
        &mut self,
        token_id: &AccountId,
        amount: U128,
        sale_id: u32
    ) {
        let amount = amount.0;

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens of payment-token {} transferred to {}",
                    amount, token_id, &self.treasury_id
                );
            },
            PromiseResult::Failed => {
                let mut sale = self.internal_get_sale(sale_id);

                // Important: Recover the claimable tokens and deposit from user.
                sale.total_payment_token = amount + sale.total_fees;
                sale.total_fees = 0;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    amount, sale_id
                );
            }
        };
    }

    pub(crate) fn internal_collect_fees(&mut self, sale: &mut Sale) -> Promise {
        let to_send = sale.total_fees;
        sale.total_fees = 0;
        self.sales.replace(sale.id as u64, &sale);

        if sale.is_near_accepted() {
            Promise::new(self.treasury_id.clone()).transfer(to_send)
        } else {
            // If sale is not in near then expect a token address.
            let token_id = sale.get_payment_token();
            self.internal_seller_withdraw_fee(
                token_id,
                to_send,
                sale.id
            )
        }
    }

    fn internal_seller_withdraw_fee(
        &mut self,
        token_id: AccountId,
        amount: u128,
        sale_id: u32
    ) -> Promise {
        let amount = U128::from(amount);

        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(self.treasury_id.clone(), amount, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .seller_withdraw_fee_resolve(
                        &token_id,
                        amount,
                        sale_id
                    )
            )
    }

    #[private]
    pub fn seller_withdraw_fee_resolve(
        &mut self,
        token_id: &AccountId,
        amount: U128,
        sale_id: u32
    ) {
        let amount = amount.0;

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens of payment-token {} transferred to {}",
                    amount, token_id, &self.treasury_id
                );
            },
            PromiseResult::Failed => {
                let mut sale = self.internal_get_sale(sale_id);
                sale.total_fees = amount;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    amount, sale_id
                );
            }
        };
    }

    // ******************************
    // * Excess sold tokens: Seller *
    // ******************************

    pub(crate) fn seller_withdraw_excess_sold_tokens(
        &mut self,
        excess: u128,
        sale: &mut Sale
    ) -> Promise {
        // `total_payment_token` and `required_sold_token` stay the same.
        sale.sold_tokens_for_buyers -= excess;
        self.sales.replace(sale.id as u64, &sale);

        let excess = U128::from(excess);
        let token_id = sale.get_sold_token();
        ext_ft::ext(token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(self.owner_id.clone(), excess, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .seller_withdraw_excess_sold_tokens_resolve(
                        &token_id,
                        excess,
                        sale.id
                    )
            )
    }

    #[private]
    pub fn seller_withdraw_excess_sold_tokens_resolve(
        &mut self,
        token_id: &AccountId,
        excess: U128,
        sale_id: u32
    ) {
        let excess = excess.0;

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens of sold-token {} transferred to {}",
                    excess, token_id, &self.owner_id
                );
            },
            PromiseResult::Failed => {
                let mut sale = self.internal_get_sale(sale_id);
                sale.sold_tokens_for_buyers += excess;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    excess, sale_id
                );
            }
        };
    }

}
