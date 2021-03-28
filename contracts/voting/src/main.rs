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
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args, ApiError, CLType, CLTyped, CLValue, ContractHash, Group, Parameter, RuntimeArgs,
    URef, U256,
};
mod errors;
use errors::Error;
use logic::voting::Voting;

const CURRENT_SUPPLY_KEY: &str = "_currentSupply";
const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";

#[casperlabs_contract]
mod Voting {

    #[casperlabs_constructor]
    fn constructor(reputation_contract_hash: ContractHash) {
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
    }

    #[casperlabs_method]
    fn calculate_vote_outcome(vote_index: U256) -> u8 {
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized = get_key(&voting_key(vote_index));
        let vote: Voting = Voting::deserialize(vote_serialized);
        Voting::calculate_vote_outcome(&mut vote, current_time);
        5 as u8
    }

    // Winning voters call this function to claim their reputation (staked + losers pro-rata)
    #[casperlabs_method]
    fn claim_reputation(vote_index: U256) -> u8 {
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized = get_key(&voting_key(vote_index));
        let mut vote: Voting = Voting::deserialize(vote_serialized);
        let gained_reputation: U256 =
            Voting::claim_reputation(&mut vote, runtime::get_caller()).unwrap_or(U256::from(0));
        if (gained_reputation > U256::from(0)) {
            let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
            let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
            transfer_args.insert("recipient", runtime::get_caller());
            transfer_args.insert("amount", gained_reputation);
            let transfer_result =
                runtime::call_contract(reputation_contract_hash, "transfer", transfer_args);
        }

        5 as u8
    }
}

pub fn assert_admin() {
    let failsafe: AccountHash = get_key("_failSafe");
    let compliance: AccountHash = get_key("_compliance");
    let caller = runtime::get_caller();
    if failsafe != caller || compliance != caller {
        runtime::revert(Error::NotTheAdminAccount);
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

fn voting_key(index: U256) -> String {
    format!("_votes_{}", index)
}

fn member_key(account: &AccountHash) -> String {
    format!("_members_{}", account)
}
