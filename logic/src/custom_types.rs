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

    pub type GovernanceProposalSerialized = (
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

    pub type GovernanceVoteConfigurationSerialized = ((U256, u64, String), (U256, u64, u64));

    pub type VotingDataSerialized = (U256, bool, bool);

    pub type VotersSerialized = BTreeMap<[u8; 32], VotingDataSerialized>;

    pub type VotingSerialized = (
        // (start, end)
        (
            (u64, u64, U256),
            (
                Option<ProposalSerialized>,
                Option<GovernanceProposalSerialized>,
                u8,
            ),
        ),
        (U256, U256, U256),
        (VotersSerialized, VotersSerialized, u8),
    );
}
