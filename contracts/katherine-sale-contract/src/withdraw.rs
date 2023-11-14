use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{Promise, env, log, near_bindgen, PromiseOrValue, require};
use crate::interface::*;

#[near_bindgen]
impl KatherineSaleContract {
    pub(crate) fn internal_buyer_withdraw_sold_tokens(
        &mut self,
        buyer_id: AccountId,
        claimable: u128,
        deposit: u128,
        sale: &mut Sale
    ) -> Promise {
        sale.sold_tokens_for_buyers -= claimable;
        sale.required_sold_token -= claimable;
        self.sales.replace(sale.id as u64, &sale);
        let claimable = U128::from(claimable);
        let deposit = U128::from(deposit);
        let token_id = sale.sold_token_contract_address.clone();

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

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens of sold-token {} transferred to {}",
                    claimable, token_id, buyer_id
                );
            },
            PromiseResult::Failed => {
                let mut sale = self.internal_get_sale(sale_id);
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

    /// Unsuccessful sale
    pub(crate) fn internal_buyer_withdraw_payment_token(
        &mut self,
        buyer_id: AccountId,
        claimable: u128,
        deposit: u128,
        sale: &mut Sale
    ) -> Promise {
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
            let token_id = sale.payment_config.payment_token_contract_address.clone().unwrap();
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
                            sale.id
                        )
                )
        }
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

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens of payment-token {} transferred back to {}",
                    claimable, token_id, buyer_id
                );
            },
            PromiseResult::Failed => {
                let deposit = deposit.0;
                let mut sale = self.internal_get_sale(sale_id);

                sale.deposits.insert(buyer_id, &deposit);
                sale.claimable_sold_token_for_buyers.insert(&buyer_id, &claimable);
                sale.total_payment_token += deposit;
                sale.required_sold_token += claimable;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    claimable, sale_id
                );
            }
        };
    }

    pub(crate) fn internal_seller_withdraw_near(&mut self) -> Promise {
        unimplemented!();
    }

    pub(crate) fn internal_seller_withdraw_payment_token(&mut self) -> Promise {
        unimplemented!();
    }
}
