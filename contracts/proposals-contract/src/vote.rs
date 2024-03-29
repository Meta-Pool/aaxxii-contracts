use crate::types::{ProposalId, VoterId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

/// Vote Types
#[derive(Serialize, Deserialize, Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum VoteType {
    Against,
    For,
    Abstain,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteJson {
    pub proposal_id: ProposalId,
    pub voter_id: VoterId,
    pub vote_type: VoteType,
    pub voting_power: U128,
    pub memo: String,
    // pub already_withdrawn: bool
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Vote {
    pub proposal_id: ProposalId,
    pub vote_type: VoteType,
    pub voting_power: u128,
    pub memo: String,
    // pub already_withdrawn: bool
}

impl Vote {
    pub(crate) fn new(
        proposal_id: ProposalId,
        _vote_type: VoteType,
        _voting_power: u128,
        _memo: String,
    ) -> Self {
        Vote {
            proposal_id,
            vote_type: _vote_type,
            voting_power: _voting_power,
            memo: _memo,
            // already_withdrawn: false
        }
    }
    pub(crate) fn to_json(&self, voter_id: VoterId) -> VoteJson {
        VoteJson {
            proposal_id: self.proposal_id.clone(),
            voter_id: voter_id.clone(),
            vote_type: self.vote_type.clone(),
            voting_power: U128::from(self.voting_power),
            memo: self.memo.clone(),
            // already_withdrawn: self.already_withdrawn.clone()
        }
    }
}

// impl Deref for Vote {
//     type Target = ProposalId;

//     fn deref(&self) -> &Self::Target {
//         &self.proposal_id
//     }
// }
