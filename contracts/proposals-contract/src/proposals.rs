use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalState {
    Draft,  // proposer share the idea. Giving awareness from the community via discussion or poll
    Active, // reviewed and accepted by managers
    VotingProcess, // on voting process
    Accepted, // accepted by votes
    Rejected, // rejected by votes
    Executed, // proposal executed, performing on-chain actions
    Canceled, // canceled by manager after community awareness
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalJSON {
    pub proposal_id: ProposalId,
    pub title: String,
    pub short_description: String,
    pub body: String,
    pub data: String,
    pub extra: String,
    pub creator_id: AccountId,
    pub vote_start_timestamp: Option<EpochMillis>,
    pub vote_end_timestamp: Option<EpochMillis>,
    pub draft: bool,
    pub executed: bool,
    pub canceled: bool,
    pub v_power_quorum_to_reach: Option<U128>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub proposal_id: ProposalId,
    pub title: String,
    pub short_description: String,
    pub body: String,
    pub data: String,
    pub extra: String,
    pub creator_id: AccountId,
    pub vote_start_timestamp: Option<EpochMillis>,
    pub vote_end_timestamp: Option<EpochMillis>,
    pub draft: bool,
    pub executed: bool,
    pub canceled: bool,
    pub v_power_quorum_to_reach: Option<VotingPower>,
}

impl Proposal {
    pub(crate) fn new(
        id: ProposalId,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) -> Self {
        Proposal {
            proposal_id: id,
            title,
            short_description,
            body,
            data,
            extra,
            creator_id: env::signer_account_id(),
            vote_end_timestamp: None,
            vote_start_timestamp: None,
            draft: true,
            executed: false,
            canceled: false,
            v_power_quorum_to_reach: None,
        }
    }

    pub(crate) fn to_json(&self) -> ProposalJSON {
        let quorum_to_reach = match self.v_power_quorum_to_reach {
            Some(quorum_to_reach) => Some(U128::from(quorum_to_reach)),
            None => None,
        };

        ProposalJSON {
            proposal_id: self.proposal_id.clone(),
            title: self.title.clone(),
            body: self.body.clone(),
            short_description: self.short_description.clone(),
            data: self.data.clone(),
            extra: self.extra.clone(),
            creator_id: self.creator_id.clone(),
            vote_end_timestamp: self.vote_end_timestamp.clone(),
            vote_start_timestamp: self.vote_start_timestamp.clone(),
            executed: self.executed,
            canceled: self.canceled,
            draft: self.draft,
            v_power_quorum_to_reach: quorum_to_reach
        }
    }
}

impl ProposalsContract {
    pub(crate) fn internal_create_proposal(
        &mut self,
        proposal_id: ProposalId,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) -> ProposalId {
        let proposal = Proposal::new(proposal_id, title, short_description, body, data, extra);
        self.proposals.insert(&proposal_id, &proposal);
        let mut proposer = self.internal_get_proposer(proposal.creator_id.clone());
        proposer.push(proposal_id);
        self.proposers.insert(&proposal.creator_id, &proposer);
        // let voter = self.internal_get_voter(&proposal.creator_id);
        // voter.used_voting_power += self.min_voting_power_amount;
        // self.voters.insert(&proposal.creator_id, &voter);
        proposal.proposal_id.into()
    }

    pub(crate) fn internal_get_proposal_state(
        &self,
        proposal_id: ProposalId
    ) -> ProposalState {
        let proposal = self.internal_get_proposal(&proposal_id);
        if proposal.executed {
            return ProposalState::Executed;
        } else if proposal.canceled {
            return ProposalState::Canceled;
        } else if proposal.draft {
            return ProposalState::Draft;
        }

        if self.internal_proposal_is_on_voting(&proposal_id) {
            return ProposalState::VotingProcess;
        } else if self.internal_proposal_is_active(proposal_id) {
            return ProposalState::Active;
        }

        if self.internal_is_quorum_reached(proposal_id)
            && self.get_proposal_vote_succeeded(proposal_id)
        {
            return ProposalState::Accepted;
        } else {
            return ProposalState::Rejected;
        }
    }
}
