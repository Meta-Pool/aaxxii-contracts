use crate::*;

impl StakingPositionContract {
    pub(crate) fn assert_only_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only the owner can call this function."
        );
    }

    pub(crate) fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "Minimum deposit amount is {} META.",
            self.min_deposit_amount
        );
    }

    /// Inner method to get or create a Voter.
    pub(crate) fn internal_get_staker(&self, account_id: AccountId) -> Staker {
        self.stakers.get(&account_id).unwrap_or(Staker::new(&account_id))
    }

    pub(crate) fn internal_get_staker_or_panic(&self) -> Staker {
        self.stakers.get(&env::predecessor_account_id()).expect("Invalid staker_id.")
    }

    fn internal_get_total_votes_for_address(
        &self,
        contract_address: &ContractAddress,
    ) -> UnorderedMap<VotableObjId, VotingPower> {
        self.votes
            .get(&contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::ContractVotes {
                hash_id: generate_hash_id(contract_address.to_string()),
            }))
    }

    pub(crate) fn internal_increase_total_votes(
        &mut self,
        voting_power: VotingPower,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let mut votes_for_address = self.internal_get_total_votes_for_address(&contract_address);
        let mut votes = votes_for_address.get(&votable_object_id).unwrap_or(0_u128);
        votes += voting_power;

        votes_for_address.insert(&votable_object_id, &votes);
        self.votes.insert(&contract_address, &votes_for_address);
    }

    pub(crate) fn internal_decrease_total_votes(
        &mut self,
        voting_power: VotingPower,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let mut votes_for_address = self.internal_get_total_votes_for_address(&contract_address);
        let mut votes = votes_for_address
            .get(&votable_object_id)
            .expect("Cannot decrease if the Contract Address has no Votable Object.");
        require!(votes >= voting_power, "Decreasing total is too large.");
        votes -= voting_power;

        if votes == 0 {
            votes_for_address.remove(&votable_object_id);
        } else {
            votes_for_address.insert(&votable_object_id, &votes);
        }

        if votes_for_address.is_empty() {
            self.votes.remove(&contract_address);
        } else {
            self.votes.insert(&contract_address, &votes_for_address);
        }
    }

    pub(crate) fn insert_new_ft(&mut self, token_address: &AccountId) {
        match self.claimable_ft.get(token_address) {
            None => self.claimable_ft.insert(
                token_address,
                &FtDetails::new(token_address)
            ),
            Some(_) => panic!("FT address already exists."),
        };
    }

    pub(crate) fn is_ft_available(&self, token_address: &AccountId) -> bool {
        match self.claimable_ft.get(token_address) {
            None => false,
            Some(_) => true,
        }
    }

    // ***************************
    // * Claimable Meta & stNear *
    // ***************************

    fn add_claimable(
        claimable_map: &mut UnorderedMap<VoterId, u128>,
        total_unclaimed: &mut u128,
        account: &AccountId,
        amount: u128
    ) {
        let existing_claimable_amount = claimable_map.get(account).unwrap_or_default();
        claimable_map.insert(account, &(existing_claimable_amount + amount));
        // keep contract total
        *total_unclaimed += amount;
    }

    fn remove_claimable(
        claimable_map: &mut UnorderedMap<VoterId, u128>,
        total_unclaimed: &mut u128,
        account: &AccountId,
        amount: u128,
        token: &str
    ) {
        let existing_claimable_amount = claimable_map.get(account).unwrap_or_default();
        assert!(
            existing_claimable_amount >= amount,
            "you don't have enough claimable {}",
            token
        );
        let after_remove = existing_claimable_amount - amount;
        if after_remove == 0 {
            // 0 means remove
            claimable_map.remove(account)
        } else {
            claimable_map.insert(account, &after_remove)
        };
        // keep contract total
        *total_unclaimed -= amount;
    }

    // TODO: this is all wrong. please do it right!!!!

    pub(crate) fn add_claimable_ft(
        &mut self,
        account: &AccountId,
        token_address: &AccountId,
        amount: u128
    ) {
        assert!(amount > 0);

        let mut details = self.claimable_ft.get(token_address)
            .expect("Invalid token address.");

        let existing_claimable_amount = details.owners.get(account).unwrap_or_default();
        details.owners.insert(account, &(existing_claimable_amount + amount));

        // keep contract total
        details.total_unclaimed_ft += amount;
        details.accum_ft_distributed_for_claims += amount;

        self.claimable_ft.insert(token_address, &details);
    }

    // pub(crate) fn add_claimable_stnear(&mut self, account: &AccountId, amount: u128) {
    //     assert!(amount > 0);
    //     Self::add_claimable(&mut self.claimable_near, &mut self.total_unclaimed_near, account, amount);
    // }

    pub(crate) fn remove_claimable_meta(&mut self, account: &AccountId, amount: u128) {
        Self::remove_claimable(&mut self.claimable_near, &mut self.total_unclaimed_near, account, amount, "META");
    }

    // pub(crate) fn remove_claimable_stnear(&mut self, account: &AccountId, amount: u128) {
    //     Self::remove_claimable(&mut self.claimable_near, &mut self.total_unclaimed_near, account, amount, "stNEAR");
    // }
}
