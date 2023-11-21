// use std::fs;
// use katherine_sale_contract::types::{EpochMillis, VaultId};
// use katherine_sale_contract::utils::proportional;
use near_units::{parse_gas, parse_near};
// use json;
// use std::str;
// use meta_tools::bond::BondLoaderJSON;
// use near_sdk::json_types::{U128, U64};
use near_sdk::AccountId as NearAccountId;

// // use workspaces::network::Sandbox;
use near_workspaces::{Account, AccountId, Contract, Worker, DevNetwork};
use near_gas::NearGas;

// use workspaces::result::ExecutionFinalResult;
// use workspaces::error::Error as WorkspaceError;

use meta_test_utils::now::Now;
use meta_test_utils::now;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

const KATHERINE_SALE_FILEPATH: &str = "./res/katherine_sale_contract.wasm";
const PTOKEN_FILEPATH: &str = "./res/test_p_token.wasm";
const TEST_UTILS_FILEPATH: &str = "./res/meta_test_utils.wasm";

pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    Ok(())
}

async fn create_test_utils(
    worker: &Worker<impl DevNetwork>
) -> anyhow::Result<Contract> {
    let test_utils_contract_wasm = std::fs::read(TEST_UTILS_FILEPATH)?;
    let test_utils_contract = worker.dev_deploy(&test_utils_contract_wasm).await?;

    let res = test_utils_contract
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
            "total_supply": format!("{}", parse_near!("1000 N"))
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
            "total_supply": format!("{}", parse_near!("1000 N"))
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
            "min_deposit_amount_in_near": format!("{}", parse_near!("1 N")),
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