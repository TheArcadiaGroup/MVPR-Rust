#![no_std]

extern crate alloc;

pub mod custom_types;
mod error;
mod proposal;
pub mod voting;
pub use error::{ProposalError, VotingEngineError};

pub use proposal::{GovernanceProposal, GovernanceVoteConfiguration, Proposal};
// pub use voting::Voting;
