use crate::utils::{days_to_millis, millis_to_days};
use crate::{constants::*, locking_position::*};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::unordered_map::UnorderedMap;
use near_sdk::collections::{Vector, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::{assert_one_yocto, env, log, near_bindgen, require, AccountId, Balance, PanicOnDefault};
use types::*;
use utils::{generate_hash_id, get_current_epoch_millis};
use staker::{Staker, StakerJSON};

mod constants;
mod deposit;
mod interface;
mod internal;
mod locking_position;
mod types;
mod utils;
mod staker;
mod withdraw;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StakingPositionContract {
    pub owner_id: AccountId,
    pub stakers: UnorderedMap<AccountId, Staker>,
    
    /// Total amount of voting power for an address - votable id.
    pub total_voting_power: VotingPower,
    pub votes: UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, VotingPower>>,
    pub min_locking_period: Days,
    pub max_locking_period: Days,

    pub min_deposit_amount: Balance,
    pub max_locking_positions: u8,
    pub max_voting_positions: u8,
    pub underlying_token_contract_address: AccountId,

    /// Stakers can claim NEAR.
    pub claimable_near: UnorderedMap<VoterId, u128>,
    pub accum_near_distributed_for_claims: u128, // accumulated total NEAR distributed
    pub total_unclaimed_near: u128,              // currently unclaimed NEAR

    /// Stakers can claim any FT token. Key is the ft address.
    pub claimable_ft: UnorderedMap<AccountId, FtDetails>,
}

#[near_bindgen]
impl StakingPositionContract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        min_locking_period: Days,
        max_locking_period: Days,
        min_deposit_amount: U128,
        max_locking_positions: u8,
        max_voting_positions: u8,
        underlying_token_contract_address: AccountId,
        available_claimable_ft_addresses: Vec<AccountId>,
    ) -> Self {
        // require!(!env::state_exists(), "The contract is already initialized");
        require!(
            min_locking_period <= max_locking_period,
            "Review the min and max locking period"
        );
        let mut contract = Self {
            owner_id,
            stakers: UnorderedMap::new(StorageKey::Stakers),
            total_voting_power: 0,
            votes: UnorderedMap::new(StorageKey::Votes),
            min_locking_period,
            max_locking_period,
            min_deposit_amount: min_deposit_amount.0,
            max_locking_positions,
            max_voting_positions,
            underlying_token_contract_address,
            claimable_near: UnorderedMap::new(StorageKey::ClaimableNear),
            accum_near_distributed_for_claims: 0,
            total_unclaimed_near: 0,
            claimable_ft: UnorderedMap::new(StorageKey::ClaimableFt),
        };
        for token_address in available_claimable_ft_addresses.iter() {
            contract.insert_new_ft(token_address);
        }

        contract
    }

    // ****************************
    // * Update contract settings *
    // ****************************

    /// The available ft addresses will never decrease in length.
    #[payable]
    pub fn insert_claimable_ft_addresses(&mut self, new_value: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.insert_new_ft(&new_value);
    }

    // *********
    // * claim *
    // *********

    // pub fn claim_near(&mut self, amount: U128) {
    //     let amount = amount.0;
    //     self.assert_min_deposit_amount(amount);
    //     let voter_id = VoterId::from(env::predecessor_account_id());
    //     self.remove_claimable_meta(&voter_id, amount);
    //     let mut voter = self.internal_get_voter_or_panic(&voter_id);
    //     // create/update locking position
    //     self.deposit_locking_position(amount, locking_period, voter_id, &mut voter);
    // }

    // // claim stNear
    // pub fn claim_stnear(&mut self, amount: U128) {
    //     let amount = amount.0;
    //     let voter_id = VoterId::from(env::predecessor_account_id());
    //     self.remove_claimable_stnear(&voter_id, amount);

    //     // IMPORTANT: if user is not a voter, then the claim is not available.
    //     let _voter = self.internal_get_voter_or_panic(&voter_id);
    //     self.transfer_stnear_to_voter(voter_id, amount);
    // }

    // ****************
    // * NEAR deposit *
    // ****************

    #[payable]
    pub fn deposit_near(
        &mut self,
        distribute_info: Vec<(AccountId, U128)>
    ) {
        let total_amount = env::attached_deposit();
        require!(total_amount > 0, "Zero NEAR deposit.");
        self.distribute_near_claims(total_amount, distribute_info);
    }

    // *************
    // * Unlocking *
    // *************

    pub fn unlock_position(&mut self, index: PositionIndex) {
        let mut staker = self.internal_get_staker_or_panic();
        let mut locking_position = staker.get_position(index);

        let voting_power = locking_position.voting_power;
        assert!(
            staker.voting_power >= voting_power,
            "Not enough free voting power to unlock! You have {}, required {}.",
            staker.voting_power,
            voting_power
        );

        log!("UNLOCK: {} unlocked position {}.", &staker.id, index);
        locking_position.unlocking_started_at = Some(get_current_epoch_millis());
        staker.locking_positions.replace(index as u64, &locking_position);
        staker.voting_power -= voting_power;
        self.total_voting_power = self.total_voting_power.saturating_sub(voting_power);
        self.stakers.insert(&staker.id, &staker);
    }

    /// @param amount - The amount to unlock.
    pub fn unlock_partial_position(&mut self, index: PositionIndex, amount: U128) {
        let mut staker = self.internal_get_staker_or_panic();
        let mut locking_position = staker.get_position(index);

        let locking_period = locking_position.locking_period;
        let amount = amount.0;

        // If the amount equals the total, then the unlock is not partial.
        if amount == locking_position.amount {
            return self.unlock_position(index);
        }
        require!(locking_position.amount > amount, "Amount too large!");
        assert!(
            (locking_position.amount - amount) >= self.min_deposit_amount,
            "A locking position cannot have less than {}",
            self.min_deposit_amount
        );
        let remove_voting_power = self.calculate_voting_power(amount, locking_period);
        assert!(
            locking_position.voting_power >= remove_voting_power,
            "Not enough free voting power to unlock! Locking position has {}, required {}.",
            locking_position.voting_power,
            remove_voting_power
        );
        assert!(
            staker.voting_power >= remove_voting_power,
            "Not enough free voting power to unlock! You have {}, required {}.",
            staker.voting_power,
            remove_voting_power
        );

        log!("UNLOCK: {} partially unlocked position {}.", &staker.id, index);
        // Create a NEW unlocking position
        self.create_unlocking_position(&mut staker, amount, locking_period, remove_voting_power);

        // Decrease current locking position
        locking_position.voting_power -= remove_voting_power;
        locking_position.amount -= amount;
        staker.locking_positions.replace(index as u64, &locking_position);

        staker.voting_power -= remove_voting_power;
        self.total_voting_power = self.total_voting_power.saturating_sub(remove_voting_power);
        self.stakers.insert(&staker.id, &staker);
    }

    // ********************************
    // * extend locking position days *
    // ********************************

    pub fn locking_position_extend_days(
        &mut self,
        index: PositionIndex,
        new_locking_period: Days
    ) {
        let mut staker = self.internal_get_staker_or_panic();
        let mut locking_position = staker.get_position(index);

        // position should be locked
        require!(
            locking_position.unlocking_started_at.is_none(),
            "position should be locked in order to extend time"
        );
        require!(
            new_locking_period > locking_position.locking_period,
            "new auto-lock period should be greater than previous one"
        );

        log!(
            "EXTEND-TIME: {} position #{} {} days",
            &staker.id,
            index,
            new_locking_period
        );

        let old_voting_power = locking_position.voting_power;
        let new_voting_power =
            self.calculate_voting_power(locking_position.amount, new_locking_period);

        // update to new total-voting-power (add delta)
        self.total_voting_power += new_voting_power - old_voting_power;

        // update to new voter-voting-power (add delta)
        staker.voting_power += new_voting_power - old_voting_power;

        // update position
        locking_position.locking_period = new_locking_period;
        locking_position.voting_power = new_voting_power;

        // save
        staker.locking_positions.replace(index as u64, &locking_position);
        self.stakers.insert(&staker.id, &staker);
    }

    // ***********
    // * Re-Lock *
    // ***********

    pub fn relock_position(
        &mut self,
        index: PositionIndex,
        locking_period: Days,
        amount_from_balance: U128,
    ) {
        let mut staker = self.internal_get_staker_or_panic();
        let locking_position = staker.get_position(index);

        // Check voter balance and unlocking position amount.
        let amount_from_balance = amount_from_balance.0;
        assert!(
            staker.balance >= amount_from_balance,
            "Not enough balance. You have {} in balance, required {}.",
            staker.balance,
            amount_from_balance
        );
        // Check if position is unlocking.
        require!(
            locking_position.unlocking_started_at.is_some(),
            "Cannot re-lock a locked position."
        );

        let now = get_current_epoch_millis();
        let unlocking_date = locking_position.unlocking_started_at.unwrap()
            + locking_position.locking_period_millis();

        if now < unlocking_date {
            // Position is still in the **unlocking** period, (unlocking_date is in the future)
            let remaining = unlocking_date - now;
            assert!(
                remaining < days_to_millis(locking_period),
                "The new locking period should be greater than {} days.",
                millis_to_days(remaining)
            );
        }

        log!("RELOCK: {} relocked position {}.", &staker.id, index);
        let amount = locking_position.amount + amount_from_balance;
        staker.remove_position(index);
        staker.balance -= amount_from_balance;
        self.deposit_locking_position(amount, locking_period, &mut staker);
    }

    // TODO: here you can unify the amount from position and from balance.
    pub fn relock_partial_position(
        &mut self,
        index: PositionIndex,
        amount_from_position: U128,
        // amount: U128,
        locking_period: Days,
        amount_from_balance: U128,
    ) {
        let mut staker = self.internal_get_staker_or_panic();
        let mut locking_position = staker.get_position(index);

        // Check voter balance and unlocking position amount.
        let amount_from_balance = amount_from_balance.0;
        let amount_from_position = amount_from_position.0;
        assert!(
            staker.balance >= amount_from_balance,
            "Not enough balance. You have {} in balance, required {}.",
            staker.balance,
            amount_from_balance
        );
        assert!(
            locking_position.amount >= amount_from_position,
            "Locking position amount is not enough. Locking position has {}, required {}.",
            locking_position.amount,
            amount_from_position
        );
        let amount = amount_from_balance + amount_from_position;
        assert!(
            amount >= self.min_deposit_amount,
            "A locking position cannot have less than {}.",
            self.min_deposit_amount
        );
        // Check if position is unlocking.
        require!(
            locking_position.unlocking_started_at.is_some(),
            "Cannot re-lock a locked position."
        );

        let now = get_current_epoch_millis();
        let unlocking_date = locking_position.unlocking_started_at.unwrap()
            + locking_position.locking_period_millis();

        if now < unlocking_date {
            // Position is **unlocking**.
            let remaining = unlocking_date - now;
            assert!(
                remaining < days_to_millis(locking_period),
                "The new locking period should be greater than {} days.",
                millis_to_days(remaining)
            );

            let new_amount = locking_position.amount - amount_from_position;
            assert!(
                amount >= self.min_deposit_amount,
                "A locking position cannot have less than {}.",
                self.min_deposit_amount
            );
            assert!(new_amount > 0, "Use relock_position() function instead.");

            locking_position.amount = new_amount;
            staker.locking_positions.replace(index as u64, &locking_position);
        } else {
            staker.balance += locking_position.amount - amount_from_position;
            staker.remove_position(index);
        }

        log!("RELOCK: {} partially relocked position {}.", &staker.id, index);
        staker.balance -= amount_from_balance;
        self.deposit_locking_position(amount, locking_period, &mut staker);
    }

    pub fn relock_from_balance(&mut self, locking_period: Days, amount_from_balance: U128) {
        let mut staker = self.internal_get_staker_or_panic();

        let amount = amount_from_balance.0;
        assert!(
            staker.balance >= amount,
            "Not enough balance. You have {} in balance, required {}.",
            staker.balance,
            amount
        );
        assert!(
            amount >= self.min_deposit_amount,
            "A locking position cannot have less than {}.",
            self.min_deposit_amount
        );

        log!("RELOCK: {} relocked position.", &staker.id);
        staker.balance -= amount;
        self.deposit_locking_position(amount, locking_period, &mut staker);
    }

    // ******************
    // * Clear Position *
    // ******************

    // clear SEVERAL locking positions
    pub fn clear_locking_position(&mut self, position_index_list: Vec<PositionIndex>) {
        require!(position_index_list.len() > 0, "Index list is empty.");
        let mut staker = self.internal_get_staker_or_panic();
        let mut position_index_list = position_index_list;

        position_index_list.sort();
        position_index_list.reverse();
        for index in position_index_list {
            let locking_position = staker.get_position(index);
            if locking_position.is_unlocked() {
                staker.balance += locking_position.amount;
                staker.remove_position(index);
            }
        }
        self.stakers.insert(&staker.id, &staker);
    }

    // ************
    // * Withdraw *
    // ************

    pub fn withdraw(
        &mut self,
        position_index_list: Vec<PositionIndex>,
        amount_from_balance: U128
    ) {
        let staker = self.internal_get_staker_or_panic();
        let amount_from_balance = amount_from_balance.0;
        assert!(
            staker.balance >= amount_from_balance,
            "Not enough balance. You have {} in balance, required {}.",
            staker.balance,
            amount_from_balance
        );
        let remaining_balance = staker.balance - amount_from_balance;
        // Clear locking positions, it can increase the voter balance.
        if position_index_list.len() > 0 {
            self.clear_locking_position(position_index_list);
        }
        // get voter again, because clear_locking_position alters the state
        let mut staker = self.internal_get_staker_or_panic();
        let total_to_withdraw = staker.balance - remaining_balance;
        require!(total_to_withdraw > 0, "Nothing to withdraw.");
        staker.balance -= total_to_withdraw;

        if staker.is_empty() {
            self.stakers.remove(&staker.id);
            log!("GODSPEED: {} is no longer part of Staking-contract!", &staker.id);
        } else {
            self.stakers.insert(&staker.id, &staker);
        }
        self.transfer_balance_to_voter(staker.id, total_to_withdraw);
    }

    pub fn withdraw_all(&mut self) {
        let staker = self.internal_get_staker_or_panic();

        let position_index_list = staker.get_unlocked_position_index();
        // Clear locking positions could increase the voter balance.
        if position_index_list.len() > 0 {
            self.clear_locking_position(position_index_list);
        }
        // get voter again because clear locking positions could increase the voter balance.
        let mut staker = self.internal_get_staker_or_panic();
        let total_to_withdraw = staker.balance;
        require!(total_to_withdraw > 0, "Nothing to withdraw.");
        staker.balance = 0;

        if staker.is_empty() {
            self.stakers.remove(&staker.id);
            log!("GODSPEED: {} is no longer part of Staking-contract!", &staker.id);
        } else {
            self.stakers.insert(&staker.id, &staker);
        }
        self.transfer_balance_to_voter(staker.id, total_to_withdraw);
    }

    // **********
    // * Voting *
    // **********

    pub fn vote(
        &mut self,
        voting_power: U128,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        let mut staker = self.internal_get_staker_or_panic();
        let voting_power = VotingPower::from(voting_power);
        assert!(
            staker.voting_power >= voting_power,
            "Not enough free voting power. You have {}, requested {}.",
            staker.voting_power,
            voting_power
        );
        assert!(
            staker.vote_positions.len() <= self.max_voting_positions as u64,
            "Cannot exceed {} voting positions.",
            self.max_voting_positions
        );

        let mut votes_for_address = staker.get_votes_for_address(
            &staker.id,
            &contract_address
        );
        let mut votes = votes_for_address.get(&votable_object_id).unwrap_or(0_u128);

        staker.voting_power -= voting_power;
        votes += voting_power;
        votes_for_address.insert(&votable_object_id, &votes);
        staker.vote_positions.insert(&contract_address, &votes_for_address);
        self.stakers.insert(&staker.id, &staker);

        log!(
            "VOTE: {} gave {} votes for object {} at address {}.",
            &staker.id,
            voting_power.to_string(),
            &votable_object_id,
            contract_address.as_str()
        );

        // Update contract state.
        self.internal_increase_total_votes(voting_power, &contract_address, &votable_object_id);
    }

    pub fn rebalance(
        &mut self,
        voting_power: U128,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        let mut staker = self.internal_get_staker_or_panic();
        let voting_power = VotingPower::from(voting_power);

        let mut votes_for_address = staker.get_votes_for_address(&staker.id, &contract_address);
        let mut votes = votes_for_address
            .get(&votable_object_id)
            .expect("Rebalance not allowed for nonexisting Votable Object.");

        require!(
            votes != voting_power,
            "Cannot rebalance to same Voting Power."
        );
        if voting_power == 0 {
            return self.unvote(contract_address, votable_object_id);
        }

        if votes < voting_power {
            // Increase votes.
            let additional_votes = voting_power - votes;
            assert!(
                staker.voting_power >= additional_votes,
                "Not enough free voting power to unlock! You have {}, required {}.",
                staker.voting_power,
                additional_votes
            );
            staker.voting_power -= additional_votes;
            votes += additional_votes;

            log!(
                "VOTE: {} increased to {} votes for object {} at address {}.",
                &staker.id,
                voting_power.to_string(),
                &votable_object_id,
                contract_address.as_str()
            );

            self.internal_increase_total_votes(
                additional_votes,
                &contract_address,
                &votable_object_id,
            );
        } else {
            // Decrease votes.
            let remove_votes = votes - voting_power;
            staker.voting_power += remove_votes;
            votes -= remove_votes;

            log!(
                "VOTE: {} decreased to {} votes for object {} at address {}.",
                &staker.id,
                voting_power.to_string(),
                &votable_object_id,
                contract_address.as_str()
            );

            self.internal_decrease_total_votes(remove_votes, &contract_address, &votable_object_id);
        }
        votes_for_address.insert(&votable_object_id, &votes);
        staker.vote_positions.insert(&contract_address, &votes_for_address);
        self.stakers.insert(&staker.id, &staker);
    }

    pub fn unvote(&mut self, contract_address: ContractAddress, votable_object_id: VotableObjId) {
        let mut staker = self.internal_get_staker_or_panic();
        let mut votes_for_address = staker.get_votes_for_address(
            &staker.id,
            &contract_address
        );
        let votes = votes_for_address
            .get(&votable_object_id)
            .expect("Cannot unvote a Votable Object without votes.");

        staker.voting_power += votes;
        votes_for_address.remove(&votable_object_id);

        if votes_for_address.is_empty() {
            staker.vote_positions.remove(&contract_address);
        } else {
            staker.vote_positions
                .insert(&contract_address, &votes_for_address);
        }
        self.stakers.insert(&staker.id, &staker);

        log!(
            "UNVOTE: {} unvoted object {} at address {}.",
            &staker.id,
            &votable_object_id,
            contract_address.as_str()
        );

        // Update contract state.
        self.internal_decrease_total_votes(votes, &contract_address, &votable_object_id);
    }

    // *********
    // * Admin *
    // *********

    pub fn update_min_locking_period(&mut self, new_period: Days) {
        self.assert_only_owner();
        self.min_locking_period = new_period;
    }

    /**********************/
    /*   View functions   */
    /**********************/

    pub fn get_owner_id(&self) -> String {
        self.owner_id.to_string()
    }

    pub fn get_voters_count(&self) -> u32 {
        self.stakers.len().try_into().unwrap()
    }

    pub fn get_total_voting_power(&self) -> U128 {
        U128::from(self.total_voting_power)
    }

    // get all information for a single voter: voter + locking-positions + voting-positions
    pub fn get_staker_info(&self, account_id: AccountId) -> StakerJSON {
        self.stakers.get(&account_id).unwrap().to_json()
    }

    // get all information for multiple voters, by index: Vec<voter + locking-positions + voting-positions>
    pub fn get_stakers(&self, from_index: u32, limit: u32) -> Vec<StakerJSON> {
        let keys = self.stakers.keys_as_vector();
        let voters_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;

        let mut results = Vec::<StakerJSON>::new();
        for index in start..std::cmp::min(start + limit, voters_len) {
            let staker_id = keys.get(index).unwrap();
            let staker = self.stakers.get(&staker_id).unwrap();
            results.push(staker.to_json());
        }
        results
    }

    pub fn get_balance(&self, account_id: AccountId) -> U128 {
        let staker = self.internal_get_staker(account_id);
        let balance = staker.balance + staker.sum_unlocked();
        U128::from(balance)
    }

    pub fn get_claimable_near(&self, account_id: AccountId) -> U128 {
        U128::from(self.claimable_near.get(&account_id).unwrap_or(0))
    }

    pub fn get_claimable_ft(
        &self,
        staker_id: AccountId,
        token_address: AccountId
    ) -> U128 {
        U128::from(
            self.claimable_ft.get(&token_address)
                .expect("Invalid ft token")
                .owners
                .get(&staker_id)
                .unwrap_or(0)
        )
    }

    // get all claims
    // TODO: We need an ft get all claims?
    pub fn get_near_claims(&self, from_index: u32, limit: u32) -> Vec<(AccountId, U128)> {
        let mut results = Vec::<(AccountId, U128)>::new();
        let keys = self.claimable_near.keys_as_vector();
        let start = from_index as u64;
        let limit = limit as u64;
        for index in start..std::cmp::min(start + limit, keys.len()) {
            let voter_id = keys.get(index).unwrap();
            let amount = self.claimable_near.get(&voter_id).unwrap();
            results.push((voter_id, amount.into()));
        }
        results
    }

    pub fn get_locked_balance(&self, account_id: AccountId) -> U128 {
        let staker = self.internal_get_staker(account_id);
        U128::from(staker.sum_locked())
    }

    pub fn get_unlocking_balance(&self, account_id: AccountId) -> U128 {
        let staker = self.internal_get_staker(account_id);
        U128::from(staker.sum_unlocking())
    }

    pub fn get_available_voting_power(&self, account_id: AccountId) -> U128 {
        let staker = self.internal_get_staker(account_id);
        U128::from(staker.voting_power)
    }

    pub fn get_used_voting_power(&self, account_id: AccountId) -> U128 {
        let staker = self.internal_get_staker(account_id);
        U128::from(staker.sum_used_votes())
    }

    pub fn get_locking_period(&self) -> (Days, Days) {
        (self.min_locking_period, self.max_locking_period)
    }

    // all locking positions for a voter
    pub fn get_all_locking_positions(
        &self,
        account_id: AccountId
    ) -> Vec<LockingPositionJSON> {
        let mut result = Vec::new();
        let staker = self.internal_get_staker(account_id);
        for index in 0..staker.locking_positions.len() {
            let locking_position = staker.locking_positions
                .get(index)
                .expect("Locking position not found!");
            result.push(locking_position.to_json(Some(index.try_into().unwrap())));
        }
        result
    }

    pub fn get_locking_position(
        &self,
        index: PositionIndex,
        account_id: AccountId,
    ) -> Option<LockingPositionJSON> {
        let staker = self.internal_get_staker(account_id);
        match staker.locking_positions.get(index as u64) {
            Some(locking_position) => Some(locking_position.to_json(Some(index))),
            None => None,
        }
    }

    // votes by app and votable_object
    pub fn get_total_votes(
        &self,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) -> U128 {
        let votes = match self.votes.get(&contract_address) {
            Some(object) => object.get(&votable_object_id).unwrap_or(0_u128),
            None => 0_u128,
        };
        U128::from(votes)
    }

    // votes by app (contract)
    pub fn get_votes_by_contract(
        &self,
        contract_address: ContractAddress,
    ) -> Vec<VotableObjectJSON> {
        let objects = self
            .votes
            .get(&contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::Votes));

        let mut results: Vec<VotableObjectJSON> = Vec::new();
        for (id, voting_power) in objects.iter() {
            results.push(VotableObjectJSON {
                votable_contract: contract_address.to_string(),
                id,
                current_votes: U128::from(voting_power),
            })
        }
        results.sort_by_key(|v| v.current_votes.0);
        results
    }

    // given a voter, total votes per app + object_id
    pub fn get_votes_by_voter(&self, account_id: AccountId) -> Vec<VotableObjectJSON> {
        let mut results: Vec<VotableObjectJSON> = Vec::new();
        let staker = self.internal_get_staker(account_id);
        for contract_address in staker.vote_positions.keys_as_vector().iter() {
            let votes_for_address = staker.vote_positions.get(&contract_address).unwrap();
            for (id, voting_power) in votes_for_address.iter() {
                results.push(VotableObjectJSON {
                    votable_contract: contract_address.to_string(),
                    id,
                    current_votes: U128::from(voting_power),
                })
            }
        }
        results.sort_by_key(|v| v.current_votes.0);
        results
    }

    pub fn get_votes_for_object(
        &self,
        account_id: AccountId,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) -> U128 {
        let staker = self.internal_get_staker(account_id);
        let votes = match staker.vote_positions.get(&contract_address) {
            Some(votes_for_address) => votes_for_address.get(&votable_object_id).unwrap_or(0_u128),
            None => 0_u128,
        };
        U128::from(votes)
    }

    // Get current NEAR ready for distribution.
    pub fn get_total_unclaimed_meta(&self) -> U128 {
        U128::from(self.total_unclaimed_near)
    }

    // pub fn get_accumulated_distributed_for_claims(&self) -> U128 {
    //     // TODO!! <-----------------
    //     U128::from(0)

    //     // self..into()
    // }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;
