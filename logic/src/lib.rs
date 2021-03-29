#![no_std]

extern crate alloc;

mod error;
mod proposal;
pub mod voting;
pub mod custom_types;
pub use error::{ProposalError, VotingEngineError};

pub use proposal::Proposal;
// pub use voting::Voting;
