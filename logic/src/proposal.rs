#![no_std]
extern crate alloc;
use crate::error::*;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    ops::Add,
};
use types::{account::AccountHash, bytesrepr::FromBytes, PublicKey, U256};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FundingTranche {
    pub funding_tranche_type: u8,
    pub amount: U256,
    pub reputation_allocation: U256,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Milestone {
    pub milestone_type: u8,
    pub progress_percentage: u8,
    pub result: u8,
    pub funding_tranches: BTreeMap<u64, FundingTranche>,
    pub funding_tranches_size: u64,
    pub timeout: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Ratios {
    pub policing_ratio: u8,
    pub op_ratio: u8,
    pub citation_ratio: u8,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VoteConfiguration {
    pub member_quorum: u64,
    pub reputation_quorum: U256,
    pub threshold: u8,
    // How long does the vote remain active
    pub timeout: u64,
    pub voter_staking_limits: U256,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct GovernanceVoteConfiguration {
    pub transition_vote_quorum: U256,
    pub transition_vote_threshold: u64,
    pub proposal_repository_address: String,
    pub full_vote_quorum: U256,
    pub full_vote_threshold: u64,
    pub timeout: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Proposal {
    pub name: String,
    pub storage_pointer: String,
    pub storage_fingerprint: String,
    pub proposal_type: ProposalType,
    pub proposer: AccountHash,
    pub citations: Vec<u64>,
    pub ratios: Ratios,
    pub vote_configuration: VoteConfiguration,
    pub milestones: BTreeMap<u64, Milestone>,
    pub proposal_status: ProposalStatus,
    pub sponsors: BTreeMap<AccountHash, U256>,
    pub cost: U256,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct GovernanceProposal {
    pub name: String,
    pub repository_url: String,
    pub proposer: AccountHash,
    pub sponsors: BTreeMap<AccountHash, U256>,
    pub proposal_type: ProposalType,
    pub vote_configuration: GovernanceVoteConfiguration,
    pub proposal_status: ProposalStatus,
    pub new_variable_key_value: (String, String),
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ProposalType {
    Grant,
    Governance,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ProposalStatus {
    WaitingFullVote,
    InFullVote,
    FullVoteComplete,
}
type ProposalSerialized = (
    (
        // 0
        // name, storage_pointer, storage_fingerprint
        (String, String, String), // 0.0
        (
            // 0.1
            // proposal type
            u8,
            // proposer
            [u8; 32],
            // citations
            Vec<u64>,
        ),
        (
            // 0.2
            // ratios
            RatiosSerialized,
            // vote configuration
            VoteConfigurationSerialized,
            // milestones
            MilestoneSerialized,
        ),
    ),
    (
        // 1
        // Proposal status
        u8, // 0.0
        // Sponsors
        SponsorsSerialized, // 0.1
        // cost
        U256, //0.2
    ),
);

type MilestoneSerialized = BTreeMap<
    u64,
    (
        (u8, u8, u8),
        (BTreeMap<u64, FundingTrancheSerialized>, u64, u64),
    ),
>;

type SponsorsSerialized = BTreeMap<[u8; 32], U256>;

type FundingTrancheSerialized = (u8, U256, U256);

type RatiosSerialized = ([u8; 3]);
type VoteConfigurationSerialized = ((u64, U256), (u8, u64, U256));
// pub name: String,
//     pub repository_url: String,
//     pub proposer: AccountHash,
//     pub sponsors: BTreeMap<AccountHash, U256>,
//     pub proposal_type: ProposalType,
//     pub vote_configuration: GovernanceVoteConfiguration,
//     pub proposal_status: ProposalStatus,
//     pub new_variable_key_value: (String, String),
type GovernanceProposalSerialized = (
    // 0
    // name, storage_pointer, storage_fingerprint
    (String, String, [u8; 32]), // 0.0 // name, repo url, proposer
    (
        // 0.1
        // sponsors
        SponsorsSerialized,
        // proposal type
        u8,
        // vote config
        GovernanceVoteConfigurationSerialized,
    ),
    (
        // 0.2
        // proposal status
        u8,
        // new_variable_key_value
        (String, String),
    ),
);

type GovernanceVoteConfigurationSerialized = ((U256, u64, String), (U256, u64, u64));

impl Proposal {
    pub fn new(
        name: String,
        storage_pointer: String,
        storage_fingerprint: String,
        category: u8,
        citations: Vec<u64>,
        ratios: Vec<u8>,
        vote_configuration: VoteConfigurationSerialized,
        milestones: Vec<((u8, u8, Vec<(u8, U256, U256)>), u64)>,
        staked_rep: U256,
        proposer: AccountHash,
        system_policing_ratio: u8,
        reputation_balance: U256,
        sponsors: Vec<(AccountHash, U256)>,
        cost: U256,
    ) -> Result<Proposal, ProposalError> {
        let proposal_policing_ratio: u8 = *ratios.get(0).unwrap();
        // let proposal_citation_ratio: u8 = *ratios.get(1).unwrap();
        if proposal_policing_ratio < system_policing_ratio || proposal_policing_ratio > 100 {
            return Err(ProposalError::InvalidPolicingRatio);
        }
        // let op_ratio : u8 = 100-(proposal_policing_ratio+);
        if category > 1 {
            return Err(ProposalError::InvalidCategory);
        }

        if staked_rep > reputation_balance {
            return Err(ProposalError::StakedRepGreaterThanReputationBalance);
        }
        let mut new_category: ProposalType = ProposalType::Grant;
        if category == 1 {
            new_category = ProposalType::Grant;
        }

        // pub milestones: BTreeMap<u64, Milestone>
        let mut mstones: BTreeMap<u64, Milestone> = BTreeMap::new();
        // The milestones_sum must be equal to the cost
        let mut milestones_sum: U256 = U256::from(0);
        let mut progress_percentages_sum: u8 = 0;
        for (i, mstone) in milestones.iter().enumerate() {
            let milestone_type: u8 = mstone.0 .0;
            let progress_percentage: u8 = mstone.0 .1;
            progress_percentages_sum = progress_percentages_sum.add(progress_percentage);
            let (funding_tranches, milestone_amount) =
                Self::create_funding_tranches_mapping(&mstone.0 .2);
            milestones_sum = milestones_sum.add(milestone_amount);
            let funding_tranches_size = mstone.0 .2.len();
            mstones.insert(
                i as u64,
                Milestone {
                    progress_percentage,
                    funding_tranches,
                    funding_tranches_size: funding_tranches_size as u64,
                    milestone_type,
                    result: 2,
                    timeout: mstone.1,
                },
            );
        }
        if cost != milestones_sum {
            return Err(ProposalError::ProjectCostNotEqualToMilestonesSum);
        }
        if progress_percentages_sum != 100 {
            return Err(ProposalError::InvalidMilestonesProgressPercentages);
        }
        let mut sponsors_mapping: BTreeMap<AccountHash, U256> = BTreeMap::new();
        for sponsor in sponsors {
            sponsors_mapping.insert(sponsor.0, sponsor.1);
        }
        let member_quorum: u64 = vote_configuration.0 .0;
        let reputation_quorum: U256 = vote_configuration.0 .1;
        let threshold: u8 = vote_configuration.1 .0;
        let timeout: u64 = vote_configuration.1 .1;
        let voter_staking_limits: U256 = vote_configuration.1 .2;
        Ok(Proposal {
            name: name,
            storage_fingerprint: storage_fingerprint,
            storage_pointer: storage_pointer,
            vote_configuration: VoteConfiguration {
                member_quorum,
                reputation_quorum,
                voter_staking_limits,
                timeout,
                threshold,
            },
            proposal_type: new_category,
            proposer,
            citations: citations,
            milestones: mstones,
            ratios: Ratios {
                policing_ratio: ratios.get(0).unwrap().clone(),
                op_ratio: ratios.get(1).unwrap().clone(),
                citation_ratio: ratios.get(2).unwrap().clone(),
            },
            proposal_status: ProposalStatus::InFullVote,
            sponsors: sponsors_mapping,
            cost,
        })
    }

    fn create_funding_tranches_mapping(
        user_funding_tranches: &Vec<(u8, U256, U256)>,
    ) -> (BTreeMap<u64, FundingTranche>, U256) {
        let mut ftranches: BTreeMap<u64, FundingTranche> = BTreeMap::new();
        let mut milestones_sum: U256 = U256::from(0);
        for (j, funding_tranche) in user_funding_tranches.iter().enumerate() {
            let funding_tranche_type = funding_tranche.0;
            let funding_tranche_amount: U256 = funding_tranche.1;
            milestones_sum = milestones_sum.add(funding_tranche_amount);
            let funding_tranche_reputation_allocation = funding_tranche.2;
            ftranches.insert(
                j as u64,
                FundingTranche {
                    amount: funding_tranche_amount,
                    funding_tranche_type,
                    reputation_allocation: funding_tranche_reputation_allocation,
                },
            );
        }
        (ftranches, milestones_sum)
    }

    pub fn serialize(&self) -> ProposalSerialized {
        (
            (
                (
                    self.name.clone(),
                    self.storage_pointer.clone(),
                    self.storage_fingerprint.clone(),
                ),
                (
                    self.proposal_type as u8,
                    self.proposer.value(),
                    self.citations.clone(),
                ),
                (
                    self.serialize_ratios(),
                    self.serialize_vote_configuration(),
                    self.serialize_milestones(),
                ),
            ),
            (
                self.proposal_status as u8,
                self.serialize_sponsors(),
                self.cost,
            ),
        )
    }

    fn serialize_ratios(&self) -> RatiosSerialized {
        [
            self.ratios.policing_ratio,
            self.ratios.op_ratio,
            self.ratios.citation_ratio,
        ]
    }
    fn serialize_vote_configuration(&self) -> VoteConfigurationSerialized {
        (
            (
                self.vote_configuration.member_quorum,
                self.vote_configuration.reputation_quorum,
            ),
            (
                self.vote_configuration.threshold,
                self.vote_configuration.timeout,
                self.vote_configuration.voter_staking_limits,
            ),
        )
    }
    fn serialize_milestones(&self) -> MilestoneSerialized {
        let mut serialized_milestones: MilestoneSerialized = BTreeMap::new();
        for (key, milestone) in self.milestones.iter() {
            let mut serialized_funding_tranches: BTreeMap<u64, FundingTrancheSerialized> =
                BTreeMap::new();
            serialized_funding_tranches =
                Self::serialize_funding_tranches(&milestone.funding_tranches);
            serialized_milestones.insert(
                key.clone(),
                (
                    (
                        milestone.milestone_type,
                        milestone.progress_percentage,
                        milestone.result,
                    ),
                    (
                        serialized_funding_tranches,
                        milestone.funding_tranches_size,
                        milestone.timeout,
                    ),
                ),
            );
        }
        serialized_milestones
    }
    fn serialize_funding_tranches(
        funding_tranches: &BTreeMap<u64, FundingTranche>,
    ) -> BTreeMap<u64, FundingTrancheSerialized> {
        let mut output: BTreeMap<u64, FundingTrancheSerialized> = BTreeMap::new();
        for (key, tranche) in funding_tranches.iter() {
            output.insert(
                key.clone(),
                (
                    tranche.funding_tranche_type,
                    tranche.amount,
                    tranche.reputation_allocation,
                ),
            );
        }
        output
    }
    fn serialize_sponsors(&self) -> SponsorsSerialized {
        type SponsorsSerialized = BTreeMap<[u8; 32], U256>;

        let mut output: SponsorsSerialized = BTreeMap::new();
        for (key, sponsor) in self.sponsors.iter() {
            output.insert(key.value(), sponsor.clone());
        }
        output
    }
    pub fn deserialize(serialized_proposal: ProposalSerialized) -> Proposal {
        let mut proposal_type: ProposalType = ProposalType::Grant;
        if serialized_proposal.0 .1 .0 == 1 {
            proposal_type = ProposalType::Grant;
        }
        Proposal {
            name: serialized_proposal.0 .0 .0,
            storage_pointer: serialized_proposal.0 .0 .1,
            storage_fingerprint: serialized_proposal.0 .0 .2,
            proposal_type: proposal_type,
            proposer: AccountHash::new(serialized_proposal.0 .1 .1),
            citations: serialized_proposal.0 .1 .2,
            ratios: Ratios {
                policing_ratio: *serialized_proposal.0 .2 .0.get(0).unwrap(),
                op_ratio: *serialized_proposal.0 .2 .0.get(1).unwrap(),
                citation_ratio: *serialized_proposal.0 .2 .0.get(2).unwrap(),
            },
            vote_configuration: VoteConfiguration {
                member_quorum: serialized_proposal.0 .2 .1 .0 .0,
                reputation_quorum: serialized_proposal.0 .2 .1 .0 .1,
                threshold: serialized_proposal.0 .2 .1 .1 .0,
                timeout: serialized_proposal.0 .2 .1 .1 .1,
                voter_staking_limits: serialized_proposal.0 .2 .1 .1 .2,
            },
            milestones: Self::deserialize_milestones(serialized_proposal.0 .2 .2),
            proposal_status: serialized_proposal.1 .0.into(),
            sponsors: Self::deserialize_sponsors(serialized_proposal.1 .1),
            cost: serialized_proposal.1 .2,
        }
    }

    fn deserialize_milestones(
        serialized_milestones: MilestoneSerialized,
    ) -> BTreeMap<u64, Milestone> {
        let mut deserialized_milestones: BTreeMap<u64, Milestone> = BTreeMap::new();
        for (key, milestone) in serialized_milestones {
            deserialized_milestones.insert(
                key,
                Milestone {
                    milestone_type: milestone.0 .0,
                    progress_percentage: milestone.0 .1,
                    result: milestone.0 .2,
                    funding_tranches: Self::deserialize_funding_tranches(milestone.1 .0),
                    funding_tranches_size: milestone.1 .1,
                    timeout: milestone.1 .2,
                },
            );
        }
        deserialized_milestones
    }
    fn deserialize_funding_tranches(
        serialized_funding_tranches: BTreeMap<u64, FundingTrancheSerialized>,
    ) -> BTreeMap<u64, FundingTranche> {
        let mut deserialized_tranches: BTreeMap<u64, FundingTranche> = BTreeMap::new();
        for (key, tranche) in serialized_funding_tranches {
            deserialized_tranches.insert(
                key,
                FundingTranche {
                    funding_tranche_type: tranche.0,
                    amount: tranche.1,
                    reputation_allocation: tranche.2,
                },
            );
        }
        deserialized_tranches
    }
    fn deserialize_sponsors(
        serialized_sponsors: BTreeMap<[u8; 32], U256>,
    ) -> BTreeMap<AccountHash, U256> {
        let mut deserialized_sponsors: BTreeMap<AccountHash, U256> = BTreeMap::new();
        for (key, commitment) in serialized_sponsors {
            deserialized_sponsors.insert(AccountHash::new(key), commitment);
        }
        deserialized_sponsors
    }
}
impl GovernanceProposal {
    pub fn new(
        name: String,
        vote_configuration: GovernanceVoteConfigurationSerialized,
        staked_rep: U256,
        proposer: AccountHash,
        reputation_balance: U256,
        sponsors: Vec<(AccountHash, U256)>,
        repository_url: String,
        new_variable_key_value: (String, String),
    ) -> Result<GovernanceProposal, ProposalError> {
        if staked_rep > reputation_balance {
            return Err(ProposalError::StakedRepGreaterThanReputationBalance);
        }
        let proposal_type: ProposalType = ProposalType::Governance;

        let mut sponsors_mapping: BTreeMap<AccountHash, U256> = BTreeMap::new();
        for sponsor in sponsors {
            sponsors_mapping.insert(sponsor.0, sponsor.1);
        }
        Ok(GovernanceProposal {
            name,
            vote_configuration: GovernanceVoteConfiguration {
                transition_vote_quorum: vote_configuration.0 .0,
                transition_vote_threshold: vote_configuration.0 .1,
                proposal_repository_address: vote_configuration.0 .2,
                full_vote_quorum: vote_configuration.1 .0,
                full_vote_threshold: vote_configuration.1 .1,
                timeout: vote_configuration.1 .2,
            },
            proposal_type,
            proposer,
            proposal_status: ProposalStatus::InFullVote,
            sponsors: sponsors_mapping,
            repository_url,
            new_variable_key_value,
        })
    }

    pub fn serialize(&self) -> GovernanceProposalSerialized {
        // type GovernanceProposalSerialized = (
        //     // 0
        //     // name, storage_pointer, storage_fingerprint
        //     (String, String, [u8; 32]), // 0.0 // name, repo url, proposer
        //     (
        //         // 0.1
        //         // sponsors
        //         SponsorsSerialized,
        //         // proposal type
        //         u8,
        //         // vote config
        //         GovernanceVoteConfigurationSerialized,
        //     ),
        //     (
        //         // 0.2
        //         // proposal status
        //         u8,
        //         // new_variable_key_value
        //         (String, String),
        //     ),
        // );
        (
            (
                self.name.clone(),
                self.repository_url.clone(),
                self.proposer.value(),
            ),
            (
                self.serialize_sponsors(),
                self.proposal_type as u8,
                self.serialize_vote_configuration(),
            ),
            (
                self.proposal_status as u8,
                (
                    self.new_variable_key_value.0.clone(),
                    self.new_variable_key_value.1.clone(),
                ),
            ),
        )
    }

    fn serialize_vote_configuration(&self) -> GovernanceVoteConfigurationSerialized {
        (
            (
                self.vote_configuration.transition_vote_quorum,
                self.vote_configuration.transition_vote_threshold,
                self.vote_configuration.proposal_repository_address.clone(),
            ),
            (
                self.vote_configuration.full_vote_quorum,
                self.vote_configuration.full_vote_threshold,
                self.vote_configuration.timeout,
            ),
        )
    }

    fn serialize_sponsors(&self) -> SponsorsSerialized {
        type SponsorsSerialized = BTreeMap<[u8; 32], U256>;

        let mut output: SponsorsSerialized = BTreeMap::new();
        for (key, sponsor) in self.sponsors.iter() {
            output.insert(key.value(), sponsor.clone());
        }
        output
    }
    pub fn deserialize(
        serialized_governance_proposal: GovernanceProposalSerialized,
    ) -> GovernanceProposal {
        // type GovernanceProposalSerialized = (
        //     // 0
        //     // name, storage_pointer, storage_fingerprint
        //     (String, String, [u8; 32]), // 0.0 // name, repo url, proposer
        //     (
        //         // 0.1
        //         // sponsors
        //         SponsorsSerialized,
        //         // proposal type
        //         u8,
        //         // vote config
        //         GovernanceVoteConfigurationSerialized,
        //     ),
        //     (
        //         // 0.2
        //         // proposal status
        //         u8,
        //         // new_variable_key_value
        //         (String, String),
        //     ),
        // );
        GovernanceProposal {
            name: serialized_governance_proposal.0 .0,
            repository_url: serialized_governance_proposal.0 .1,
            proposer: AccountHash::new(serialized_governance_proposal.0 .2),
            sponsors: GovernanceProposal::deserialize_sponsors(serialized_governance_proposal.1 .0),
            proposal_type: ProposalType::Governance,
            vote_configuration: GovernanceVoteConfiguration {
                transition_vote_quorum: serialized_governance_proposal.1 .2 .0 .0,
                transition_vote_threshold: serialized_governance_proposal.1 .2 .0 .1,
                proposal_repository_address: serialized_governance_proposal.1 .2 .0 .2,
                full_vote_quorum: serialized_governance_proposal.1 .2 .1 .0,
                full_vote_threshold: serialized_governance_proposal.1 .2 .1 .1,
                timeout: serialized_governance_proposal.1 .2 .1 .2,
            },
            proposal_status: serialized_governance_proposal.2 .0.into(),
            new_variable_key_value: serialized_governance_proposal.2 .1,
        }
    }

    fn deserialize_sponsors(
        serialized_sponsors: BTreeMap<[u8; 32], U256>,
    ) -> BTreeMap<AccountHash, U256> {
        let mut deserialized_sponsors: BTreeMap<AccountHash, U256> = BTreeMap::new();
        for (key, commitment) in serialized_sponsors {
            deserialized_sponsors.insert(AccountHash::new(key), commitment);
        }
        deserialized_sponsors
    }
}

impl From<u8> for ProposalStatus {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return ProposalStatus::WaitingFullVote,
            0x1 => return ProposalStatus::InFullVote,
            0x2 => return ProposalStatus::FullVoteComplete,
            _ => return ProposalStatus::WaitingFullVote,
        };
    }
}
