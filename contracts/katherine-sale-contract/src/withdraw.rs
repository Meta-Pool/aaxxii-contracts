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
        amount: U128,
        sale: &mut Sale
    ) -> Promise {
        sale.claimable_sold_token_for_buyers.remove(&buyer_id);
        sale.sold_tokens_for_buyers -= amount.0;
        self.sales.replace(sale.id as u64, &sale);
        ext_ft::ext(sale.sold_token_contract_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(buyer_id.clone(), amount, None).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .buyer_withdraw_sold_tokens_resolve(
                        &buyer_id,
                        amount,
                        sale.id
                    )
            )
    }

    #[private]
    pub fn buyer_withdraw_sold_tokens_resolve(
        &mut self,
        buyer_id: &AccountId,
        amount: U128,
        sale_id: u32
    ) {
        let amount = amount.0;

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} tokens transferred to {}",
                    amount, buyer_id
                );
            },
            PromiseResult::Failed => {
                let mut sale = self.internal_get_sale(sale_id);
                sale.claimable_sold_token_for_buyers.insert(&buyer_id, &amount);
                sale.sold_tokens_for_buyers += amount;
                self.sales.replace(sale.id as u64, &sale);
                log!(
                    "FAILED: {} tokens not transferred. Recovering sale {} state.",
                    amount, sale_id
                );
            }
        };
    }

    pub(crate) fn internal_buyer_withdraw_payment_token(
        &mut self,
        buyer_id: AccountId,
        amount: U128,
        sale: &Sale
    ) -> Promise {
        unimplemented!();
    }
}
