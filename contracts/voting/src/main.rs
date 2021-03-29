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
use logic::voting::{VoteResult, Voting};

const CURRENT_SUPPLY_KEY: &str = "_currentSupply";
const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";
const REPUTATION_CONTRACT_ADDRESS_KEY: &str = "reputation_contract_address";
const REPUTATION_ALLOCATION_RATIO_KEY: &str = "reputation_allocation_ratio";

#[casperlabs_contract]
mod Vote {

    #[casperlabs_constructor]
    fn constructor(
        reputation_contract_hash: ContractHash,
        reputation_contract_address: AccountHash,
        reputation_allocation_ratio: U256,
    ) {
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
        set_key(REPUTATION_CONTRACT_ADDRESS_KEY, reputation_contract_address);
        set_key(REPUTATION_ALLOCATION_RATIO_KEY, reputation_allocation_ratio);
    }

    fn change_reputation_contract_info(
        reputation_contract_hash: ContractHash,
        reputation_contract_address: AccountHash,
    ) {
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
        set_key(REPUTATION_CONTRACT_ADDRESS_KEY, reputation_contract_address);
    }

    fn change_reputation_allocation_ratio(reputation_contract_hash: ContractHash) {
        set_key(REPUTATION_CONTRACT_HASH_KEY, reputation_contract_hash);
    }

    #[casperlabs_method]
    fn new_vote(proposal: Proposal) {
        assert_caller(get_key(REPUTATION_CONTRACT_ADDRESS_KEY));
        // Voting::new();
    }

    #[casperlabs_method]
    fn cast_vote(vote_index: U256, reputation_to_stake: U256, vote_direction: bool) {
        assert_member();
        let caller = runtime::get_caller();
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized = get_key(&voting_key(vote_index));
        if (vote_serialized == None) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let mut vote: Voting = Voting::deserialize(vote_serialized);
        let reputation_contract_hash = get_key(REPUTATION_CONTRACT_HASH_KEY);
        let mut balance_of_args: RuntimeArgs = RuntimeArgs::new();
        balance_of_args.insert("account", caller);
        let reputation_balance: U256 =
            runtime::call_contract::<U256>(reputation_contract_hash, "balanceOf", balance_of_args);
        let committed_reputation = get_key(&committed_reputation_key(&caller));
        vote.cast_vote(
            caller,
            current_time,
            reputation_balance,
            reputation_to_stake,
            committed_reputation,
            vote_direction,
        )
        .map_err(|e| runtime::revert(Error::from(e)))
        .unwrap();
        save_voting(vote_index, vote);
        set_key(
            &committed_reputation_key(&caller),
            committed_reputation + reputation_to_stake,
        );
    }

    #[casperlabs_method]
    fn calculate_vote_outcome(vote_index: U256) -> VoteResult {
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized: Option<VoteSerialized> = get_key(&voting_key(vote_index));
        if (vote_serialized == None) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let vote: Voting = Voting::deserialize(vote_serialized.unwrap());
        let reputation_allocation_ratio: U256 = get_key(REPUTATION_ALLOCATION_RATIO_KEY);
        let mut outcome: VoteResult =
            Voting::calculate_vote_outcome(&mut vote, current_time, reputation_allocation_ratio)
                .map_err(|e| runtime::revert(Error::from(e)))
                .unwrap();
        outcome
    }

    // Winning voters call this function to claim their reputation (staked + losers pro-rata)
    #[casperlabs_method]
    fn claim_reputation(vote_index: U256) {
        assert_member();
        let vote_serialized = get_key(&voting_key(vote_index));
        if (vote_serialized == None) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let mut vote: Voting = Voting::deserialize(vote_serialized);
        let gained_reputation: U256 = vote
            .claim_reputation(runtime::get_caller())
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap_or(U256::from(0));
        save_voting(vote_index, vote);

        if (gained_reputation > U256::from(0)) {
            let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);
            let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
            transfer_args.insert("recipient", runtime::get_caller());
            transfer_args.insert("amount", gained_reputation);
            let transfer_result =
                runtime::call_contract(reputation_contract_hash, "transfer", transfer_args);
        }
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
pub fn assert_member() {
    let caller = runtime::get_caller();
    let reputation_contract_hash: ContractHash = get_key(REPUTATION_CONTRACT_HASH_KEY);

    let mut is_member_args: RuntimeArgs = RuntimeArgs::new();
    is_member_args.insert("account", runtime::get_caller());

    let is_member: bool =
        runtime::call_contract::<bool>(reputation_contract_hash, "is_member", is_member_args);
    if (!is_member) {
        runtime::revert(Error::NotAMember);
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

pub fn assert_caller(authorized_account: AccountHash) {
    let caller = runtime::get_caller();
    if caller != authorized_account {
        runtime::revert(Error::NotTheAdminAccount);
    }
}

fn save_voting(index: U256, vote: Voting) {
    set_key(&voting_key(index), vote.serialize());
}

fn read_voting(index: U256) -> Voting {
    let serialized = get_key(&voting_key(index));
    Voting::deserialize(serialized)
}

fn voting_key(index: U256) -> String {
    format!("_votes_{}", index)
}

fn committed_reputation_key(account: &AccountHash) -> String {
    format!("_committed_reputation_{}", account)
}

fn member_key(account: &AccountHash) -> String {
    format!("_members_{}", account)
}
