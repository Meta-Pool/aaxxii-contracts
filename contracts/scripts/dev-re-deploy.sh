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

PROPOSALS_CONTRACT_ADDRESS="v010.meta-proposals.testnet"
# NEAR_ENV=testnet near deploy $PROPOSALS_CONTRACT_ADDRESS res/proposals_contract.wasm --force

# NEAR_ENV=testnet near view $PROPOSALS_CONTRACT_ADDRESS get_staking_position_contract_address '{}'
# NEAR_ENV=testnet near call $PROPOSALS_CONTRACT_ADDRESS insert_operator_role '{"account":"alpha-centauri.testnet"}' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS
# NEAR_ENV=testnet near view $PROPOSALS_CONTRACT_ADDRESS get_operators '{}'
NEAR_ENV=testnet near call $PROPOSALS_CONTRACT_ADDRESS create_proposal '{"title":"First Proposal","short_description":"New Proposal in last deployed contract", "body": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer tincidunt mi sollicitudin, auctor lectus eu, commodo elit. Quisque in tempor arcu. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer tincidunt mi sollicitudin, auctor lectus eu, commodo elit. Quisque in tempor arcu.", "data": "", "extra": ""}' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS

# insert_operator_role(&mut self, account: AccountId)