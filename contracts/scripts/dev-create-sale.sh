NEAR_ACCOUNT="kate_tester3.testnet"
BUYER_ACCOUNT="jomsox.testnet"
YOCTO_UNITS="000000000000000000000000"
USDC_UNITS="000000"
TOTAL_PREPAID_GAS=300000000000000
SALE_CREATION_COST="10000000000000000000000" # 0.01 NEAR

PTOKEN_CONTRACT_ADDRESS="dev-1700222788132-28726601756903"
USDC_CONTRACT_ADDRESS="dev-1700222819018-99235604921228"
KATHERINE_CONTRACT_ADDRESS="dev-1700222862343-99136631464723"

echo "*************************************"
echo "* Creating Sales from these params: *"
echo "*************************************"
echo "Sold Token: "$PTOKEN_CONTRACT_ADDRESS
echo "USDC Token: "$USDC_CONTRACT_ADDRESS
echo "Katherine-: "$KATHERINE_CONTRACT_ADDRESS
echo "Owner ----: "$NEAR_ACCOUNT
echo "*************************************"

NOW_IN_MILLISECS=$(($(date +%s) * 1000))
DURATION_OF_A_WEEK_IN_MILLISECS=604800000

SALE_1_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
SALE_1_CLOSE_DATE=$(($SALE_1_OPEN_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))
SALE_1_RELEASE_DATE=$(($SALE_1_CLOSE_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))

SALE_2_OPEN_DATE=$(($NOW_IN_MILLISECS + 60000))
SALE_2_CLOSE_DATE=$(($SALE_2_OPEN_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))
SALE_2_RELEASE_DATE=$(($SALE_2_CLOSE_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))

echo "------------------ Creating first dev-Sale:"
# NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale '{\
#     "slug": "dev-sale-usdc-0", \
#     "is_in_near": false, \
#     "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", \
#     "one_payment_token_purchase_rate": "25'$YOCTO_UNITS'", \
#     "max_available_sold_token": "25000'$YOCTO_UNITS'", \
#     "open_date_timestamp": "'$SALE_1_OPEN_DATE'", \
#     "close_date_timestamp": "'$SALE_1_CLOSE_DATE'", \
#     "release_date_timestamp": "'$SALE_1_RELEASE_DATE'" \
#     }' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS --deposit $SALE_CREATION_COST

# NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale '{"slug": "dev-sale-usdc-0", "is_in_near": false, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "25'$YOCTO_UNITS'", "max_available_sold_token": "25000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_1_OPEN_DATE'", "close_date_timestamp": "'$SALE_1_CLOSE_DATE'", "release_date_timestamp": "'$SALE_1_RELEASE_DATE'"}' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS --depositYocto $SALE_CREATION_COST

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale '{"slug": "dev-sale-near-1", "is_in_near": true, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "4'$YOCTO_UNITS'", "max_available_sold_token": "25000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_1_OPEN_DATE'", "close_date_timestamp": "'$SALE_1_CLOSE_DATE'", "release_date_timestamp": "'$SALE_1_RELEASE_DATE'"}' --accountId $NEAR_ACCOUNT --gas $TOTAL_PREPAID_GAS --depositYocto $SALE_CREATION_COST


NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_sales '{"from_index": 0, "limit": 10}' --accountId $NEAR_ACCOUNT
