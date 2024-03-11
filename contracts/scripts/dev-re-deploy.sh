#!/bin/bash
set -e

NEAR_ACCOUNT="kate_tester3.testnet"
BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000

# # Re-Deploy
# STAKING_CONTRACT_ADDRESS="dev-1701718093376-11844838355640"
# NEAR_ENV=testnet near deploy --wasmFile res/staking_position_contract.wasm --accountId $STAKING_CONTRACT_ADDRESS

# PROPOSALS_CONTRACT_ADDRESS="meta-proposals.testnet"
# NEAR_ENV=testnet near deploy $PROPOSALS_CONTRACT_ADDRESS res/proposals_contract.wasm --force

# NEAR_ENV=testnet near view $PROPOSALS_CONTRACT_ADDRESS get_operators '{}'
# NEAR_ENV=testnet near call $PROPOSALS_CONTRACT_ADDRESS insert_operator_role '{"account":"alpha-centauri.testnet"}' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS
# insert_operator_role(&mut self, account: AccountId)