#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::convert::TryInto;

use casperlabs_contract_macro::{casperlabs_constructor, casperlabs_contract, casperlabs_method};
use contract::{
    contract_api::{account, runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
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

use proposal_logic::{Proposal, ProposalError};

const MINIMUM_STABILITY_TIME_KEY: &str = "minimum_stability_time";
const POLICING_RATIO: &str = "proposal_ratio";
const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";
const PROPOSALS_KEY: &str = "proposals";
#[casperlabs_contract]

mod ProposawlEngine {
    use types::U256;

    #[casperlabs_constructor]
    fn constructor(
        minimumStabilityTime: U256,
        policing_ratio: u8,
        reputation_contract_hash: ContractHash,
    ) {
        set_key(MINIMUM_STABILITY_TIME_KEY, minimumStabilityTime);
        set_key(POLICING_RATIO, policing_ratio);
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
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
    fn create_proposal(
        name: String,
        storage_pointer: String,
        storage_fingerprint: String,
        category: u8,
        citations: Vec<u64>,
        ratios: Vec<u8>,
        vote_configuration: ((u64, u64), (u8, u8, u8)),
        milestones: Vec<(u8, u8, Vec<(u8, U256, U256)>)>,
        staked_rep: U256,
        sponsors: Vec<(AccountHash, U256)>,
        cost: U256,
    ) {
        let caller = runtime::get_caller();
        let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);

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

        Proposal::new(
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
        .map(save_proposal)
        .map_err(|e| runtime::revert(Error::from(e)))
        .unwrap();
        // Voting::new(start_at, end_at)
        //     .map(save_voting)
        //     .map_err(|e| runtime::revert(Error::from(e)))
        //     .unwrap()
    }
}

fn save_proposal(proposal: Proposal) {
    let mut test: BTreeMap<u64, String> = BTreeMap::new();
    set_key(PROPOSALS_KEY, proposal.serialize());
}

fn read_voting() -> Proposal {
    let serialized = get_key(PROPOSALS_KEY);
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
