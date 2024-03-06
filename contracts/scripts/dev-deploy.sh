#!/bin/bash
set -e

NEAR_ACCOUNT="kate_tester3.testnet"
BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000
# Proposals
OPERATOR_ADDRESS="kate_tester3.testnet"
VOTING_PERIOD_MS="86400000" # 1 day in milliseconds
MIN_VOTING_POWER_AMOUNT="1000"
PROPOSAL_STORAGE_NEAR="100000000000000000000000" # 0.1 NEAR
QUORUM_FLOOR=1000

rm -rf neardev_staking/
rm -rf neardev_sales/
rm -rf neardev_ptoken/
rm -rf neardev_usdc/
rm -rf neardev_aaxxii/
rm -rf neardev_proposals/

# # Deploying tokens
# NEAR_ENV=testnet near dev-deploy --wasmFile res/test_p_token.wasm \
#     --initFunction new_default_meta \
#     --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "total_supply": "1000000000'$YOCTO_UNITS'", "symbol": "pToken", "decimals": 24}' \
#     --accountId $NEAR_ACCOUNT
# mv neardev/ neardev_ptoken/
# NEAR_ENV=testnet near dev-deploy --wasmFile res/test_p_token.wasm \
#     --initFunction new_default_meta \
#     --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "total_supply": "1000000000'$USDC_UNITS'", "symbol": "USDC", "decimals": 6}' \
#     --accountId $NEAR_ACCOUNT
# mv neardev/ neardev_usdc/
# NEAR_ENV=testnet near dev-deploy --wasmFile res/test_p_token.wasm \
#     --initFunction new_default_meta \
#     --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "total_supply": "1000000000'$YOCTO_UNITS'", "symbol": "AAXXII", "decimals": 24}' \
#     --accountId $NEAR_ACCOUNT
# mv neardev/ neardev_aaxxii/

# PTOKEN_CONTRACT_ADDRESS=$(head -n1 ./neardev_ptoken/dev-account)
# USDC_CONTRACT_ADDRESS=$(head -n1 ./neardev_usdc/dev-account)
# AAXXII_CONTRACT_ADDRESS=$(head -n1 ./neardev_aaxxii/dev-account)

# # Deploying Sales, Staking and Proposals
# NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_sale_contract.wasm \
#     --initFunction new \
#     --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "min_deposit_amount_in_near": "1'$YOCTO_UNITS'", "min_deposit_amount_in_payment_token": "10'$USDC_UNITS'", "payment_token_contract_address": "'$USDC_CONTRACT_ADDRESS'", "payment_token_unit": "1'$USDC_UNITS'", "treasury_id": "'$NEAR_ACCOUNT'", "sale_fee": 200 }' \
#     --accountId $NEAR_ACCOUNT
# mv neardev/ neardev_sales/
# NEAR_ENV=testnet near dev-deploy --wasmFile res/staking_position_contract.wasm \
#     --initFunction new \
#     --initArgs '{"owner_id": "'$NEAR_ACCOUNT'", "min_locking_period": 30, "max_locking_period": 30, "min_deposit_amount": "1'$YOCTO_UNITS'", "max_locking_positions": 5, "max_voting_positions": 5, "underlying_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "available_claimable_ft_addresses": ["'$USDC_CONTRACT_ADDRESS'"]}' \
#     --accountId $NEAR_ACCOUNT
# mv neardev/ neardev_staking/

# STAKING_CONTRACT_ADDRESS=$(head -n1 ./neardev_staking/dev-account)
# KATHERINE_CONTRACT_ADDRESS=$(head -n1 ./neardev_sales/dev-account)

# Deploying proposals
AAXXII_CONTRACT_ADDRESS='dev-1701718056213-45735667916226'
STAKING_CONTRACT_ADDRESS='dev-1701718093376-11844838355640'

# NEAR_ENV=testnet near deploy meta-proposals.testnet res/proposals_contract.wasm \
#     --initFunction new \
#     --initArgs '{"admin_id": "'$NEAR_ACCOUNT'", "operator_ids": ["'$OPERATOR_ADDRESS'"], "asset_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "staking_position_contract_address": "'$STAKING_CONTRACT_ADDRESS'", "voting_period":"'$VOTING_PERIOD_MS'", "min_voting_power_amount": "'$MIN_VOTING_POWER_AMOUNT'", "proposal_storage_near": "'$PROPOSAL_STORAGE_NEAR'", "quorum_floor": '$QUORUM_FLOOR' }' \
# # mv neardev/ neardev_proposals/

# PROPOSALS_CONTRACT_ADDRESS=$(head -n1 ./neardev_proposals/dev-account)
PROPOSALS_CONTRACT_ADDRESS="meta-proposals.testnet"

echo "Sold Token -: "$PTOKEN_CONTRACT_ADDRESS
echo "USDC Token -: "$USDC_CONTRACT_ADDRESS
echo "AAXXII Token: "$AAXXII_CONTRACT_ADDRESS
echo "Sales ------: "$KATHERINE_CONTRACT_ADDRESS
echo "Staking ----: "$STAKING_CONTRACT_ADDRESS
echo "Proposals --: "$PROPOSALS_CONTRACT_ADDRESS
echo "Owner ------: "$NEAR_ACCOUNT

echo "------------------ Registering accounts"
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS register_account '{"account_id": "'$PROPOSALS_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
#
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS register_account '{"account_id": "'$PROPOSALS_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
#
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$BUYER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$KATHERINE_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$STAKING_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$PROPOSALS_CONTRACT_ADDRESS'"}' --accountId $NEAR_ACCOUNT
