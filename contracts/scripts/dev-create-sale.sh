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

NOW_IN_MILLISECS=$(($(date +%s) * 1000))
DURATION_OF_A_DAY_IN_MILLISECS=86400000
DURATION_OF_A_WEEK_IN_MILLISECS=604800000
DURATION_OF_A_YEAR_IN_MILLISECS=31536000000

# TEST SCENARIO #1 - una en near y otra en usdc, que abra en 1 año
SALE_1_OPEN_DATE=$(($NOW_IN_MILLISECS + $DURATION_OF_A_YEAR_IN_MILLISECS))
SALE_1_CLOSE_DATE=$(($SALE_1_OPEN_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))
SALE_1_RELEASE_DATE=$(($SALE_1_CLOSE_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-0", "is_in_near": true, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "2'$YOCTO_UNITS'", "max_available_sold_token": "200'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_1_OPEN_DATE'", "close_date_timestamp": "'$SALE_1_CLOSE_DATE'", "release_date_timestamp": "'$SALE_1_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-1", "is_in_near": false, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "3'$YOCTO_UNITS'", "max_available_sold_token": "3000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_1_OPEN_DATE'", "close_date_timestamp": "'$SALE_1_CLOSE_DATE'", "release_date_timestamp": "'$SALE_1_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

# TEST SCENARIO #2 - una en near y otra en usdc, que abra hoy y cierre en 1 año
SALE_2_OPEN_DATE=$(($NOW_IN_MILLISECS + 160000))
SALE_2_CLOSE_DATE=$(($SALE_2_OPEN_DATE + $DURATION_OF_A_YEAR_IN_MILLISECS))
SALE_2_RELEASE_DATE=$(($SALE_2_CLOSE_DATE + $DURATION_OF_A_WEEK_IN_MILLISECS))

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-2", "is_in_near": true, "sold_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "4'$YOCTO_UNITS'", "max_available_sold_token": "40000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_2_OPEN_DATE'", "close_date_timestamp": "'$SALE_2_CLOSE_DATE'", "release_date_timestamp": "'$SALE_2_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-3", "is_in_near": false, "sold_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "5'$YOCTO_UNITS'", "max_available_sold_token": "5000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_2_OPEN_DATE'", "close_date_timestamp": "'$SALE_2_CLOSE_DATE'", "release_date_timestamp": "'$SALE_2_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

# TEST SCENARIO #3 - una en near y otra en usdc, que abra hoy y cierre/release mañana
SALE_3_OPEN_DATE=$(($NOW_IN_MILLISECS + 160000))
SALE_3_CLOSE_DATE=$(($SALE_3_OPEN_DATE + $DURATION_OF_A_DAY_IN_MILLISECS))
SALE_3_RELEASE_DATE=$(($SALE_3_CLOSE_DATE + $DURATION_OF_A_DAY_IN_MILLISECS))

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-4", "is_in_near": true, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "6'$YOCTO_UNITS'", "max_available_sold_token": "6000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_3_OPEN_DATE'", "close_date_timestamp": "'$SALE_3_CLOSE_DATE'", "release_date_timestamp": "'$SALE_3_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-5", "is_in_near": false, "sold_token_contract_address": "'$PTOKEN_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "7'$YOCTO_UNITS'", "max_available_sold_token": "700'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_3_OPEN_DATE'", "close_date_timestamp": "'$SALE_3_CLOSE_DATE'", "release_date_timestamp": "'$SALE_3_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

# TEST SCENARIO #4 - una en near y otra en usdc, que abra hoy y cierre mañana pero el release sea en 1 año
SALE_4_OPEN_DATE=$(($NOW_IN_MILLISECS + 160000))
SALE_4_CLOSE_DATE=$(($SALE_4_OPEN_DATE + $DURATION_OF_A_DAY_IN_MILLISECS))
SALE_4_RELEASE_DATE=$(($SALE_4_CLOSE_DATE + $DURATION_OF_A_YEAR_IN_MILLISECS))

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-6", "is_in_near": true, "sold_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "8'$YOCTO_UNITS'", "max_available_sold_token": "80'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_4_OPEN_DATE'", "close_date_timestamp": "'$SALE_4_CLOSE_DATE'", "release_date_timestamp": "'$SALE_4_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

NEAR_ENV=testnet near call $KATHERINE_CONTRACT_ADDRESS create_sale \
    '{"slug": "dev-sale-near-7", "is_in_near": false, "sold_token_contract_address": "'$AAXXII_CONTRACT_ADDRESS'", "one_payment_token_purchase_rate": "9'$YOCTO_UNITS'", "max_available_sold_token": "9000'$YOCTO_UNITS'", "open_date_timestamp": "'$SALE_4_OPEN_DATE'", "close_date_timestamp": "'$SALE_4_CLOSE_DATE'", "release_date_timestamp": "'$SALE_4_RELEASE_DATE'"}' \
    --accountId $NEAR_ACCOUNT \
    --gas $TOTAL_PREPAID_GAS \
    --depositYocto $SALE_CREATION_COST

# NEAR_ENV=testnet near view $KATHERINE_CONTRACT_ADDRESS get_sales '{"from_index": 0, "limit": 10}' --accountId $NEAR_ACCOUNT
