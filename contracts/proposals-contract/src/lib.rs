use crate::constants::*;
use crate::interface::*;
use proposals::{Proposal, ProposalJSON, ProposalState};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{unordered_map::UnorderedMap, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::{env, log, near_bindgen, require, AccountId, Balance, PanicOnDefault, Promise};
use types::*;
use utils::get_current_epoch_millis;
use vote::{Vote, VoteJson, VoteType};
use vote_counting::{ProposalVote, ProposalVoteJson};
use voter::{Voter, VoterJson};

mod constants;
mod interface;
mod internal;
mod proposals;
mod types;
mod utils;
mod vote;
mod vote_counting;
mod voter;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ProposalsContract {
    pub admin_id: AccountId,
    pub operator_ids: LookupSet<AccountId>,
    pub asset_token_contract_address: ContractAddress,
    pub staking_position_contract_address: ContractAddress,
    pub proposals: UnorderedMap<ProposalId, Proposal>,
    pub votes: UnorderedMap<ProposalId, ProposalVote>,
    pub voters: UnorderedMap<AccountId, Voter>,
    pub proposers: UnorderedMap<AccountId, Vec<ProposalId>>,
    /// Duration of the voting period.
    pub voting_period: EpochMillis,

    /// Parameters to allow an Account to create a new Proposal. (proposal threshold)
    pub min_asset_token_amount: Balance,
    pub min_st_near_amount: Balance,
    pub min_voting_power_amount: VotingPower,

    /// Cost of committing a new proposal. Base Token is burned. Near is for storage.
    pub proposal_cost_in_asset_token: Balance,
    pub proposal_storage_near: Balance,

    /// The creation of new Proposals could be stopped.
    pub open_for_new_proposals: bool,

    /// Minimum number of `asset token` circulating supply required for a governing body to approve a proposal.
    /// If a quorum is set to 50%, this means that 50% of all circulating tokens need to vote yes for the proposal to pass.
    /// Percent is denominated in basis points 100% equals 10_000 basis points.
    pub quorum_floor: BasisPoints,
}

#[near_bindgen]
impl ProposalsContract {
    #[init]
    pub fn new(
        admin_id: AccountId,
        operator_ids: Vec<AccountId>,
        asset_token_contract_address: ContractAddress,
        staking_position_contract_address: ContractAddress,
        voting_period: U64,
        min_voting_power_amount: U128,
        proposal_storage_near: U128,
        quorum_floor: BasisPoints,
    ) -> Self {
        require!(!env::state_exists(), "The contract is already initialized");
        require!(
            quorum_floor <= ONE_HUNDRED,
            "Incorrect quorum basis points."
        );

        let mut contract = Self {
            admin_id,
            operator_ids: LookupSet::new(StorageKey::Operators),
            asset_token_contract_address,
            staking_position_contract_address,
            proposals: UnorderedMap::new(StorageKey::Proposals),
            voting_period: voting_period.0,
            min_asset_token_amount: 0,
            min_st_near_amount: 0,
            min_voting_power_amount: min_voting_power_amount.0,
            proposal_cost_in_asset_token: 0,
            proposal_storage_near: proposal_storage_near.0,
            open_for_new_proposals: true,
            quorum_floor,
            votes: UnorderedMap::new(StorageKey::ProposalVotes),
            voters: UnorderedMap::new(StorageKey::Voters),
            proposers: UnorderedMap::new(StorageKey::Proposers),
        };

        for operator in operator_ids {
            if !contract.operator_ids.insert(&operator) {
                panic!("Duplicated account ids in operator list.")
            }
        }

        contract
    }

    // *********
    // * Admin *
    // *********

    /// Stop/Re-activate the submission of new proposals.
    pub fn update_open_for_new_proposals(&mut self, new_value: bool) {
        self.assert_only_admin();
        self.open_for_new_proposals = new_value;
    }

    /// Replace an operator role with a new account.
    pub fn replace_operator_role(&mut self, old_value: AccountId, new_value: AccountId) {
        self.assert_only_admin();
        if self.operator_ids.remove(&old_value) {
            self.operator_ids.insert(&new_value);
        }
    }

    /// Remove an operator role.
    pub fn remove_operator_role(&mut self, account: AccountId) {
        self.assert_only_admin();
        if !self.operator_ids.remove(&account) {
            panic!("Account is not operator.");
        }
    }

    /// Insert a new operator role.
    pub fn insert_operator_role(&mut self, account: AccountId) {
        self.assert_only_admin();
        if !self.operator_ids.insert(&account) {
            panic!("Account is already an operator.");
        }
    }

    /// Update the Admin role.
    pub fn update_admin_role(&mut self, new_value: AccountId) {
        self.assert_only_admin();
        self.admin_id = new_value;
    }

    pub fn pay_to_account(&mut self, amount: U128, to: AccountId) -> Promise {
        self.assert_only_admin();
        Promise::new(to).transfer(amount.0)
    }

    // ************
    // * Operator *
    // ************

    /// Update the voting period duration in milliseconds.
    pub fn update_voting_period(&mut self, new_value: U64) {
        self.assert_only_operator();
        self.voting_period = new_value.0;
    }

    /// Update minimum asset amount to submit a proposal.
    pub fn update_min_asset_token_amount(&mut self, new_value: U128) {
        self.assert_only_operator();
        self.min_asset_token_amount = new_value.0;
    }

    /// Update minimum voting power to submit a proposal (proposal threshold).
    pub fn update_min_voting_power_amount(&mut self, new_value: U128) {
        self.assert_only_operator();
        self.min_voting_power_amount = new_value.0;
    }

    /// Update the storage cost in NEAR to submit a proposal.
    pub fn update_proposal_storage_near(&mut self, new_value: U128) {
        self.assert_only_operator();
        self.proposal_storage_near = new_value.0;
    }

    /// Update quorum floor: percent of all voting power need to vote yes for the proposal to pass.
    pub fn update_quorum_floor(&mut self, new_value: u16) {
        self.assert_only_operator();
        require!(new_value <= ONE_HUNDRED, "Incorrect quorum basis points.");

        self.quorum_floor = new_value;
    }

    // ************
    // *  *
    // ************

    pub fn start_voting_period(&mut self, proposal_id: ProposalId) {
        self.assert_only_operator_or_creator(proposal_id);
        self.assert_proposal_is_active_or_draft(proposal_id);
        ext_proposal_vote::ext(self.staking_position_contract_address.clone())
            .with_static_gas(GAS_FOR_GET_VOTING_POWER)
            .with_attached_deposit(1)
            .get_total_voting_power()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_VOTE)
                    .start_voting_period_callback(proposal_id),
            );
    }

    #[private]
    pub fn start_voting_period_callback(&mut self, proposal_id: ProposalId) {
        let total_voting_power = self.internal_get_total_voting_power_from_promise();
        let mut proposal = self.internal_get_proposal(&proposal_id);
        let now = get_current_epoch_millis();
        proposal.vote_start_timestamp = Some(now);
        proposal.vote_end_timestamp = Some(now + self.voting_period);
        proposal.draft = false;
        proposal.v_power_quorum_to_reach = Some(self.internal_get_quorum(total_voting_power));
        self.proposals.insert(&proposal_id, &proposal);
    }

    // *******************************
    // * Proposal creators functions *
    // *******************************

    #[payable]
    pub fn create_proposal(
        &mut self,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) {
        self.assert_open_for_new_proposals();
        // self.assert_proposal_storage_is_covered();
        self.assert_only_operator();
        ext_proposal_vote::ext(self.staking_position_contract_address.clone())
            .with_static_gas(GAS_FOR_GET_VOTING_POWER)
            .with_attached_deposit(1)
            .get_all_locking_positions(env::predecessor_account_id())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_VOTE)
                    .create_proposal_callback(title, short_description, body, data, extra),
            );
    }

    #[private]
    pub fn create_proposal_callback(
        &mut self,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) -> ProposalId {
        let total_v_power = self.internal_get_user_total_voting_power_from_promise();
        self.assert_proposal_threshold(total_v_power);
        let id = self.proposals.len() as ProposalId;
        self.internal_create_proposal(id, title, short_description, body, data, extra);
        id
    }

    pub fn cancel_proposal(&mut self, proposal_id: ProposalId) {
        self.assert_only_operator_or_creator(proposal_id);
        self.assert_proposal_is_active_or_draft(proposal_id);
        let mut proposal = self.internal_get_proposal(&proposal_id);
        proposal.canceled = true;
        self.proposals.insert(&proposal_id, &proposal);
    }

    pub fn update_proposal(
        &mut self,
        proposal_id: ProposalId,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) {
        self.assert_only_creator(proposal_id);
        self.assert_proposal_is_active_or_draft(proposal_id);
        let mut proposal = self.internal_get_proposal(&proposal_id);
        proposal.title = title;
        proposal.short_description = short_description;
        proposal.body = body;
        proposal.data = data;
        proposal.extra = extra;
        self.proposals.insert(&proposal_id, &proposal);
    }

    pub fn get_my_proposals(&self, proposer_id: AccountId) -> Vec<ProposalId> {
        self.internal_get_proposer(proposer_id)
    }

    // ******************
    // * View functions *
    // ******************

    /// Check proposal threshold:
    /// if account has the minimum Voting Power to participate in Governance.
    pub fn check_proposal_threshold(&self, voting_power: U128) -> bool {
        self.internal_check_proposal_threshold(voting_power.0)
    }

    pub fn get_proposal_storage_near(&self) -> U128 {
        U128::from(self.proposal_storage_near)
    }

    pub fn get_proposal_votes(&self, proposal_id: ProposalId) -> ProposalVoteJson {
        let proposal_vote = self.internal_get_proposal_vote(proposal_id);
        proposal_vote.to_json()
    }

    pub fn get_quorum_reached(&self, proposal_id: ProposalId) -> bool {
        self.assert_only_operator();

        self.internal_is_quorum_reached(proposal_id)
    }

    pub fn get_proposal_vote_succeeded(&self, proposal_id: ProposalId) -> bool {
        let proposal_vote = self.internal_get_proposal_vote(proposal_id);
        proposal_vote.for_votes > proposal_vote.against_votes
    }

    pub fn get_proposal_state(&self, proposal_id: ProposalId) -> ProposalState {
        self.internal_get_proposal_state(proposal_id)
    }

    pub fn get_proposal(&self, proposal_id: ProposalId) -> ProposalJSON {
        let proposal = self.internal_get_proposal(&proposal_id);
        proposal.to_json()
    }

    pub fn get_proposals(&self, from_index: u32, limit: u32) -> Option<Vec<ProposalJSON>> {
        let mut result = Vec::<ProposalJSON>::new();

        let keys = self.proposals.keys_as_vector();
        let proposals_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;

        if start >= proposals_len {
            return None;
        }
        for index in start..std::cmp::min(start + limit, proposals_len) {
            let proposal_id = keys.get(index).unwrap();
            let proposal = self.proposals.get(&proposal_id).unwrap();
            result.push(proposal.to_json());
        }
        Some(result)
    }

    pub fn get_user_proposals_ids(&self, proposer_id: AccountId) -> Vec<ProposalId> {
        self.proposers.get(&proposer_id).unwrap_or(Vec::new())
    }

    pub fn get_last_proposal_id(&self) -> Option<ProposalId> {
        if self.proposals.len() > 0 {
            Some(self.proposals.len() as ProposalId - 1)
        } else {
            None
        }
    }

    pub fn get_quorum_floor(&self) -> BasisPoints {
        self.quorum_floor
    }

    pub fn get_proposal_threshold(&self) -> U128 {
        U128::from(self.min_voting_power_amount)
    }

    pub fn get_total_voters(&self) -> String {
        self.voters.len().to_string()
    }

    pub fn get_proposal_is_active_or_draft(&self, proposal_id: ProposalId) -> bool {
        self.internal_proposal_is_active_or_draft(proposal_id)
    }

    // *******************
    // * Voter functions *
    // *******************

    pub fn vote_proposal(
        &mut self,
        proposal_id: ProposalId,
        vote: VoteType,
        memo: String,
    ) {
        self.assert_proposal_is_on_voting(&proposal_id);
        self.assert_has_not_voted(proposal_id, env::predecessor_account_id());
        ext_proposal_vote::ext(self.staking_position_contract_address.clone())
            .with_static_gas(GAS_FOR_GET_VOTING_POWER)
            .with_attached_deposit(1)
            .get_all_locking_positions(env::predecessor_account_id())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_VOTE)
                    .vote_proposal_callback(
                        proposal_id.clone(),
                        env::predecessor_account_id(),
                        vote,
                        memo,
                    ),
            );
    }

    #[private]
    pub fn vote_proposal_callback(
        &mut self,
        proposal_id: ProposalId,
        voter_id: AccountId,
        vote_type: VoteType,
        memo: String,
    ) {
        let total_v_power = self.internal_get_user_total_voting_power_from_promise();
        let mut voter = self.internal_get_voter(&voter_id);
        assert!(
            total_v_power > 0,
            "Not enough voting power to vote! You have {}",
            total_v_power
        );
        let mut proposal_vote = self.internal_get_proposal_vote(proposal_id);
        let vote_v_power = total_v_power;
        let vote = Vote::new(
            proposal_id.clone(),
            vote_type.clone(),
            vote_v_power.clone(),
            memo.clone(),
        );

        proposal_vote
            .has_voted
            .insert(&voter_id.clone(), &vote.clone());
        match vote_type {
            VoteType::For => {
                proposal_vote.for_votes += vote_v_power;
            }
            VoteType::Against => {
                proposal_vote.against_votes += vote_v_power;
            }
            VoteType::Abstain => {
                proposal_vote.abstain_votes += vote_v_power;
            }
        }
        self.votes.insert(&proposal_id.clone(), &proposal_vote);
        voter.votes.insert(&proposal_id.clone(), &vote.clone());
        self.voters.insert(&voter_id.clone(), &voter);
    }

    pub fn remove_vote_proposal(&mut self, proposal_id: ProposalId) {
        let voter_id = env::predecessor_account_id();
        self.assert_proposal_is_on_voting(&proposal_id);
        self.assert_has_voted(proposal_id, voter_id.clone());
        let mut proposal_vote = self.internal_get_proposal_vote(proposal_id);
        let user_vote = proposal_vote.has_voted.get(&voter_id).unwrap();
        let mut voter = self.internal_get_voter(&voter_id);

        match user_vote.vote_type {
            VoteType::For => {
                proposal_vote.for_votes -= user_vote.voting_power;
            }
            VoteType::Against => {
                proposal_vote.against_votes -= user_vote.voting_power;
            }
            VoteType::Abstain => {
                proposal_vote.abstain_votes -= user_vote.voting_power;
            }
        }
        proposal_vote.has_voted.remove(&voter_id);
        self.votes.insert(&proposal_id, &proposal_vote);
        voter.votes.remove(&proposal_id);

        if voter.votes.is_empty() {
            self.voters.remove(&voter_id);
        } else {
            self.voters.insert(&voter_id, &voter);
        }
    }

    pub fn has_voted(&self, voter_id: AccountId, proposal_id: ProposalId) -> bool {
        self.internal_has_voted(&proposal_id, &voter_id)
    }

    pub fn get_my_vote(&self, voter_id: VoterId, proposal_id: ProposalId) -> Option<VoteJson> {
        let has_voted = self.internal_has_voted(&proposal_id, &voter_id);
        match has_voted {
            true => Some(
                self.internal_get_voter_vote(&proposal_id, &voter_id)
                    .to_json(voter_id),
            ),
            false => None,
        }
    }

    pub fn get_voter(&self, voter_id: VoterId) -> VoterJson {
        let voter = self.internal_get_voter(&voter_id);
        voter.to_json(voter_id)
    }

    // *********
    // * BOT FUNCTIONS *
    // *********

    pub fn process_voting_status(&mut self, proposal_id: ProposalId) {
        self.assert_only_operator();
        if !self.internal_proposal_is_on_voting(&proposal_id) {
            return;
        }
        let mut proposal = self.internal_get_proposal(&proposal_id);
        if get_current_epoch_millis() >= proposal.vote_end_timestamp.unwrap() {
            if self.internal_is_quorum_reached(proposal_id) {
                // TODO EXECUTE
                proposal.executed = true;
                self.proposals.insert(&proposal_id, &proposal);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;
