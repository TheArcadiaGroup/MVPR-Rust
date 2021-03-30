#![no_std]
extern crate alloc;
use crate::{
    custom_types::custom_types::{
        GovernanceProposalSerialized, ProposalSerialized, VotersSerialized, VotingSerialized,
    },
    error::*,
    proposal::{ProposalStatus, ProposalType},
    GovernanceProposal, Proposal,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    ops::Add,
};
use types::{account::AccountHash, bytesrepr::FromBytes, ContractHash, PublicKey, U256};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct VotingData {
    pub reputation_staked: U256,
    pub vote: bool,
    pub claimed: bool,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum VoteResult {
    InVote,
    FailCriteriaUnmet,
    Approved,
    Rejected,
    MemberQuorumUnmet,
    ReputationQuorumUnmet,
    PassThresholdUnmet,
    FailThresholdUnmet,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Voting {
    pub start_timestamp: u64,
    pub total_members: u64,
    pub total_staked_reputation: U256,
    pub proposal: Option<Proposal>,
    pub governance_proposal: Option<GovernanceProposal>,
    pub proposal_type: ProposalType,
    pub for_votes: U256,
    pub against_votes: U256,
    pub input_reputation: U256,
    pub for_voters: BTreeMap<AccountHash, VotingData>,
    pub against_voters: BTreeMap<AccountHash, VotingData>,
    pub result: VoteResult,
}

impl Voting {
    pub fn new(
        start_timestamp: u64,
        serialized_proposal: ProposalSerialized,
    ) -> Result<Voting, VotingEngineError> {
        Ok(Voting {
            start_timestamp,
            against_voters: BTreeMap::new(),
            for_voters: BTreeMap::new(),
            against_votes: U256::from(0),
            for_votes: U256::from(0),
            total_members: 0 as u64,
            total_staked_reputation: U256::from(0),
            result: VoteResult::InVote,
            proposal: Some(Proposal::deserialize(serialized_proposal)),
            input_reputation: U256::from(0),
            governance_proposal: None,
            proposal_type: ProposalType::Grant,
        })
    }
    pub fn new_governance(
        start_timestamp: u64,
        serialized_governance_proposal: GovernanceProposalSerialized,
    ) -> Result<Voting, VotingEngineError> {
        Ok(Voting {
            start_timestamp,
            against_voters: BTreeMap::new(),
            for_voters: BTreeMap::new(),
            against_votes: U256::from(0),
            for_votes: U256::from(0),
            total_members: 0 as u64,
            total_staked_reputation: U256::from(0),
            result: VoteResult::InVote,
            proposal: None,
            input_reputation: U256::from(0),
            governance_proposal: Some(GovernanceProposal::deserialize(
                serialized_governance_proposal,
            )),
            proposal_type: ProposalType::Governance,
        })
    }

    pub fn start_at(&self) -> u64 {
        self.start_timestamp
    }

    pub fn end_at(&self) -> u64 {
        if self.proposal_type == ProposalType::Grant {
            self.proposal.clone().unwrap().vote_configuration.timeout
        } else {
            self.governance_proposal
                .clone()
                .unwrap()
                .vote_configuration
                .timeout
        }
    }

    pub fn calculate_vote_outcome(
        &mut self,
        current_time: u64,
        reputation_allocation_ratio: u64,
    ) -> Result<VoteResult, VotingEngineError> {
        // Check that voting has ended
        let proposal: Proposal = self.proposal.clone().unwrap();
        let timeout: u64 = proposal.vote_configuration.timeout;
        if timeout > current_time {
            return Err(VotingEngineError::VotingOngoing);
        }
        if proposal.proposal_status != ProposalStatus::InFullVote {
            return Err(VotingEngineError::VotingNotOngoing);
        }
        if self.total_members < proposal.vote_configuration.member_quorum {
            self.result = VoteResult::MemberQuorumUnmet;
        }
        if self.total_staked_reputation < proposal.vote_configuration.reputation_quorum {
            self.result = VoteResult::ReputationQuorumUnmet;
        }
        let total_votes = self.for_votes + self.against_votes;
        if self.for_votes > self.against_votes {
            let percentage: U256 = (self.for_votes * 10000) / (U256::from(100) * (total_votes));
            if percentage <= proposal.vote_configuration.threshold.into() {
                self.result = VoteResult::PassThresholdUnmet;
            } else {
                // calculate input reputation
                let denominator =
                    (U256::from(10).pow(U256::from(12))) / reputation_allocation_ratio;
                let input_reputation =
                    (proposal.cost * U256::from(10).pow(U256::from(16))) / denominator; // 8 decimals
                self.input_reputation = input_reputation;
                self.result = VoteResult::Approved;
            }
        } else if self.against_votes >= self.for_votes {
            let percentage: U256 = (self.against_votes * 10000) / (U256::from(100) * (total_votes));
            if percentage <= proposal.vote_configuration.threshold.into() {
                self.result = VoteResult::FailThresholdUnmet;
            } else {
                self.result = VoteResult::Rejected;
            }
        }
        Ok(self.result)
    }
    pub fn calculate_governance_vote_outcome(
        &mut self,
        current_time: u64,
    ) -> Result<(VoteResult, (bool, (String, String))), VotingEngineError> {
        // Check that voting has ended
        let proposal: GovernanceProposal = self.governance_proposal.clone().unwrap();
        let timeout: u64 = proposal.vote_configuration.timeout;
        let mut executed: bool = false;
        if timeout > current_time {
            return Err(VotingEngineError::VotingOngoing);
        }
        if proposal.proposal_status != ProposalStatus::InFullVote {
            return Err(VotingEngineError::VotingNotOngoing);
        }
        if self.total_staked_reputation < proposal.vote_configuration.full_vote_quorum {
            self.result = VoteResult::ReputationQuorumUnmet;
        }
        let total_votes = self.for_votes + self.against_votes;
        if self.for_votes > self.against_votes {
            let percentage: U256 = (self.for_votes * 10000) / (U256::from(100) * (total_votes));
            if percentage <= proposal.vote_configuration.full_vote_threshold.into() {
                self.result = VoteResult::PassThresholdUnmet;
            } else {
                // Vote passed
                if self.result == VoteResult::Approved {
                    executed = true;
                }
                self.result = VoteResult::Approved;
            }
        } else if self.against_votes >= self.for_votes {
            let percentage: U256 = (self.against_votes * 10000) / (U256::from(100) * (total_votes));
            if percentage <= proposal.vote_configuration.full_vote_threshold.into() {
                self.result = VoteResult::FailThresholdUnmet;
            } else {
                self.result = VoteResult::Rejected;
            }
        }
        Ok((self.result, (executed, proposal.new_variable_key_value)))
    }

    pub fn claim_reputation(&mut self, caller: AccountHash) -> Result<U256, VotingEngineError> {
        if self.result != VoteResult::Approved || self.result != VoteResult::Rejected {
            return Err(VotingEngineError::VoteFailed);
        }
        let is_for_voter = self.for_voters.contains_key(&caller);
        let is_against_voter = self.against_voters.contains_key(&caller);
        let mut caller_voting_data: Option<&mut VotingData> = None;
        if is_against_voter {
            caller_voting_data = self.against_voters.get_mut(&caller);
        } else if is_for_voter {
            caller_voting_data = self.for_voters.get_mut(&caller);
        }
        match caller_voting_data {
            Some(voting_data) => {
                if voting_data.claimed {
                    return Err(VotingEngineError::ReputationAlreadyClaimed);
                }
                voting_data.claimed = true;
                let mut similar_votes = self.for_votes;
                let mut opposite_votes = self.against_votes;
                let vote_rejected: bool = self.result == VoteResult::Rejected;
                if vote_rejected {
                    similar_votes = self.against_votes;
                    opposite_votes = self.for_votes;
                }
                let voter_shares =
                    (voting_data.reputation_staked * 1000) / (U256::from(10) * (similar_votes));
                let mut rep_gained: U256 = (voter_shares * (opposite_votes)) / 100;
                if self.proposal_type == ProposalType::Grant {
                    if !vote_rejected {
                        // If vote was approved, distribute input reputation
                        let nominator = self.input_reputation;
                        if caller == self.proposal.clone().unwrap().proposer {
                            // Give OP 1-Policing Ratio rep

                            // Op ratio = 1-Policing Ratio
                            let op_ratio = U256::from(10000)
                                - U256::from(
                                    self.proposal.clone().unwrap().ratios.policing_ratio * 100,
                                );
                            let denominator: U256 = (U256::from(10).pow(U256::from(12)))
                                / (U256::from(10000) / op_ratio);
                            rep_gained += nominator / denominator;
                        } else {
                            // NOT OP, gets pro rata policing ratio
                            let denominator: U256 = (U256::from(10).pow(U256::from(12)))
                                / (U256::from(10000)
                                    / self.proposal.clone().unwrap().ratios.policing_ratio
                                    * 100);
                            rep_gained = (nominator / denominator)
                                / (U256::from(10000) / (voter_shares * 100));
                        }
                    }
                }
                Ok(rep_gained + voting_data.reputation_staked)
            }
            None => Err(VotingEngineError::NoReputationToClaim),
        }
    }

    // If vote fails, users call this function to get their stakes back
    pub fn get_stake(&mut self, caller: AccountHash) -> Result<U256, VotingEngineError> {
        if self.result != VoteResult::FailCriteriaUnmet
            || self.result != VoteResult::MemberQuorumUnmet
            || self.result != VoteResult::FailThresholdUnmet
            || self.result != VoteResult::ReputationQuorumUnmet
        {
            return Err(VotingEngineError::VoteDidNotFail);
        }
        let is_for_voter = self.for_voters.contains_key(&caller);
        let is_against_voter = self.against_voters.contains_key(&caller);
        let mut caller_voting_data: Option<&mut VotingData> = None;
        if is_against_voter {
            caller_voting_data = self.against_voters.get_mut(&caller);
        } else if is_for_voter {
            caller_voting_data = self.for_voters.get_mut(&caller);
        }
        match caller_voting_data {
            Some(voting_data) => {
                if voting_data.claimed {
                    return Err(VotingEngineError::ReputationAlreadyClaimed);
                }
                voting_data.claimed = true;
                Ok(voting_data.reputation_staked)
            }
            None => Err(VotingEngineError::NoReputationToClaim),
        }
    }

    pub fn cast_vote(
        &mut self,
        caller: AccountHash,
        current_time: u64,
        reputation_balance: U256,
        reputation_to_stake: U256,
        committed_reputation: U256,
        vote_direction: bool,
    ) -> Result<(), VotingEngineError> {
        let is_for_voter = self.for_voters.contains_key(&caller);
        let is_against_voter = self.against_voters.contains_key(&caller);
        if is_against_voter || is_for_voter {
            return Err(VotingEngineError::AlreadyVoted);
        }
        if self.proposal_type == ProposalType::Grant {
            if current_time > self.proposal.clone().unwrap().vote_configuration.timeout {
                return Err(VotingEngineError::VotingEnded);
            }
            if self.proposal.clone().unwrap().proposal_status != ProposalStatus::InFullVote {
                return Err(VotingEngineError::VotingNotOngoing);
            }
        } else {
            if current_time
                > self
                    .governance_proposal
                    .clone()
                    .unwrap()
                    .vote_configuration
                    .timeout
            {
                return Err(VotingEngineError::VotingEnded);
            }
            if self.proposal.clone().unwrap().proposal_status != ProposalStatus::InFullVote {
                return Err(VotingEngineError::VotingNotOngoing);
            }
        }

        if reputation_to_stake > reputation_balance - committed_reputation {
            return Err(VotingEngineError::InvalidReputationToStake);
        }
        if self.proposal_type == ProposalType::Grant {
            let voter_staking_limit: U256 = self
                .proposal
                .clone()
                .unwrap()
                .vote_configuration
                .voter_staking_limits;
            let staking_percentage: U256 = reputation_balance / reputation_balance;
            if staking_percentage > voter_staking_limit {
                return Err(VotingEngineError::StakingLimitReached);
            }
        }

        let voting_data: VotingData = VotingData {
            claimed: false,
            reputation_staked: reputation_to_stake,
            vote: vote_direction,
        };
        if vote_direction {
            self.for_voters.insert(caller, voting_data);
            self.for_votes += reputation_to_stake;
        } else {
            self.against_voters.insert(caller, voting_data);
            self.against_votes += reputation_to_stake;
        }
        if !self.for_voters.contains_key(&caller) && !self.against_voters.contains_key(&caller) {
            // First time voting on this proposal
            self.total_members += 1;
        }
        self.total_staked_reputation += reputation_to_stake;
        Ok(())
    }

    pub fn serialize(&self) -> VotingSerialized {
        let mut proposal_option: Option<ProposalSerialized>;
        let mut governance_proposal_option: Option<GovernanceProposalSerialized>;
        if (self.proposal_type == ProposalType::Grant) {
            proposal_option = Some(Proposal::serialize(&self.proposal.as_ref().unwrap()));
            governance_proposal_option = None;
        } else {
            proposal_option = None;
            governance_proposal_option = Some(GovernanceProposal::serialize(
                &self.governance_proposal.as_ref().unwrap(),
            ));
        }
        (
            (
                (
                    self.start_timestamp,
                    self.total_members,
                    self.total_staked_reputation,
                ),
                (
                    proposal_option,
                    governance_proposal_option,
                    self.proposal_type as u8,
                ),
            ),
            (self.for_votes, self.against_votes, self.input_reputation),
            (
                self.serialize_voters().0,
                self.serialize_voters().1,
                self.result as u8,
            ),
        )
    }

    fn serialize_voters(&self) -> (VotersSerialized, VotersSerialized) {
        let mut for_voters_output = BTreeMap::new();
        for (key, voting_data) in self.for_voters.iter() {
            for_voters_output.insert(
                key.value(),
                (
                    voting_data.reputation_staked,
                    voting_data.vote,
                    voting_data.claimed,
                ),
            );
        }
        let mut against_voters_output = BTreeMap::new();
        for (key, voting_data) in self.against_voters.iter() {
            against_voters_output.insert(
                key.value(),
                (
                    voting_data.reputation_staked,
                    voting_data.vote,
                    voting_data.claimed,
                ),
            );
        }
        (for_voters_output, against_voters_output)
    }

    pub fn deserialize(serialized_voting: VotingSerialized) -> Voting {
        let mut proposal_type: ProposalType = ProposalType::Grant;
        let mut proposal: Option<Proposal>;
        let mut governance_proposal: Option<GovernanceProposal>;
        if serialized_voting.0 .1 .2 == 1 {
            proposal_type = ProposalType::Governance;
            proposal = None;
            governance_proposal = Some(GovernanceProposal::deserialize(
                serialized_voting.0 .1 .1.unwrap(),
            ))
        } else {
            // is grant
            proposal = Some(Proposal::deserialize(serialized_voting.0 .1 .0.unwrap()));
            governance_proposal = None;
        }
        Voting {
            start_timestamp: serialized_voting.0 .0 .0,
            total_members: serialized_voting.0 .0 .1,
            total_staked_reputation: serialized_voting.0 .0 .2,
            proposal: proposal,
            governance_proposal: governance_proposal,
            proposal_type: proposal_type,
            for_votes: serialized_voting.1 .0,
            against_votes: serialized_voting.1 .1,
            input_reputation: serialized_voting.1 .2,
            for_voters: Voting::deserialize_voters(serialized_voting.2 .0),
            against_voters: Voting::deserialize_voters(serialized_voting.2 .1),
            result: serialized_voting.2 .2.into(),
        }
    }

    fn deserialize_voters(
        voters_serialized: VotersSerialized,
    ) -> BTreeMap<AccountHash, VotingData> {
        let mut deserialized_voters: BTreeMap<AccountHash, VotingData> = BTreeMap::new();
        for (key, voting_data) in voters_serialized {
            deserialized_voters.insert(
                AccountHash::new(key),
                VotingData {
                    reputation_staked: voting_data.0,
                    vote: voting_data.1,
                    claimed: voting_data.2,
                },
            );
        }
        deserialized_voters
    }

    // #[cfg(test)]
    // mod tests {
    //     use super::*;
    //     use alloc::string::ToString;

    //     pub const ALI: AccountHash = AccountHash::new([1u8; 32]);
    //     pub const BOB: AccountHash = AccountHash::new([2u8; 32]);
    //     pub const JON: AccountHash = AccountHash::new([3u8; 32]);

    //     fn example_project(name: &str) -> Project {
    //         Project {
    //             name: name.to_string(),
    //             team_name: "casperlabs".to_string(),
    //             video_link: "https://www.youtube.com/channel/UCjFz9Sfi4yFwocnDQTWDSqA".to_string(),
    //             github_link: "https://github.com/CasperLabs/CasperLabs".to_string(),
    //             google_drive_link: "http://drive.google.com".to_string(),
    //         }
    //     }

    //     #[test]
    //     fn test_serialization() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let project_id = ProjectId(1);
    //         let project = example_project("project");
    //         let ali_power = 10;
    //         voting.add_or_update_project(project_id, project);
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();
    //         voting.cast_vote(ALI, project_id, ali_power, 1).unwrap();

    //         let serialized = voting.serialize();
    //         let deserialized = Voting::deserialize(serialized);
    //         assert_eq!(voting, deserialized);
    //     }

    //     #[test]
    //     fn test_init() {
    //         let start = 10;
    //         assert!(Voting::new(start, start).is_err());
    //         assert!(Voting::new(start, start - 1).is_err());
    //         let voting = Voting::new(start, start + 1).unwrap();
    //         assert_eq!(voting.start_at(), start);
    //         assert_eq!(voting.end_at(), start + 1);
    //     }

    //     #[test]
    //     fn test_add_or_update_participant() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let ali_power = 10;

    //         // Add participant.
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: 0,
    //                 votes: BTreeMap::new()
    //             }
    //         );

    //         // Update particpant.
    //         let updated_ali_power = 11;
    //         voting
    //             .add_or_update_participant(ALI, updated_ali_power)
    //             .unwrap();
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: updated_ali_power,
    //                 used_voting_power: 0,
    //                 votes: BTreeMap::new()
    //             }
    //         );
    //     }

    //     #[test]
    //     fn test_update_participant_with_vote() {
    //         let mut voting = Voting::new(1, 2).unwrap();

    //         // Add participant.
    //         let ali_power = 10;
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();

    //         // Add project.
    //         let project_id = ProjectId(1);
    //         let project = example_project("project");
    //         voting.add_or_update_project(project_id, project);

    //         // Make a vote.
    //         let vote = 5;
    //         voting.cast_vote(ALI, project_id, vote, 1).unwrap();

    //         // Update voting power to the currently used voting power.
    //         voting.add_or_update_participant(ALI, vote).unwrap();

    //         // Assert participant.
    //         let mut votes = BTreeMap::new();
    //         votes.insert(project_id, vote);
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: vote,
    //                 used_voting_power: vote,
    //                 votes: votes
    //             }
    //         );

    //         // Check if it's not possible to update below the used voting power.
    //         assert_eq!(
    //             voting.add_or_update_participant(ALI, vote - 1).unwrap_err(),
    //             NewVotingPowerBelowUsed
    //         );
    //     }

    //     #[test]
    //     fn test_remove_participant() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let ali_power = 10;
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();
    //         voting.remove_participant_if_exists(&ALI);
    //         assert!(voting.participants.get(&ALI).is_none());
    //     }

    //     #[test]
    //     fn test_add_and_update_project() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let project_id = ProjectId(1);
    //         let project = example_project("project");

    //         // Add new project.
    //         voting.add_or_update_project(project_id, project.clone());
    //         assert_eq!(voting.projects.get(&project_id).unwrap(), &project);

    //         // Update the project.
    //         let updated_project = example_project("project2");
    //         voting.add_or_update_project(project_id, updated_project.clone());
    //         assert_eq!(voting.projects.get(&project_id).unwrap(), &updated_project);
    //     }

    //     #[test]
    //     fn test_remove_project() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let project_id = ProjectId(1);
    //         let project = example_project("project");
    //         voting.add_or_update_project(project_id, project);
    //         voting.remove_project_if_exists_and_cancel_votes(project_id);
    //         assert!(voting.projects.get(&project_id).is_none());
    //     }

    //     #[test]
    //     fn test_voting() {
    //         let mut voting = Voting::new(1, 2).unwrap();

    //         // Setup projects.
    //         let a_project_id = ProjectId(1);
    //         let a_project = example_project("a_project");
    //         let b_project_id = ProjectId(2);
    //         let b_project = example_project("b_project");
    //         voting.add_or_update_project(a_project_id, a_project);
    //         voting.add_or_update_project(b_project_id, b_project);

    //         // Setup participants.
    //         let ali_power = 3;
    //         let bob_power = 5;
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();
    //         voting.add_or_update_participant(BOB, bob_power).unwrap();

    //         // Cannot vote before voting starts.
    //         let vote_at = 0;
    //         assert_eq!(
    //             voting.cast_vote(ALI, a_project_id, 1, vote_at).unwrap_err(),
    //             VotingEngineError::VotingNotStarted
    //         );

    //         // Cannot vote after voting ends.
    //         let vote_at = 2;
    //         assert_eq!(
    //             voting.cast_vote(ALI, a_project_id, 1, vote_at).unwrap_err(),
    //             VotingEngineError::VotingEnded
    //         );

    //         // Cannot vote more then voting power.
    //         let vote_at = 1;
    //         assert_eq!(
    //             voting
    //                 .cast_vote(ALI, b_project_id, ali_power + 1, vote_at)
    //                 .unwrap_err(),
    //             VotingEngineError::NotEnoughVotingPower
    //         );

    //         // Cannot vote on non exisitng projects.
    //         let unknown_project_id = ProjectId(3);
    //         assert_eq!(
    //             voting
    //                 .cast_vote(ALI, unknown_project_id, ali_power, vote_at)
    //                 .unwrap_err(),
    //             VotingEngineError::ProjectDoesNotExists
    //         );

    //         // Cannot vote as non participant.
    //         assert_eq!(
    //             voting
    //                 .cast_vote(JON, a_project_id, ali_power, vote_at)
    //                 .unwrap_err(),
    //             VotingEngineError::NotAParticipant
    //         );

    //         // Vote as Ali.
    //         let ali_a_project_vote = 2;
    //         let ali_b_project_vote = 1;
    //         voting
    //             .cast_vote(ALI, a_project_id, ali_a_project_vote, vote_at)
    //             .unwrap();
    //         voting
    //             .cast_vote(ALI, b_project_id, ali_b_project_vote, vote_at)
    //             .unwrap();
    //         let mut votes = BTreeMap::new();
    //         votes.insert(a_project_id, ali_a_project_vote);
    //         votes.insert(b_project_id, ali_b_project_vote);
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: ali_a_project_vote + ali_b_project_vote,
    //                 votes: votes
    //             }
    //         );

    //         // Vote as Bob.
    //         let bob_a_project_vote = 1;
    //         let bob_b_project_vote = 2;
    //         voting
    //             .cast_vote(BOB, a_project_id, bob_a_project_vote, vote_at)
    //             .unwrap();
    //         voting
    //             .cast_vote(BOB, b_project_id, bob_b_project_vote, vote_at)
    //             .unwrap();
    //         let mut votes = BTreeMap::new();
    //         votes.insert(a_project_id, bob_a_project_vote);
    //         votes.insert(b_project_id, bob_b_project_vote);
    //         assert_eq!(
    //             voting.participants.get(&BOB).unwrap(),
    //             &Participant {
    //                 total_voting_power: bob_power,
    //                 used_voting_power: bob_a_project_vote + bob_b_project_vote,
    //                 votes: votes
    //             }
    //         );

    //         // Removing project should remove votes.
    //         voting.remove_project_if_exists_and_cancel_votes(b_project_id);
    //         let mut votes = BTreeMap::new();
    //         votes.insert(a_project_id, ali_a_project_vote);
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: ali_a_project_vote,
    //                 votes: votes
    //             }
    //         );
    //         let mut votes = BTreeMap::new();
    //         votes.insert(a_project_id, bob_a_project_vote);
    //         assert_eq!(
    //             voting.participants.get(&BOB).unwrap(),
    //             &Participant {
    //                 total_voting_power: bob_power,
    //                 used_voting_power: bob_a_project_vote,
    //                 votes: votes
    //             }
    //         );
    //     }

    //     #[test]
    //     fn test_voting_update() {
    //         let mut voting = Voting::new(1, 2).unwrap();

    //         // Add new project.
    //         let project_id = ProjectId(1);
    //         let project = example_project("project");
    //         voting.add_or_update_project(project_id, project.clone());
    //         assert_eq!(voting.projects.get(&project_id).unwrap(), &project);

    //         // Add participant.
    //         let ali_power = 10;
    //         voting.add_or_update_participant(ALI, ali_power).unwrap();

    //         // Make the first vote.
    //         let first_vote = 5;
    //         voting.cast_vote(ALI, project_id, first_vote, 1).unwrap();

    //         // Assert the first vote.
    //         let mut votes = BTreeMap::new();
    //         votes.insert(project_id, first_vote);
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: first_vote,
    //                 votes: votes
    //             }
    //         );

    //         // Make the second vote.
    //         let second_vote = 3;
    //         voting.cast_vote(ALI, project_id, second_vote, 1).unwrap();

    //         // Assert the second vote.
    //         let mut votes = BTreeMap::new();
    //         votes.insert(project_id, second_vote);
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: second_vote,
    //                 votes: votes
    //             }
    //         );

    //         // Remove vote if the voting power is 0.
    //         voting.cast_vote(ALI, project_id, 0, 1).unwrap();
    //         assert_eq!(
    //             voting.participants.get(&ALI).unwrap(),
    //             &Participant {
    //                 total_voting_power: ali_power,
    //                 used_voting_power: 0,
    //                 votes: BTreeMap::new()
    //             }
    //         );
    //     }

    //     #[test]
    //     fn test_update_dates() {
    //         let mut voting = Voting::new(1, 2).unwrap();
    //         let new_start = 5;
    //         let new_end = 10;

    //         // Test errors.
    //         let result = voting.update_dates(new_end, new_start);
    //         assert_eq!(result, Err(StartNotBeforeEnd));

    //         // Test updates.
    //         let result = voting.update_dates(new_start, new_end);
    //         assert!(result.is_ok());
    //         assert_eq!(voting.start_at(), new_start);
    //         assert_eq!(voting.end_at(), new_end);
    //     }
}

impl From<u8> for VoteResult {
    fn from(orig: u8) -> Self {
        match orig {
            0 => return VoteResult::InVote,
            1 => return VoteResult::FailCriteriaUnmet,
            2 => return VoteResult::Approved,
            3 => return VoteResult::Rejected,
            4 => return VoteResult::MemberQuorumUnmet,
            5 => return VoteResult::ReputationQuorumUnmet,
            6 => return VoteResult::PassThresholdUnmet,
            7 => return VoteResult::FailThresholdUnmet,
            _ => return VoteResult::InVote,
        };
    }
}
