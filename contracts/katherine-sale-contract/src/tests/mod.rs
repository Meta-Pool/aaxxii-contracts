use super::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::testing_env;

mod utils;
use utils::*;

const E20: u128 = 100_000_000_000_000_000_000;

fn new_katherine_contract() -> KatherineSaleContract {
    KatherineSaleContract::new(
        owner_account(),
        U128::from(MIN_DEPOSIT_AMOUNT_IN_NEAR),
        U128::from(MIN_DEPOSIT_AMOUNT_IN_PAYMENT_TOKEN),
        usdt_token_contract(),
        U128::from(USDT_UNIT),
        SALE_FEE,
    )
}

fn setup_new_test() -> KatherineSaleContract {
    let call_context = get_context(
        &meta_token_account(),
        ntoy(TEST_INITIAL_BALANCE),
        0,
        to_ts(GENESIS_TIME_IN_DAYS),
    );
    testing_env!(call_context.clone());
    new_metavote_contract()
}

#[test]
fn test_single_deposit() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E24);
    let msg: String = "30".to_owned();

    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());
    assert_eq!(1, contract.voters.len(), "Voter was not created!");

    let voter = contract.internal_get_voter(&sender_id);
    assert_eq!(
        1,
        voter.locking_positions.len(),
        "Locking position was not created!"
    );

    let vote_power =
        contract.calculate_voting_power(Meta::from(amount), msg.parse::<Days>().unwrap());
    assert_eq!(
        vote_power, voter.voting_power,
        "Incorrect voting power calculation!"
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 1);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 0);
}
