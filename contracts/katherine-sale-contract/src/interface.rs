use crate::*;
use near_sdk::ext_contract;
use near_sdk::json_types::U128;

#[ext_contract(ext_ft)]
pub trait FungibleTokenCore {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );

    // fn fail(&self);
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn buyer_withdraw_sold_tokens_resolve(
        &mut self,
        buyer_id: AccountId,
        amount: U128,
        sale_id: u32
    );
}
