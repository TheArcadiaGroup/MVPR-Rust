#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use casperlabs_contract_macro::{casperlabs_constructor, casperlabs_contract, casperlabs_method};
use contract::{
    contract_api::{account, runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use core::convert::TryInto;
use types::{
    account::{
        AccountHash, ActionType, AddKeyFailure, RemoveKeyFailure, SetThresholdFailure,
        UpdateKeyFailure, Weight,
    },
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractHash,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, ApiError, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};

use logic::{
    custom_types::custom_types::GovernanceVoteConfigurationSerialized, voting::Voting,
    GovernanceProposal, Proposal, ProposalError, VotingEngineError,
};

const MINIMUM_STABILITY_TIME_KEY: &str = "minimum_stability_time";
const POLICING_RATIO: &str = "policing_ratio";
const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";
const GOVERNANCE_ADDRESS_KEY: &str = "governance";
const VOTING_CONTRACT_HASH_KEY: &str = "voting_contract_hash";
const VOTING_CONTRACT_ADDRESS_KEY: &str = "voting_contract_address";
const DEPLOYER_ADDRESS_KEY: &str = "deployer_address";
const VOTING_CONTRACT_CHANGED_KEY: &str = "voting_contract_changed";
const NUMBER_OF_GOVERNANCE_PROPOSALS_KEY: &str = "governance_proposals_number";
const NUMBER_OF_PROPOSALS_KEY: &str = "proposals_number";
#[casperlabs_contract]

mod ProposawlEngine {

    #[casperlabs_constructor]
    fn constructor(
        minimum_stability_time: U256,
        policing_ratio: u8,
        reputation_contract_hash: ContractHash,
        governance_address: AccountHash,
    ) {
        set_key(MINIMUM_STABILITY_TIME_KEY, minimum_stability_time);
        set_key(POLICING_RATIO, policing_ratio);
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
        set_key(GOVERNANCE_ADDRESS_KEY, governance_address);
        set_key(DEPLOYER_ADDRESS_KEY, runtime::get_caller());
        set_key(VOTING_CONTRACT_CHANGED_KEY, false);
    }
    #[casperlabs_method]
    fn name() -> String {
        get_key("_name")
    }

    #[casperlabs_method]
    fn policing_ratio() -> u8 {
        get_key(POLICING_RATIO)
    }

    #[casperlabs_method]
    fn reputation_contract_hash() -> u8 {
        get_key(REPUTATION_CONTRACT_HASH_KEY)
    }

    #[casperlabs_method]
    fn update_policing_ratio(new_policing_ratio: u8) {
        assert_caller(get_key(GOVERNANCE_ADDRESS_KEY));
        set_key(POLICING_RATIO, new_policing_ratio);
    }

    #[casperlabs_method]
    fn update_voting_contract_info(
        new_voting_contract_address: AccountHash,
        new_voting_contract_hash: ContractHash,
    ) {
        let deployer_address: AccountHash = get_key(DEPLOYER_ADDRESS_KEY);
        let voting_contact_changed: bool = get_key(VOTING_CONTRACT_CHANGED_KEY);
        // Deployer can only set the voting contract once after deployment.
        if voting_contact_changed {
            assert_caller(get_key(VOTING_CONTRACT_ADDRESS_KEY));
        } else {
            assert_caller(get_key(DEPLOYER_ADDRESS_KEY));
            set_key(VOTING_CONTRACT_CHANGED_KEY, true);
        }
        set_key(VOTING_CONTRACT_ADDRESS_KEY, new_voting_contract_address);
        set_key(VOTING_CONTRACT_HASH_KEY, new_voting_contract_hash);
    }

    #[casperlabs_method]
    fn create_proposal(
        name: String,
        storage_pointer: String,
        storage_fingerprint: String,
        category: u8,
        citations: Vec<u64>,
        ratios: Vec<u8>,
        vote_configuration: ((u64, U256), (u8, u64, U256)),
        milestones: Vec<(u8, u8, Vec<(u8, U256, U256)>)>,
        staked_rep: U256,
        sponsors: Vec<(AccountHash, U256)>,
        cost: U256,
    ) {
        let caller = runtime::get_caller();
        let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
        let voting_contract_changed: bool = get_key(VOTING_CONTRACT_CHANGED_KEY);
        if !voting_contract_changed {
            runtime::revert(Error::InvalidVotingContractAddress);
        }
        let mut is_member_args: RuntimeArgs = RuntimeArgs::new();
        is_member_args.insert("account", runtime::get_caller());

        let is_member: bool =
            runtime::call_contract::<bool>(reputation_contract_hash, "is_member", is_member_args);
        if (!is_member) {
            runtime::revert(Error::NotAMember);
        }

        let mut balance_of_args: RuntimeArgs = RuntimeArgs::new();
        balance_of_args.insert("account", runtime::get_caller());
        let reputation_balance: U256 =
            runtime::call_contract::<U256>(reputation_contract_hash, "balanceOf", balance_of_args);

        // TO DO: Use Oracles to get $100 Equivalent
        let system_policing_ratio: u8 = get_key(POLICING_RATIO);

        let proposal: Proposal = Proposal::new(
            name,
            storage_pointer,
            storage_fingerprint,
            category,
            citations,
            ratios,
            vote_configuration,
            milestones,
            staked_rep,
            caller,
            system_policing_ratio,
            reputation_balance,
            sponsors,
            cost,
        )
        .map_err(|e| runtime::revert(Error::from(e)))
        .unwrap();
        let new_proposal_index: U256 = get_key(NUMBER_OF_PROPOSALS_KEY);

        save_proposal(new_proposal_index, proposal.clone());

        // Send staked rep to voting engine
        let voting_contract_address: ContractHash = get_key(VOTING_CONTRACT_ADDRESS_KEY);
        let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
        transfer_args.insert("from", runtime::get_caller());
        transfer_args.insert("to", voting_contract_address);
        transfer_args.insert("amount", staked_rep);
        // Create new vote in voting engine
        let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
        runtime::call_contract::<bool>(reputation_contract_hash, "transferFrom", transfer_args);
        let mut new_vote_args: RuntimeArgs = RuntimeArgs::new();
        new_vote_args.insert("proposal", Proposal::serialize(&proposal));
        let voting_contract_hash: ContractHash = get_key(VOTING_CONTRACT_HASH_KEY);
        runtime::call_contract::<bool>(voting_contract_hash, "new_vote", new_vote_args);
    }

    #[casperlabs_method]
    fn create_governance_proposal(
        name: String,
        vote_configuration: GovernanceVoteConfigurationSerialized,
        staked_rep: U256,
        sponsors: Vec<(AccountHash, U256)>,
        repository_url: String,
        new_variable_key_value: (String, String),
    ) {
        let caller = runtime::get_caller();
        let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
        let voting_contract_changed: bool = get_key(VOTING_CONTRACT_CHANGED_KEY);
        if !voting_contract_changed {
            runtime::revert(Error::InvalidVotingContractAddress);
        }
        let mut is_member_args: RuntimeArgs = RuntimeArgs::new();
        is_member_args.insert("account", runtime::get_caller());

        let is_member: bool =
            runtime::call_contract::<bool>(reputation_contract_hash, "is_member", is_member_args);
        if (!is_member) {
            runtime::revert(Error::NotAMember);
        }

        let mut balance_of_args: RuntimeArgs = RuntimeArgs::new();
        balance_of_args.insert("account", runtime::get_caller());
        let reputation_balance: U256 =
            runtime::call_contract::<U256>(reputation_contract_hash, "balanceOf", balance_of_args);

        // TO DO: Use Oracles to get $100 Equivalent
        let system_policing_ratio: u8 = get_key(POLICING_RATIO);

        let governance_proposal: GovernanceProposal = GovernanceProposal::new(
            name,
            vote_configuration,
            staked_rep,
            caller,
            reputation_balance,
            sponsors,
            repository_url,
            new_variable_key_value,
        )
        .map_err(|e| runtime::revert(Error::from(e)))
        .unwrap();
        let new_governance_proposal_index: U256 = get_key(NUMBER_OF_GOVERNANCE_PROPOSALS_KEY);

        save_governance_proposal(new_governance_proposal_index, governance_proposal.clone());

        // Send staked rep to voting engine
        let voting_contract_address: ContractHash = get_key(VOTING_CONTRACT_ADDRESS_KEY);
        let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
        transfer_args.insert("from", runtime::get_caller());
        transfer_args.insert("to", voting_contract_address);
        transfer_args.insert("amount", staked_rep);
        // Create new vote in voting engine
        let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
        runtime::call_contract::<bool>(reputation_contract_hash, "transferFrom", transfer_args);
        let mut new_vote_args: RuntimeArgs = RuntimeArgs::new();
        new_vote_args.insert(
            "proposal",
            GovernanceProposal::serialize(&governance_proposal),
        );
        let voting_contract_hash: ContractHash = get_key(VOTING_CONTRACT_HASH_KEY);
        runtime::call_contract::<bool>(voting_contract_hash, "new_governance_vote", new_vote_args);
    }
}

fn save_proposal(proposal_index: U256, proposal: Proposal) {
    set_key(&proposal_key(proposal_index), proposal.serialize());
}

fn save_governance_proposal(governance_proposal_index: U256, proposal: GovernanceProposal) {
    set_key(
        &governance_proposal_key(governance_proposal_index),
        proposal.serialize(),
    );
}

fn governance_proposal_key(index: U256) -> String {
    format!("_governance_proposals_{}", index)
}
fn proposal_key(index: U256) -> String {
    format!("_proposals_{}", index)
}

fn read_proposal(proposal_index: U256) -> Proposal {
    let serialized = get_key(&proposal_key(proposal_index));
    Proposal::deserialize(serialized)
}
fn read_governance_proposal(proposal_index: U256) -> Proposal {
    let serialized = get_key(&governance_proposal_key(proposal_index));
    Proposal::deserialize(serialized)
}
#[repr(u16)]
pub enum Error {
    NotTheAdminAccount,
    InvalidPolicingRatio,
    NotAMember,
    InvalidCategory,
    InvalidMilestonesProgressPercentages,
    ProjectCostNotEqualToMilestonesSum,
    StakedRepGreaterThanReputationBalance,
    InvalidVotingContractAddress,
}
#[repr(u16)]
pub enum ProposalStatus {
    Accepted,
    TransitionVote,
    FullVote,
    Withdrawn,
    Rejected,
    Discussion,
    PendingApproval,
}
#[repr(u16)]
pub enum ProposalType {
    Signaling,
    Grant,
    Internal,
    External,
}

pub fn assert_admin() {
    let failsafe: AccountHash = get_key("_failSafe");
    let compliance: AccountHash = get_key("_compliance");
    let caller = runtime::get_caller();
    if failsafe != caller || compliance != caller {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

pub fn assert_caller(authorized_account: AccountHash) {
    let caller = runtime::get_caller();
    if caller != authorized_account {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

pub fn assert_proposal_owner(proposalId: U256) {
    let caller = runtime::get_caller();
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

impl From<ProposalError> for Error {
    fn from(error: ProposalError) -> Error {
        match error {
            ProposalError::InvalidCategory => Error::InvalidCategory,
            ProposalError::InvalidPolicingRatio => Error::InvalidPolicingRatio,
            ProposalError::InvalidMilestonesProgressPercentages => {
                Error::InvalidMilestonesProgressPercentages
            }
            ProposalError::ProjectCostNotEqualToMilestonesSum => {
                Error::ProjectCostNotEqualToMilestonesSum
            }
            ProposalError::StakedRepGreaterThanReputationBalance => {
                Error::StakedRepGreaterThanReputationBalance
            }
        }
    }
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}
