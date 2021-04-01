#![no_std]

extern crate alloc;

pub mod custom_types;
mod error;
mod execution;
mod proposal;
mod voting;

pub use error::{ProposalError, VotingEngineError};

pub use {
    execution::Project,
    proposal::{GovernanceProposal, GovernanceVoteConfiguration, Proposal, ProposalType},
    voting::{VoteResult, Voting},
};
