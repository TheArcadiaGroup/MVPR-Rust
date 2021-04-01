#![no_main]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::{collections::BTreeMap, collections::BTreeSet};
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
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, ApiError, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs,
    URef, U256,
};
mod errors;
use errors::Error;
const POLICING_RATIO_KEY: &str = "policing_ratio";
const VOTING_CONTRACT_HASH_KEY: &str = "voting_contract_hash";
const VOTING_ENGINE_CONTRACT_ADDRESS_KEY: &str = "voting_engine_contract_address";
const REPUTATION_ALLOCATION_RATIO_KEY: &str = "reputation_allocation_ratio";
const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";
const EXECUTION_CONTRACT_HASH_KEY: &str = "execution_contract_hash";

#[casperlabs_contract]
mod Governance {

    #[casperlabs_constructor]
    fn constructor(
        voting_engine_address: AccountHash,
        voting_contract_hash: ContractHash,
        reputation_contract_hash: ContractHash,
        policing_ratio: u64,
        execution_contract_hash: ContractHash,
    ) {
        set_key(VOTING_ENGINE_CONTRACT_ADDRESS_KEY, voting_engine_address);
        set_key(POLICING_RATIO_KEY, policing_ratio);
        set_key(VOTING_CONTRACT_HASH_KEY, voting_contract_hash);
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
        set_key(EXECUTION_CONTRACT_HASH_KEY, execution_contract_hash);
    }

    #[casperlabs_method]
    fn reputation_contract_hash() -> u64 {
        get_key(REPUTATION_CONTRACT_HASH_KEY)
    }
    #[casperlabs_method]
    fn update_reputation_contract_hash(update_reputation_contract_hash: AccountHash) -> bool {
        assert_voting_engine();
        set_key(
            REPUTATION_CONTRACT_HASH_KEY,
            update_reputation_contract_hash,
        );
        true
    }
    #[casperlabs_method]
    fn policing_ratio() -> u64 {
        get_key(POLICING_RATIO_KEY)
    }
    
    #[casperlabs_method]
    fn update_policing_ratio(update_policing_ratio: u64) -> bool {
        assert_voting_engine();
        set_key(POLICING_RATIO_KEY, update_policing_ratio);
        true
    }
    #[casperlabs_method]
    fn reputation_allocation_ratio() -> u64 {
        get_key(REPUTATION_ALLOCATION_RATIO_KEY)
    }
    #[casperlabs_method]
    fn update_reputation_allocation_ratio(update_reputation_allocation_ratio: u64) -> bool {
        assert_voting_engine();
        set_key(
            REPUTATION_ALLOCATION_RATIO_KEY,
            update_reputation_allocation_ratio,
        );
        true
    }

    #[casperlabs_method]
    fn voting_engine_address() -> AccountHash {
        get_key(VOTING_ENGINE_CONTRACT_ADDRESS_KEY)
    }
    #[casperlabs_method]
    fn update_voting_engine_address(update_voting_engine_address: AccountHash) -> bool {
        assert_voting_engine();
        set_key(
            VOTING_ENGINE_CONTRACT_ADDRESS_KEY,
            update_voting_engine_address,
        );
        true
    }

    #[casperlabs_method]
    fn voting_engine_contract_hash() -> ContractHash {
        get_key(VOTING_CONTRACT_HASH_KEY)
    }
    #[casperlabs_method]
    fn update_voting_engine_contract_hash(
        update_voting_engine_contract_hash: ContractHash,
    ) -> bool {
        set_key(VOTING_CONTRACT_HASH_KEY, update_voting_engine_contract_hash);
        true
    }
    #[casperlabs_method]
    fn execution_contract_hash() -> ContractHash {
        get_key(EXECUTION_CONTRACT_HASH_KEY)
    }
    #[casperlabs_method]
    fn update_execution_contract_hash(update_execution_contract_hash: ContractHash) -> bool {
        set_key(EXECUTION_CONTRACT_HASH_KEY, update_execution_contract_hash);
        true
    }
}

pub fn assert_voting_engine() {
    let voting_engine_address: AccountHash = get_key(VOTING_ENGINE_CONTRACT_ADDRESS_KEY);
    let caller = runtime::get_caller();
    if voting_engine_address != caller {
        runtime::revert(Error::NotVotingEngine);
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
