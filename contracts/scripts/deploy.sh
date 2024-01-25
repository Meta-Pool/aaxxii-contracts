#!/bin/bash
set -e

AAXXII_STAKING_POSITION_ADDRESS="aaxxii-stake.near"
USDC_CONTRACT_ADDRESS="usdt.tether-token.near"
AAXXII_CONTRACT_ADDRESS="aaxxii.laboratory.jumpfinance.near"
NEAR_ACCOUNT="jomsox.near"
# BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000

# Deploying tokens
NEAR_ENV=mainnet near deploy --accountId aaxxii-stake.near --wasmFile res/staking_position_contract.wasm --initFunction new --initArgs '{"owner_id": "jomsox.near", "min_locking_period": 30, "max_locking_period": 30, "min_deposit_amount": "100000000000000000000000", "max_locking_positions": 5, "max_voting_positions": 5, "underlying_token_contract_address": "aaxxii.laboratory.jumpfinance.near", "available_claimable_ft_addresses": ["usdt.tether-token.near"]}'

# echo "Sold Token -: "$PTOKEN_CONTRACT_ADDRESS
echo "USDC Token -: "$USDC_CONTRACT_ADDRESS
echo "AAXXII Token: "$AAXXII_CONTRACT_ADDRESS
# echo "Sales ------: "$KATHERINE_CONTRACT_ADDRESS
# echo "Staking ----: "$STAKING_CONTRACT_ADDRESS
# echo "Owner ------: "$NEAR_ACCOUNT

# echo "------------------ Registering accounts"
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# #
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# #
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
