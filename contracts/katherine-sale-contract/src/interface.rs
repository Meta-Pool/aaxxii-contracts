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

    fn fail(&self);
}

// #[ext_contract(ext_metapool)]
// pub trait ExtSelfMetapool {
//     fn get_st_near_price(&self) -> U128;
// }

#[ext_contract(ext_self)]
pub trait ExtSelf {
    // fn pay_near_bond(&mut self, bond_id: BondId, near_amount_to_send: U128);

    fn buyer_withdraw_sold_tokens_resolve(
        &mut self,
        buyer_id: AccountId,
        amount: U128,
        sale_id: u32
    );

    // fn pay_near_bond_resolve(
    //     &mut self,
    //     bond_id: BondId,
    //     owner_id: AccountId,
    //     stnear_amount: U128
    // );

    // fn pay_ptoken_bond_resolve(
    //     &mut self,
    //     bond_id: BondId,
    //     owner_id: AccountId,
    //     amount: U128
    // );

    // fn beneficiary_withdraw_resolve_transfer(
    //     &mut self,
    //     vault_id: VaultId,
    //     amount: U128,
    //     receiver_id: AccountId
    // );

    // fn beneficiary_withdraw_callback(
    //     &mut self,
    //     vault_id: VaultId,
    //     receiver_id: AccountId
    // );

    // fn set_stnear_price_at_unfreeze(&mut self, vault_id: VaultId);
}
