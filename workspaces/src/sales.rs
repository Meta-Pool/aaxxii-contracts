// use std::fs;
// use katherine_sale_contract::types::{EpochMillis, VaultId};
// use katherine_sale_contract::utils::proportional;
// use near_units::{parse_gas, parse_near};
// use json;
use std::str;
// use meta_tools::bond::BondLoaderJSON;
use near_sdk::json_types::{U128, U64};
// use near_sdk::AccountId as NearAccountId;

// use katherine_sale_contract::constants::STORAGE_PER_SALE;
// use crate::constants::STORAGE_PER_SALE;
pub const STORAGE_PER_SALE: u128 = NEAR / 100;

use near_workspaces::result::ExecutionFinalResult;
// use near_workspaces::types::NearToken;
use near_workspaces::types::NearToken;

// // use workspaces::network::Sandbox;
use near_workspaces::{Account, AccountId, Contract, Worker, DevNetwork};
use near_gas::NearGas;

// use workspaces::result::ExecutionFinalResult;
// use workspaces::error::Error as WorkspaceError;

use test_utils::now::Now;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

const KATHERINE_SALE_FILEPATH: &str = "./res/katherine_sale_contract.wasm";
const PTOKEN_FILEPATH: &str = "./res/test_p_token.wasm";
const TEST_UTILS_FILEPATH: &str = "./res/test_utils.wasm";

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;

    // Creating Accounts.
    let owner = worker.dev_create_account().await?;
    let treasury = worker.dev_create_account().await?;
    let buyer = worker.dev_create_account().await?;

    // **************************************
    // * Stage 1: Deploy relevant contracts *
    // **************************************

    let test_utils_contract = create_test_utils(&worker).await?;
    let sold_token_contract = create_sold_token(&owner, &worker).await?;
    let usdc_token_contract = create_usdc_token(&owner, &worker).await?;
    let katherine_contract = create_katherine(
        &owner,
        &treasury,
        usdc_token_contract.id(),
        &worker).await?;

    println!("Sold-Token Contract: {}", sold_token_contract.id());
    println!("USDC-Token Contract: {}", usdc_token_contract.id());
    println!("Katherine Contract: {}", katherine_contract.id());

    let now = Now::new_from_epoch_millis(test_utils_contract.call("get_now").view().await?.json()?);
    println!("Current Timestamp milliseconds: {}", now);

    let res = registering_accounts(
        &sold_token_contract,
        &usdc_token_contract,
        &katherine_contract,
        &treasury,
        &buyer,
    ).await?;
    println!("Registering Accounts.: {:?}\n", res);

    // Transferring USDC to buyer.
    let _ = owner
        .call(usdc_token_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": buyer.id(),
            "amount": "10000000", // 10 usdc
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    // ***************************
    // * Stage 2: Creating Sales *
    // ***************************

    let outcome = create_sale(
        0,
        true,
        &now,
        &owner,
        &katherine_contract,
        &sold_token_contract
    ).await?;
    println!("create_sale #0 outcome: {:#?}", outcome);
    assert!(outcome.is_success());
    let outcome = &outcome.raw_bytes().unwrap().clone();
    let id = str::from_utf8(outcome).unwrap();
    assert_eq!(id, 0.to_string());

    let outcome: serde_json::Value = katherine_contract
        .call("get_number_of_sales")
        .view()
        .await?
        .json()?;
    let n = outcome.as_u64().unwrap();
    assert_eq!(n, 1u64);

    let outcome: serde_json::Value = katherine_contract
        .call("get_active_sales")
        .args_json(serde_json::json!({
            "from_index": 0,
            "limit": 10
        }))
        .view()
        .await?
        .json()?;
    let sales = outcome.as_array().unwrap();
    assert_eq!(1, sales.len());

    let outcome = create_sale(
        1,
        false,
        &now,
        &owner,
        &katherine_contract,
        &sold_token_contract
    ).await?;
    println!("create_sale #1 outcome: {:#?}", outcome);
    assert!(outcome.is_success(), "Cannot split bond.");

    let outcome = &outcome.raw_bytes().unwrap().clone();
    let id = str::from_utf8(outcome).unwrap();
    assert_eq!(id, 1.to_string());

    let outcome: serde_json::Value = katherine_contract
        .call("get_number_of_sales")
        .view()
        .await?
        .json()?;
    let n = outcome.as_u64().unwrap();
    assert_eq!(n, 2u64);

    let outcome: serde_json::Value = katherine_contract
        .call("get_active_sales")
        .args_json(serde_json::json!({
            "from_index": 0,
            "limit": 10
        }))
        .view()
        .await?
        .json()?;
    let sales = outcome.as_array().unwrap();
    assert_eq!(2, sales.len());

    // **************************
    // * Stage 3: Buying tokens *
    // **************************

    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;
    worker.fast_forward(500).await?;
    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;

    // purchase_token_with_near(&mut self, sale_id: u32)
    let outcome = buyer
        .call(katherine_contract.id(), "purchase_token_with_near")
        .args_json(json!({
            "sale_id": 0,
        }))
        .deposit(NearToken::from_near(10))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("purchase_token_with_near #0: {:#?}", outcome);
    assert!(outcome.is_success());

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10 * 2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u64>().unwrap()
    );

    // ******************
    // Checking bad math
    // ******************

    let outcome = owner
        .call(katherine_contract.id(), "purchase_token_with_near")
        .args_json(json!({
            "sale_id": 0,
        }))
        .deposit(NearToken::from_near(5))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("purchase_token_with_near #0: {:#?}", outcome);
    assert!(outcome.is_success());

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10*2 + 5*2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(15).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u64>().unwrap()
    );

    // This first call must fail because it's pointing to the incorrect sale_id.
    // near call <ft-contract> ft_transfer_call '{"receiver_id": "<receiver-contract>", "amount": "<amount>", "msg": "<a-string-message>"}' --accountId <user_account_id> --depositYocto 1
    let outcome = buyer
        .call(usdc_token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": katherine_contract.id(),
            "amount": "10000000", // 10 USDC
            "msg": "0"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("ft_transfer_call #0: {:#?}", outcome);
    // Due to Workspaces nuances, this is the way to see if a receipt in the tx failed.
    assert!(outcome.is_success() && outcome.receipt_failures().len() == 1, "Incorrect payment token");

    let outcome = buyer
        .call(usdc_token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": katherine_contract.id(),
            "amount": "10000000", // 10 USDC
            "msg": "1"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("ft_transfer_call #1: {:#?}", outcome);
    assert!(outcome.is_success());

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 1
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10 * 2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        10_000_000_u128,
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u64>().unwrap()
    );

    // **********************************************************
    // * Stage 4: Seller delivers for sale 0 but not for sale 1 *
    // **********************************************************

    let outcome = owner
        .call(sold_token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": katherine_contract.id(),
            "amount": U128::from(NearToken::from_near(3*10).as_yoctonear()),
            "msg": "0"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("Transferring sold tokens for sale 0: {:#?}", outcome);

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10*2 + 5*2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(15).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(10 * 3).as_yoctonear(),
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u128>().unwrap()
    );

    let _ = owner
        .call(sold_token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": katherine_contract.id(),
            "amount": U128::from(NearToken::from_near(10).as_yoctonear()),
            "msg": "1"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 1
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10 * 2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        10_000_000_u128,
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(),
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u128>().unwrap()
    );

    // ******************************
    // * Stage 5: Withdraw payments *
    // ******************************

    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;
    worker.fast_forward(500).await?;
    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;

    let treasury_original_balance = treasury.view_account().await?.balance;
    let outcome = owner
        .call(katherine_contract.id(), "collect_payments")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        // .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    println!("Collect payments 🏦: {:#?}", outcome);
    let treasury_new_balance = treasury.view_account().await?.balance;
    // Get se sale earnings without the 2.5% of fee.
    assert_eq!(
        treasury_new_balance.as_yoctonear(),
        treasury_original_balance.as_yoctonear()
        + NearToken::from_millinear(14625).as_yoctonear() // Manually calculating the fee.
    );

    println!("old: {} \nnew: {}", treasury_original_balance, treasury_new_balance);

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10*2 + 5*2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_millinear(375).as_yoctonear(), // Manually calculating the fee.
        sale["total_fees"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(10 * 3).as_yoctonear(),
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u128>().unwrap()
    );

    let outcome = owner
        .call(katherine_contract.id(), "collect_fees")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        // .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    println!("Collect payments 🏦: {:#?}", outcome);
    let treasury_new_balance = treasury.view_account().await?.balance;
    // Get se sale earnings without the 2.5% of fee.
    assert_eq!(
        treasury_new_balance.as_yoctonear(),
        treasury_original_balance.as_yoctonear()
        + NearToken::from_near(15).as_yoctonear()
    );

    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();
    assert_eq!(
        NearToken::from_near(10*2 + 5*2).as_yoctonear(), // every 1 deposit gives 2 sold_tokens
        sale["required_sold_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["total_payment_token"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        0,
        sale["total_fees"].as_str().unwrap().parse::<u128>().unwrap()
    );
    assert_eq!(
        NearToken::from_near(10 * 3).as_yoctonear(),
        sale["sold_tokens_for_buyers"].as_str().unwrap().parse::<u128>().unwrap()
    );

    // **************************************
    // * Stage 6: Buyer Withdraw sold token *
    // **************************************

    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;
    worker.fast_forward(500).await?;
    let _ = print_time_status(&katherine_contract, &test_utils_contract).await?;

    // Treasury don't purchased any sold-tokens; only the buyer.
    let outcome = treasury
        .call(katherine_contract.id(), "withdraw_tokens")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        // .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    assert!(outcome.is_failure());
    // println!("Buyer gets sold-tokens 🏦: {:#?}", outcome);

    let outcome: serde_json::Value = sold_token_contract
        .call("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": buyer.id()
        }))
        .view()
        .await?
        .json()?;
    let buyer_original_balance = outcome.as_str().unwrap().parse::<u128>().unwrap();

    let outcome = buyer
        .call(katherine_contract.id(), "withdraw_tokens")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        // .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    // assert!(outcome.is_failure());
    println!("Buyer gets sold-tokens 🏦: {:#?}", outcome);

    let outcome: serde_json::Value = sold_token_contract
        .call("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": buyer.id()
        }))
        .view()
        .await?
        .json()?;
    let buyer_new_balance = outcome.as_str().unwrap().parse::<u128>().unwrap();

    println!("old: {} \nnew: {}", buyer_original_balance, buyer_new_balance);
    assert_eq!(buyer_original_balance, 0);
    assert_eq!(buyer_new_balance, NearToken::from_near(10 * 2).as_yoctonear());

    // ------RETURNING PAYMENT TOKENS---------
    let outcome: serde_json::Value = usdc_token_contract
        .call("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": buyer.id()
        }))
        .view()
        .await?
        .json()?;
    let buyer_original_balance = outcome.as_str().unwrap().parse::<u128>().unwrap();

    let outcome = buyer
        .call(katherine_contract.id(), "withdraw_tokens")
        .args_json(serde_json::json!({
            "sale_id": 1
        }))
        // .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    // assert!(outcome.is_failure());
    println!("Buyer gets sold-tokens 🏦: {:#?}", outcome);

    let outcome: serde_json::Value = usdc_token_contract
        .call("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": buyer.id()
        }))
        .view()
        .await?
        .json()?;
    let buyer_new_balance = outcome.as_str().unwrap().parse::<u128>().unwrap();

    println!("old: {} \nnew: {}", buyer_original_balance, buyer_new_balance);
    assert_eq!(buyer_original_balance, 0);
    assert_eq!(buyer_new_balance, 10000000);

    // ***********************************
    // * Stage 7: Seller withdraw excess *
    // ***********************************

    println!("All tests passed ✅");
    Ok(())
}

async fn create_test_utils(
    worker: &Worker<impl DevNetwork>
) -> anyhow::Result<Contract> {
    let test_utils_contract_wasm = std::fs::read(TEST_UTILS_FILEPATH)?;
    let test_utils_contract = worker.dev_deploy(&test_utils_contract_wasm).await?;

    let _ = test_utils_contract
        .call("new")
        .args_json(serde_json::json!({}))
        .transact()
        .await?;

    Ok(test_utils_contract)
}

async fn create_sold_token(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let sold_token_contract_wasm = std::fs::read(PTOKEN_FILEPATH)?;
    let sold_token_contract = worker.dev_deploy(&sold_token_contract_wasm).await?;

    let res = sold_token_contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "decimals": 24,
            "symbol": "pTOKEN",
            "total_supply": format!("{}", NearToken::from_near(1000).as_yoctonear())
        }))
        .transact()
        .await?;
    println!("Sold-Token: {:#?}", res);

    Ok(sold_token_contract)
}

async fn create_usdc_token(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let usdc_token_contract_wasm = std::fs::read(PTOKEN_FILEPATH)?;
    let usdc_token_contract = worker.dev_deploy(&usdc_token_contract_wasm).await?;

    let res = usdc_token_contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "decimals": 6,
            "symbol": "USDC",
            "total_supply": format!("{}", NearToken::from_near(1000).as_yoctonear())
        }))
        .transact()
        .await?;
    println!("USDC-Token: {:#?}", res);

    Ok(usdc_token_contract)
}

async fn create_katherine(
    owner: &Account,
    treasury: &Account,
    usdc_token: &AccountId,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let katherine_contract_wasm = std::fs::read(KATHERINE_SALE_FILEPATH)?;
    let katherine_contract = worker.dev_deploy(&katherine_contract_wasm).await?;

    let res = katherine_contract
        .call("new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "min_deposit_amount_in_near": format!("{}", NearToken::from_near(1).as_yoctonear()),
            "min_deposit_amount_in_payment_token": "1000000", // 1 USDC
            "payment_token_contract_address": usdc_token,
            "payment_token_unit": "1000000",
            "treasury_id": treasury.id(),
            "sale_fee": 250,
        }))
        .transact()
        .await?;
    println!("KATH: {:#?}", res);

    Ok(katherine_contract)
}

async fn registering_accounts(
    sold_token_contract: &Contract,
    usdc_token_contract: &Contract,
    katherine_contract: &Contract,
    treasury: &Account,
    buyer: &Account,
) -> anyhow::Result<()> {
    // Register Accounts
    let _ = usdc_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": katherine_contract.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    let _ = usdc_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": treasury.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    let _ = usdc_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": buyer.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    let _ = sold_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": katherine_contract.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    let _ = sold_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": treasury.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    let _ = sold_token_contract
        .call("register_account")
        .args_json(serde_json::json!({
            "account_id": buyer.id(),
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    Ok(())
}

async fn create_sale(
    n: u32,
    is_in_near: bool,
    now: &Now,
    owner: &Account,
    katherine_contract: &Contract,
    sold_token_contract: &Contract,
) -> anyhow::Result<ExecutionFinalResult> {

    let one_payment_token_purchase_rate = U128::from(NearToken::from_near(2).as_yoctonear());
    let max_available_sold_token = U128::from(NearToken::from_near(100).as_yoctonear());
    let open_date_timestamp = U64::from(now.increment_min(2).to_epoch_millis());
    let close_date_timestamp = U64::from(now.increment_min(4).to_epoch_millis());
    let release_date_timestamp = U64::from(now.increment_min(6).to_epoch_millis());

    let outcome = owner
        .call(katherine_contract.id(), "create_sale")
        .args_json(json!({
            "slug": format!("test-sale-{}", n),
            "is_in_near": is_in_near,
            "sold_token_contract_address": sold_token_contract.id(),
            "one_payment_token_purchase_rate": one_payment_token_purchase_rate,
            "max_available_sold_token": max_available_sold_token,
            "open_date_timestamp": open_date_timestamp,
            "close_date_timestamp": close_date_timestamp,
            "release_date_timestamp": release_date_timestamp,
        }))
        .deposit(NearToken::from_yoctonear(STORAGE_PER_SALE-1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(outcome.is_failure());

    let outcome = owner
        .call(katherine_contract.id(), "create_sale")
        .args_json(json!({
            "slug": format!("test-sale-{}", n),
            "is_in_near": is_in_near,
            "sold_token_contract_address": sold_token_contract.id(),
            "one_payment_token_purchase_rate": one_payment_token_purchase_rate,
            "max_available_sold_token": max_available_sold_token,
            "open_date_timestamp": open_date_timestamp,
            "close_date_timestamp": close_date_timestamp,
            "release_date_timestamp": release_date_timestamp,
        }))
        .deposit(NearToken::from_yoctonear(STORAGE_PER_SALE))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    Ok(outcome)
}

async fn print_time_status(
    katherine_contract: &Contract,
    test_utils_contract: &Contract,
) -> anyhow::Result<()> {
    let outcome: serde_json::Value = katherine_contract
        .call("get_sale")
        .args_json(serde_json::json!({
            "sale_id": 0
        }))
        .view()
        .await?
        .json()?;
    let sale = outcome.as_object().unwrap();

    let now = Now::new_from_epoch_millis(test_utils_contract.call("get_now").view().await?.json()?);
    println!("NOW    : {:?}", now.to_epoch_millis());
    let ts = sale["open_date_timestamp"].as_str().unwrap().parse::<u64>().unwrap();
    println!("OPEN   : {:?}{}", ts, if ts < now.to_epoch_millis() { "*" } else { "" });
    let ts = sale["close_date_timestamp"].as_str().unwrap().parse::<u64>().unwrap();
    println!("CLOSE  : {:?}{}", ts, if ts < now.to_epoch_millis() { "*" } else { "" });
    let ts = sale["release_date_timestamp"].as_str().unwrap().parse::<u64>().unwrap();
    println!("RELEASE: {:?}{}", ts, if ts < now.to_epoch_millis() { "*" } else { "" });

    Ok(())
}