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
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, ApiError, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};

use proposal_logic::Proposal;

#[casperlabs_contract]
mod ProposalEngine {

    #[casperlabs_constructor]
    fn constructor(minimumStabilityTime: U256) {
        set_key("_minimumStabilityTime", minimumStabilityTime);
    }

    #[casperlabs_method]
    fn name() -> String {
        get_key("_name")
    }

    #[casperlabs_method]
    fn symbol() -> String {
        get_key("_symbol")
    }

    #[casperlabs_method]
    fn currentSupply() -> U256 {
        get_key("_currentSupply")
    }

    #[casperlabs_method]
    fn create_proposal(
        name: String,
        storage_pointer: String,
        storage_fingerprint: String,
        category: u8,
        citations: Vec<U256>,
        ratios: Vec<U256>,
        vote_configuration: Vec<U256>,
        milestone_types: Vec<U256>,
        milestone_progress_percentage: Vec<U256>,
        staked_rep: U256,
    ) -> bool {
        false
    }
}
#[repr(u16)]
pub enum Error {
    NotTheAdminAccount,
}
#[repr(u16)]
pub enum ProposalStatus {
    Accepted,
    TransitionVote,
    FullVote,
    Withdrawn,
    Rejected,
    MajorityNotReachedButAccepted,
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

fn proposal_key(id: &U256) -> String {
    format!("_proposals_{}", id)
}
