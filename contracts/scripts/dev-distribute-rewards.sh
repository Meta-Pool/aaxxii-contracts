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

# NEAR_ENV=testnet near view $STAKING_CONTRACT_ADDRESS get_stakers \
#     '{"from_index": 0, "limit": 20}' \
#     --accountId $NEAR_ACCOUNT

# ft_transfer_call(
#         &mut self,
#         receiver_id: AccountId,
#         amount: U128,
#         memo: Option<String>,
#         msg: String
#     );
# get_stakers
# NEAR_ENV=testnet near call $USDC_CONTRACT_ADDRESS ft_transfer_call \
#     '{"receiver_id": "'$STAKING_CONTRACT_ADDRESS'", "amount": "10'$USDC_UNITS'", "memo": null, "msg": "for-claims:02[[\"kate_tester3.testnet\", 100000]]"}' \
#     --accountId $NEAR_ACCOUNT \
#     --gas $TOTAL_PREPAID_GAS \
#     --depositYocto 1

NEAR_ENV=testnet near call $PTOKEN_CONTRACT_ADDRESS ft_transfer_call \
    '{"receiver_id": "'$STAKING_CONTRACT_ADDRESS'", "amount": "10'$USDC_UNITS'", "memo": null, "msg": "for-claims:02[[\"kate_tester3.testnet\", 100000]]"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto 1
