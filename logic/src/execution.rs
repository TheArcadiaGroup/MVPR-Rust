#![no_std]
extern crate alloc;
use crate::{
    custom_types::custom_types::{
        FundingTrancheSerialized, MilestoneAnalysisSerialized, MilestoneSerialized,
        ProjectSerialized, ProposalSerialized,
    },
    error::*,
    proposal::{FundingTranche, Milestone},
    Proposal,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    ops::Add,
};
use types::{account::AccountHash, bytesrepr::FromBytes, PublicKey, RuntimeArgs, U256};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MilestoneAnalysis {
    pub is_favorable: bool,
    pub recommendations: BTreeMap<String, String>,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Project {
    pub proposal: Proposal,
    pub active_milestone: u64,
    pub status: ProjectStatus,
    pub claimed_milestones: Vec<Milestone>,
    pub milestone_analyses: BTreeMap<U256, MilestoneAnalysis>,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ProjectStatus {
    Active,
    MilestoneUnderReview,
    VotingOnMilestoneAnalysis,
    MilestoneTimeout,
    Completed,
    Remediation,
}
impl Project {
    pub fn new(serialized_proposal: ProposalSerialized) -> Result<Project, ProposalError> {
        Ok(Project {
            proposal: Proposal::deserialize(serialized_proposal),
            active_milestone: 0,
            status: ProjectStatus::Active,
            claimed_milestones: Vec::new(),
            milestone_analyses: BTreeMap::new(),
        })
    }

    pub fn check_milestone_timeout(&mut self, current_time: u64) -> Result<bool, ProposalError> {
        let current_milestone: Milestone = self
            .proposal
            .milestones
            .get(&self.active_milestone)
            .unwrap()
            .clone();
        if current_milestone.timeout < current_time
            && !self.claimed_milestones.contains(&current_milestone)
        {
            self.status = ProjectStatus::MilestoneTimeout;
            return Ok(true);
        }
        Ok(false)
    }
    pub fn claim_milestone(&mut self) -> Result<(), ProposalError> {
        let current_milestone: Milestone = self
            .proposal
            .milestones
            .get(&self.active_milestone)
            .unwrap()
            .clone();
        self.claimed_milestones.push(current_milestone.clone());
        self.status = ProjectStatus::MilestoneUnderReview;
        Ok(())
    }
    pub fn approve_milestone_analysis(
        &mut self,
        milestone_analysis_index: U256,
    ) -> Result<Option<BTreeMap<String, String>>, ProposalError> {
        let is_favorable = self
            .milestone_analyses
            .get(&milestone_analysis_index)
            .unwrap()
            .is_favorable;
        if !is_favorable {
            self.status = ProjectStatus::Remediation;
            Some(self.milestone_analyses.clone());
        } else {
            let (index, _) = self
                .proposal
                .milestones
                .get_key_value(&self.active_milestone)
                .unwrap()
                .clone();
            if index + 1 < self.proposal.milestones.len() as u64 {
                self.active_milestone = index + 1;
            } else {
                // Last milestone completed, project completed
                self.status = ProjectStatus::Completed;
            }
            self.status = ProjectStatus::Active;
        }

        Ok(None)
    }
    pub fn submit_milestone_analysis(
        &mut self,
        is_favorable: bool,
        recommendations: Vec<(String, String)>,
        vote_index: U256,
    ) -> Result<(), ProposalError> {
        let mut recommendations_mapping: BTreeMap<String, String> = BTreeMap::new();
        for recommendation in recommendations {
            recommendations_mapping.insert(recommendation.0, recommendation.1);
        }

        self.milestone_analyses.insert(
            vote_index,
            MilestoneAnalysis {
                is_favorable,
                recommendations: recommendations_mapping,
            },
        );
        self.status = ProjectStatus::VotingOnMilestoneAnalysis;
        Ok(())
    }
    pub fn extend_milestone_deadline(&mut self, new_timeout: u64) -> Result<(), ProposalError> {
        let mut milestone = self
            .proposal
            .milestones
            .get_mut(&self.active_milestone)
            .unwrap();
        self.status = ProjectStatus::Active;
        milestone.timeout = new_timeout;
        Ok(())
    }

    // pub struct MilestoneAnalysis {
    //     pub is_favorable: bool,
    //     pub recommendations: BTreeMap<String, String>,
    // }
    // #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
    // pub struct Project {
    //     pub proposal: Proposal,
    //     pub active_milestone: u64,
    //     pub status: ProjectStatus,
    //     pub claimed_milestones: Vec<Milestone>,
    //     pub milestone_analyses: BTreeMap<U256, MilestoneAnalysis>,
    // }
    // pub type MilestoneAnalysisSerialized = (bool, BTreeMap<String, String>);
    // pub type ProjectSerialized = (
    //     (ProposalSerialized, u64),
    //     (
    //         u8,
    //         Vec<MilestoneSerialized>,
    //         BTreeMap<U256, MilestoneAnalysisSerialized>,
    //     ),
    // );
    pub fn serialize(&self) -> ProjectSerialized {
        (
            (self.proposal.serialize(), self.active_milestone),
            (
                self.status as u8,
                self.serialize_claimed_milestones(),
                self.serialize_milestone_analyses(),
            ),
        )
    }

    fn serialize_milestone(milestone: Milestone) -> MilestoneSerialized {
        (
            (
                milestone.milestone_type,
                milestone.progress_percentage,
                milestone.result,
            ),
            (
                Self::serialize_funding_tranches(&milestone.funding_tranches),
                milestone.funding_tranches_size,
                milestone.timeout,
            ),
        )
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
    pub fn serialize_claimed_milestones(&self) -> Vec<MilestoneSerialized> {
        let mut output: Vec<MilestoneSerialized> = Vec::new();
        for mstone in &self.claimed_milestones {
            output.push(Self::serialize_milestone(mstone.clone()));
        }
        output
    }
    pub fn serialize_milestone_analyses(&self) -> BTreeMap<U256, MilestoneAnalysisSerialized> {
        let mut output: BTreeMap<U256, MilestoneAnalysisSerialized> = BTreeMap::new();
        for mstone in self.milestone_analyses.clone() {
            output.insert(mstone.0, (mstone.1.is_favorable, mstone.1.recommendations));
        }
        output
    }

    pub fn deserialize(project_serialized: ProjectSerialized) -> Project {
        Project {
            proposal: Proposal::deserialize(project_serialized.0 .0),
            active_milestone: project_serialized.0 .1,
            status: project_serialized.1 .0.into(),
            claimed_milestones: Self::deserialize_claimed_milestones(project_serialized.1 .1),
            milestone_analyses: Self::deserialize_milestone_analyses(project_serialized.1 .2),
        }
    }

    pub fn deserialize_claimed_milestones(
        serialized_milestones: Vec<MilestoneSerialized>,
    ) -> Vec<Milestone> {
        let mut output: Vec<Milestone> = Vec::new();
        for milestone in serialized_milestones {
            output.push(Milestone {
                milestone_type: milestone.0 .0,
                progress_percentage: milestone.0 .1,
                result: milestone.0 .2,
                funding_tranches: Self::deserialize_funding_tranches(milestone.1 .0),
                funding_tranches_size: milestone.1 .1,
                timeout: milestone.1 .2,
            })
        }
        output
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
    pub fn deserialize_milestone_analyses(
        serialized_milestones: BTreeMap<U256, MilestoneAnalysisSerialized>,
    ) -> BTreeMap<U256, MilestoneAnalysis> {
        let mut output: BTreeMap<U256, MilestoneAnalysis> = BTreeMap::new();
        for (key, milestone_analysis) in serialized_milestones {
            output.insert(
                key,
                MilestoneAnalysis {
                    is_favorable: milestone_analysis.0,
                    recommendations: milestone_analysis.1,
                },
            );
        }
        output
    }
}

impl From<u8> for ProjectStatus {
    fn from(orig: u8) -> Self {
        match orig {
            0u8 => return ProjectStatus::Active,
            1u8 => return ProjectStatus::MilestoneUnderReview,
            2u8 => return ProjectStatus::VotingOnMilestoneAnalysis,
            3u8 => return ProjectStatus::MilestoneTimeout,
            4u8 => return ProjectStatus::Completed,
            5u8 => return ProjectStatus::Remediation,
            _ => return ProjectStatus::Remediation,
        };
    }
}
