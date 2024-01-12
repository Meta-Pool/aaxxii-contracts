NEAR_ACCOUNT="kate_tester3.testnet"
BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000
SALE_CREATION_COST="10000000000000000000000" # 0.01 NEAR

PTOKEN_CONTRACT_ADDRESS="dev-1701718020576-78138536511074"
USDC_CONTRACT_ADDRESS="dev-1701718038513-75477534450754"
AAXXII_CONTRACT_ADDRESS="dev-1701718056213-45735667916226"
KATHERINE_CONTRACT_ADDRESS="dev-1701718074453-75641635531545"
STAKING_CONTRACT_ADDRESS="dev-1701718093376-11844838355640"
NEAR_ACCOUNT="kate_tester3.testnet"

echo "*************************************"
echo "* Creating Sales from these params: *"
echo "*************************************"
echo "Sold Token -: "$PTOKEN_CONTRACT_ADDRESS
echo "USDC Token -: "$USDC_CONTRACT_ADDRESS
echo "AAXXII Token: "$AAXXII_CONTRACT_ADDRESS
echo "Sales ------: "$KATHERINE_CONTRACT_ADDRESS
echo "Staking ----: "$STAKING_CONTRACT_ADDRESS
echo "Owner ------: "$NEAR_ACCOUNT
echo "*************************************"

RECEIVER_ACCOUNT=lucastestmetavote.testnet

# NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS register_account '{"account_id": "'$RECEIVER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT
NEAR_ENV=testnet near call $AAXXII_CONTRACT_ADDRESS ft_transfer '{"receiver_id": "'$RECEIVER_ACCOUNT'", "amount": "1234000000000000000000"}' --accountId $NEAR_ACCOUNT --depositYocto 1
NEAR_ENV=testnet near view $AAXXII_CONTRACT_ADDRESS ft_balance_of '{"account_id": "'$RECEIVER_ACCOUNT'"}' --accountId $NEAR_ACCOUNT


