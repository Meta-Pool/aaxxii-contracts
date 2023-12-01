use crate::*;
use crate::interface::*;
use near_sdk::{near_bindgen, PromiseResult, json_types::U128};

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

    // /// This transfer is only to claim available stNEAR
    // pub(crate) fn transfer_stnear_to_voter(
    //     &mut self,
    //     voter_id: VoterId,
    //     amount: Balance
    // ) {
    //     /// TODO: Correct address??? change stnear
    //     ext_ft::ext(self.aaxxii_token_contract_address.clone())
    //         .with_static_gas(GAS_FOR_FT_TRANSFER)
    //         .with_attached_deposit(1)
    //         .ft_transfer(
    //             voter_id.clone(),
    //             U128::from(amount),
    //             None
    //     ).then(
    //         Self::ext(env::current_account_id())
    //             .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
    //             .after_transfer_stnear_callback(
    //                 voter_id,
    //                 U128::from(amount)
    //             )
    //     );
    // }

    // #[private]
    // pub fn after_transfer_stnear_callback(
    //     &mut self,
    //     voter_id: VoterId,
    //     amount: U128
    // ) {
    //     let amount = amount.0;
    //     match env::promise_result(0) {
    //         PromiseResult::NotReady => unreachable!(),
    //         PromiseResult::Successful(_) => {
    //             log!("WITHDRAW: {} stNEAR transfer to {}", amount, voter_id.to_string());
    //         },
    //         PromiseResult::Failed => {
    //             log!(
    //                 "FAILED: {} stNEAR not transferred. Recovering {} state.",
    //                 amount, &voter_id.to_string()
    //             );
    //             self.add_claimable_stnear(&voter_id, amount);
    //         },
    //     };
    // }
}