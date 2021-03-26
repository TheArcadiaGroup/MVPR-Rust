#![no_std]

extern crate alloc;

mod error;
mod proposal;
mod voting;
pub use error::{ProposalError, VotingError};

pub use proposal::Proposal;
// pub use voting::Voting;
