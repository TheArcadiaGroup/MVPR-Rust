#![no_std]
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use casperlabs_types::account::AccountHash;
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FundingTranche {
    pub funding_tranche_type: u64,
    pub amount: u64,
    pub reputation_allocation: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Milestone {
    pub milestone_type: u64,
    pub progressPercentage: u64,
    pub result: u64,
    pub funding_tranches: BTreeMap<u64, FundingTranche>,
    pub funding_tranches_size: u64,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Proposal {
    pub name: String,
    pub storagePointer: String,
    pub storageFingerprint: String,
    pub proposal_type: ProposalType,
    pub proposer: AccountHash,
    pub citations: Vec<u64>,
    pub ratios: Vec<u64>,
    pub vote_configuration: Vec<u64>,
    pub milestones: BTreeMap<u64, Milestone>,
    // pub participants: BTreeMap<AccountHash, Participant>,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProposalType {
    Signaling,
    Grant,
    Internal,
    External,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProposalStatus {
    Accepted,
    TransitionVote,
    FullVote,
    Withdrawn,
    Rejected,
    Discussion,
    PendingApproval,
}
