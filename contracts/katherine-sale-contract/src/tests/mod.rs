use super::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json;
use near_sdk::testing_env;
use near_sdk::test_utils::{accounts, VMContextBuilder};

mod utils;
use utils::*;

// const E20: u128 = 100_000_000_000_000_000_000;

fn new_katherine_contract() -> KatherineSaleContract {
    KatherineSaleContract::new(
        owner_account(),
        U128::from(MIN_DEPOSIT_AMOUNT_IN_NEAR),
        U128::from(MIN_DEPOSIT_AMOUNT_IN_PAYMENT_TOKEN),
        usdt_token_contract(),
        U128::from(USDT_UNIT),
        SALE_FEE,
    )
}

fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id);
    builder
}

fn create_sale(
    contract: &mut KatherineSaleContract,
    slug: &str,
    is_in_near: bool
) {
    let unit = if is_in_near {NEAR} else {USDT_UNIT};
    contract.create_sale(
        // slug: String,
        String::from(slug),
        // is_in_near: bool,
        is_in_near,
        // sold_token_contract_address: AccountId,
        sold_token_contract(),
        // one_payment_token_purchase_rate: u128,
        U128::from(2 * unit),
        // max_available_sold_token: Balance,
        U128::from(10 * unit),
        // open_date_timestamp: EpochMillis,
        U64::from(nanos_to_millis(to_ts(0))),
        // close_date_timestamp: EpochMillis,
        U64::from(nanos_to_millis(to_ts(10))),
        // release_date_timestamp: EpochMillis,
        U64::from(nanos_to_millis(to_ts(15))),
    );
}

// Check the docs: https://docs.near.org/sdk/rust/testing/integration-tests#

#[test]
fn test_near_sale_creation() {
    let mut context = get_context(owner_account());
    testing_env!(context.build());
    let mut contract = new_katherine_contract();

    testing_env!(context
        .predecessor_account_id(owner_account())
        .attached_deposit(STORAGE_PER_SALE)
        .build()
    );
    create_sale(&mut contract, "test-sale-1", true);

    testing_env!(context.is_view(true).build());
    assert_eq!(1, contract.get_number_of_sales(), "Sale was not created!");
    assert_eq!(1, contract.active_sales.len(), "Sale was not created!");
}

#[test]
fn test_near_deposit() {
    let mut context = get_context(owner_account());
    testing_env!(context.build());
    let mut contract = new_katherine_contract();

    testing_env!(context
        .predecessor_account_id(owner_account())
        .attached_deposit(STORAGE_PER_SALE)
        .build()
    );
    create_sale(&mut contract, "test-sale-1", true);

    testing_env!(context
        .predecessor_account_id(accounts(1))
        .attached_deposit(3 * NEAR)
        .block_timestamp(to_ts(0))
        .build()
    );
    contract.purchase_token_with_near(0);

    testing_env!(context.is_view(true).build());
    assert_eq!(6 * NEAR, contract.get_claimable_sold_token_for_buyers(accounts(1), 0).0);
    assert_eq!(6 * NEAR, contract.sales.get(0).unwrap().required_sold_token);
    assert_eq!(3 * NEAR, contract.sales.get(0).unwrap().total_payment_token);

    testing_env!(context
        .predecessor_account_id(accounts(2))
        .is_view(false)
        .attached_deposit(1 * NEAR)
        .block_timestamp(to_ts(1))
        .build()
    );
    contract.purchase_token_with_near(0);

    testing_env!(context.is_view(true).build());
    assert_eq!(2 * NEAR, contract.get_claimable_sold_token_for_buyers(accounts(2), 0).0);
    assert_eq!(8 * NEAR, contract.sales.get(0).unwrap().required_sold_token);
    assert_eq!(4 * NEAR, contract.sales.get(0).unwrap().total_payment_token);
}

#[test]
#[should_panic(expected = "Not enough token for sale.")]
fn test_fail_near_deposit() {
    let mut context = get_context(owner_account());
    testing_env!(context.build());
    let mut contract = new_katherine_contract();

    testing_env!(context
        .predecessor_account_id(owner_account())
        .attached_deposit(STORAGE_PER_SALE)
        .build()
    );
    create_sale(&mut contract, "test-sale-1", true);

    testing_env!(context
        .predecessor_account_id(accounts(1))
        // The [6 * 2 = 12] and the max tokens are 10!
        .attached_deposit(6 * NEAR)
        .block_timestamp(to_ts(0))
        .build()
    );
    contract.purchase_token_with_near(0);
}
