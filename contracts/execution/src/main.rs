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
    custom_types::custom_types::{ProjectSerialized, ProposalSerialized},
    Project, Proposal, ProposalError, ProposalType, Voting, VotingEngineError,
};

const VOTING_CONTRACT_HASH_KEY: &str = "voting_contract_hash";
const VOTING_ENGINE_CONTRACT_ADDRESS_KEY: &str = "voting_engine_contract_address";
const NUMBER_OF_PROJECTS_KEY: &str = "projects_number";
const GOVERNANCE_CONTRACT_HASH_KEY: &str = "governance_contract_hash";

#[casperlabs_contract]

mod ProjectExecutionEngine {

    #[casperlabs_constructor]
    fn constructor(voting_engine_address: AccountHash, voting_contract_hash: ContractHash) {
        set_key(VOTING_ENGINE_CONTRACT_ADDRESS_KEY, voting_engine_address);
        set_key(VOTING_CONTRACT_HASH_KEY, voting_contract_hash);
        set_key(NUMBER_OF_PROJECTS_KEY, 0);
    }

    fn internal_get_voting_engine_address() -> AccountHash {
        let args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<AccountHash>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "voting_engine_address",
            args,
        )
    }

    fn internal_get_voting_engine_contract_hash() -> ContractHash {
        let args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<ContractHash>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "voting_engine_contract_hash",
            args,
        )
    }

    #[casperlabs_method]
    fn voting_engine_address() -> AccountHash {
        internal_get_voting_engine_address()
    }

    #[casperlabs_method]
    fn voting_engine_contract_hash() -> ContractHash {
        internal_get_voting_engine_contract_hash()
    }

    #[casperlabs_method]
    fn new_project(proposal: ProposalSerialized) -> bool {
        assert_caller(internal_get_voting_engine_address());
        let index: U256 = get_key(NUMBER_OF_PROJECTS_KEY);
        Project::new(proposal)
            .map(|project| save_project(index, project))
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        set_key(NUMBER_OF_PROJECTS_KEY, index + 1);
        true
    }

    // Mark milesetone as complete
    #[casperlabs_method]
    fn trigger_milestone_completion(project_index: U256) {
        let project_serialized: ProjectSerialized = get_key(&project_key(project_index));
        let mut project: Project = Project::deserialize(project_serialized);
        assert_caller(project.proposal.proposer);
        project
            .claim_milestone()
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        save_project(project_index, project);
    }

    #[casperlabs_method]
    fn submit_milestone_analysis(
        project_index: U256,
        is_favorable: bool,
        recommendations: Vec<(String, String)>,
    ) {
        let project_serialized: ProjectSerialized = get_key(&project_key(project_index));
        let mut project: Project = Project::deserialize(project_serialized);
        assert_caller(project.proposal.proposer);
        let mut args: RuntimeArgs = RuntimeArgs::new();
        let mut new_proposal: Proposal = project.clone().proposal;
        new_proposal.proposal_type = ProposalType::AnalysisAcceptance;
        // cost here is project ID
        new_proposal.cost = project_index;
        args.insert("proposal", new_proposal.serialize());
        let vote_index: U256 =
            runtime::call_contract(internal_get_voting_engine_contract_hash(), "new_vote", args);
        project
            .submit_milestone_analysis(is_favorable, recommendations, vote_index)
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        save_project(project_index, project);
    }
    #[casperlabs_method]
    fn approve_milestone_analysis(project_index: U256, milestone_analysis_index: U256) {
        let project_serialized: ProjectSerialized = get_key(&project_key(project_index));
        let mut project: Project = Project::deserialize(project_serialized);
        assert_caller(internal_get_voting_engine_address());
        let output: Option<BTreeMap<String, String>> = project
            .approve_milestone_analysis(milestone_analysis_index)
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        match output {
            Some(recommendations) => {
                for recommendation in recommendations {
                    let mut args: RuntimeArgs = RuntimeArgs::new();
                    args.insert(recommendation.0, recommendation.1);
                }
            }
            None => {}
        }
        save_project(project_index, project.clone());
        let mut args: RuntimeArgs = RuntimeArgs::new();
        let mut new_proposal: Proposal = project.proposal;
        new_proposal.proposal_type = ProposalType::AnalysisAcceptance;
        args.insert("proposal", new_proposal.serialize());
        runtime::call_contract::<U256>(
            internal_get_voting_engine_contract_hash(),
            "new_vote",
            args,
        );
    }

    #[casperlabs_method]
    fn check_timeout(project_index: U256) -> bool {
        // IF Milestone Expires without an OP Claim the Remediation Process begins;
        let project_serialized: ProjectSerialized = get_key(&project_key(project_index));
        let mut project: Project = Project::deserialize(project_serialized);
        let result: bool = project
            .check_milestone_timeout(runtime::get_blocktime().into())
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        if (result) {
            // milestone timed out
            // update project
            save_project(project_index, project);
        }
        result
    }
}

fn save_project(project_index: U256, project: Project) {
    set_key(&project_key(project_index), project.serialize());
}

fn project_key(index: U256) -> String {
    format!("_projects_{}", index)
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
