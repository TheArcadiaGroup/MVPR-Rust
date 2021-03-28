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
pub mod custom_types {
    pub type ProposalSerialized = (
        (
            // 0
            // name, storage_pointer, storage_fingerprint
            (
                alloc::string::String,
                alloc::string::String,
                alloc::string::String,
            ), // 0.0
            (
                // 0.1
                // proposal type
                u8,
                // proposer
                [u8; 32],
                // citations
                alloc::vec::Vec<u64>,
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
            types::U256, //0.2
        ),
    );

    type MilestoneSerialized = alloc::collections::BTreeMap<
        u64,
        (
            (u8, u8, u8),
            alloc::collections::BTreeMap<u64, FundingTrancheSerialized>,
            u64,
        ),
    >;

    type SponsorsSerialized = alloc::collections::BTreeMap<[u8; 32], types::U256>;

    type FundingTrancheSerialized = (u8, types::U256, types::U256);

    type RatiosSerialized = ([u8; 3]);
    type VoteConfigurationSerialized = ((u64, u64), (u8, u8, u8));
}
