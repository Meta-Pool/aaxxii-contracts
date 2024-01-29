#!/bin/bash
set -e

NEAR_ACCOUNT="kate_tester3.testnet"
BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000

# Re-Deploy
AAXXII_STAKING_POSITION_ADDRESS="aaxxii-stake.near"
NEAR_ENV=mainnet near deploy --wasmFile res/staking_position_contract.wasm --accountId $AAXXII_STAKING_POSITION_ADDRESS


# NEAR_ENV=mainnet near call aaxxii-stake.near update_min_deposit_amount '{"new_value": "100000"}' --accountId "jomsox.near" --depositYocto 1