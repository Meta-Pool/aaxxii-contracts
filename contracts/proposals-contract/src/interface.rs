use crate::types::ProposalId;
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_proposal_vote)]
pub trait ExtStakingPosition {
    // should call a fn to get all voting power (used or not) get_user_total_voting_power
    fn get_available_voting_power(&self, account_id: AccountId);
    fn get_all_locking_positions(&self, account_id: AccountId);
    fn get_total_voting_power(&self);
}

#[ext_contract(ext_self)]
pub trait SelfProposals {
    fn vote_proposal_callback(
        &mut self,
        proposal_id: ProposalId,
        voting_power: U128,
        voter_id: AccountId,
        vote_type: VoteType,
    );

    fn create_proposal_callback(
        &mut self,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    );
}
