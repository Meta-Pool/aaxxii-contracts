use crate::*;
use crate::interface::*;
use near_sdk::{near_bindgen, PromiseResult, json_types::U128, Promise};

#[near_bindgen]
impl StakingPositionContract {
    pub(crate) fn transfer_balance_to_voter(
        &mut self,
        staker_id: AccountId,
        amount: Balance
    ) {
        ext_ft::ext(self.underlying_token_contract_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(
                staker_id.clone(),
                U128::from(amount),
                None
        ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                .after_transfer_balance_callback(
                    staker_id,
                    U128::from(amount)
                )
        );
    }

    #[private]
    pub fn after_transfer_balance_callback(
        &mut self,
        staker_id: AccountId,
        amount: U128
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("WITHDRAW: {} transfer to {}", amount, &staker_id);
            },
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} not transferred. Recovering {} state.",
                    amount, &staker_id
                );
                let mut staker = self.internal_get_staker(staker_id);
                staker.balance += amount;
                self.stakers.insert(&staker.id, &staker);
            },
        };
    }

    /// This transfer is only to claim available FT
    pub(crate) fn transfer_claimable_ft(
        &mut self,
        account: AccountId,
        amount: Balance,
        token_address: AccountId
    ) -> Promise {
        ext_ft::ext(token_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(
                account.clone(),
                U128::from(amount),
                None
        ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                .after_transfer_ft_callback(
                    account,
                    U128::from(amount),
                    token_address
                )
        )
    }

    #[private]
    pub fn after_transfer_ft_callback(
        &mut self,
        account: AccountId,
        amount: U128,
        token_address: AccountId
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!("WITHDRAW: {} FT transferred to {}", amount, &account);
            },
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} FT not transferred. Recovering {} state.",
                    amount, &account
                );
                self.add_claimable_ft(&account, &token_address, amount);
            },
        };
    }
}