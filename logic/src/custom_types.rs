#![no_std]
extern crate alloc;

pub mod custom_types {
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec::Vec;
    use types::U256;

    pub type ProposalSerialized = (
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

    type MilestoneSerialized =
        BTreeMap<u64, ((u8, u8, u8), BTreeMap<u64, FundingTrancheSerialized>, u64)>;

    type SponsorsSerialized = BTreeMap<[u8; 32], U256>;

    type FundingTrancheSerialized = (u8, U256, U256);

    type RatiosSerialized = ([u8; 3]);
    type VoteConfigurationSerialized = ((u64, U256), (u8, u64, U256));
}
