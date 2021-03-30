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
use logic::{
    custom_types::custom_types::{
        GovernanceProposalSerialized, ProposalSerialized, VotersSerialized, VotingSerialized,
    },
    voting::{VoteResult, Voting},
    Proposal,
};

const GOVERNANCE_CONTRACT_HASH_KEY: &str = "governance_contract_hash";
// const REPUTATION_CONTRACT_HASH_KEY: &str = "reputation_contract_hash";
// const REPUTATION_CONTRACT_ADDRESS_KEY: &str = "reputation_contract_address";
// const REPUTATION_ALLOCATION_RATIO_KEY: &str = "reputation_allocation_ratio";
const NUMBER_OF_VOTES_KEY: &str = "number_of_votes";

#[casperlabs_contract]
mod Vote {
    use contract::contract_api::runtime::has_key;

    #[casperlabs_constructor]
    fn constructor(
        reputation_contract_hash: ContractHash,
        reputation_contract_address: AccountHash,
        reputation_allocation_ratio: U256,
    ) {
        set_key(NUMBER_OF_VOTES_KEY, U256::from(0));
    }

    #[casperlabs_method]
    fn new_vote(proposal: ProposalSerialized) -> bool {
        // WHO CAN CALL THIS FUNCTION?
        // assert_caller(get_key(REPUTATION_CONTRACT_ADDRESS_KEY));
        let current_time: u64 = runtime::get_blocktime().into();
        let next_vote_index: U256 = get_key(NUMBER_OF_VOTES_KEY);
        Voting::new(current_time, proposal)
            .map(|vote| save_voting(next_vote_index, vote))
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        set_key(NUMBER_OF_VOTES_KEY, next_vote_index + 1);
        true
    }
    #[casperlabs_method]
    fn new_governance_vote(governance_proposal: GovernanceProposalSerialized) -> bool {
        // WHO CAN CALL THIS FUNCTION?
        // assert_caller(get_key(REPUTATION_CONTRACT_ADDRESS_KEY));
        let current_time: u64 = runtime::get_blocktime().into();
        let next_vote_index: U256 = get_key(NUMBER_OF_VOTES_KEY);
        Voting::new_governance(current_time, governance_proposal)
            .map(|vote| save_voting(next_vote_index, vote))
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        set_key(NUMBER_OF_VOTES_KEY, next_vote_index + 1);
        true
    }
    #[casperlabs_method]
    fn get_reputation_contract_hash() -> ContractHash {
        let mut args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<ContractHash>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "reputation_contract_hash",
            args,
        )
    }
    fn internal_get_reputation_contract_hash() -> ContractHash {
        let mut args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<ContractHash>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "reputation_contract_hash",
            args,
        )
    }

    #[casperlabs_method]
    fn get_reputation_allocation_ratio() -> u64 {
        let mut args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<u64>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "reputation_allocation_ratio",
            args,
        )
    }
    fn internal_get_reputation_allocation_ratio() -> u64 {
        let mut args: RuntimeArgs = RuntimeArgs::new();
        runtime::call_contract::<u64>(
            get_key(GOVERNANCE_CONTRACT_HASH_KEY),
            "reputation_allocation_ratio",
            args,
        )
    }

    #[casperlabs_method]
    fn cast_vote(vote_index: U256, reputation_to_stake: U256, vote_direction: bool) {
        assert_member();
        let caller = runtime::get_caller();
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized = get_key(&voting_key(vote_index));
        if (!runtime::has_key(&voting_key(vote_index))) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let mut vote: Voting = Voting::deserialize(vote_serialized);
        let reputation_contract_hash = internal_get_reputation_contract_hash();
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
    fn calculate_vote_outcome(vote_index: U256) -> u8 {
        let current_time: u64 = runtime::get_blocktime().into();
        let vote_serialized: Option<VotingSerialized> = get_key(&voting_key(vote_index));
        if (vote_serialized == None) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let mut vote: Voting = Voting::deserialize(vote_serialized.unwrap());
        let reputation_allocation_ratio: u64 = internal_get_reputation_allocation_ratio();
        let outcome: VoteResult;
        if vote.proposal_type as u8 == 0 {
            outcome = Voting::calculate_vote_outcome(
                &mut vote,
                current_time,
                reputation_allocation_ratio,
            )
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap();
        } else {
            // This is a governance proposal
            let (result, (executed, (key, value))) =
                Voting::calculate_governance_vote_outcome(&mut vote, current_time)
                    .map_err(|e| runtime::revert(Error::from(e)))
                    .unwrap();
            if (!executed) {
                let governance_contract_hash: ContractHash = get_key(GOVERNANCE_CONTRACT_HASH_KEY);
                let mut voting_engine_address_args: RuntimeArgs = RuntimeArgs::new();
                let voting_engine_contract_hash: ContractHash = runtime::call_contract(
                    governance_contract_hash,
                    "voting_engine_address",
                    voting_engine_address_args,
                );
                let new_variable_key_value: (String, String) =
                    vote.governance_proposal.unwrap().new_variable_key_value;
                let mut governance_args: RuntimeArgs = RuntimeArgs::new();
                if (new_variable_key_value.0.clone() == "update_policing_ratio") {
                    let value = new_variable_key_value.1.parse::<u64>().unwrap();
                    governance_args.insert(new_variable_key_value.0.clone(), value);
                } else if (new_variable_key_value.0.clone() == "update_reputation_allocation_ratio")
                {
                    let value = new_variable_key_value.1.parse::<u64>().unwrap();
                    governance_args.insert(new_variable_key_value.0.clone(), value);
                } else if (new_variable_key_value.0.clone() == "update_voting_engine_address") {
                    let account_hash_with_prefix =
                        "account-hash-".to_owned() + &new_variable_key_value.1;
                    let value: AccountHash =
                        AccountHash::from_formatted_str(&account_hash_with_prefix).unwrap();
                    governance_args.insert(new_variable_key_value.0.clone(), value);
                } else if (new_variable_key_value.0.clone() == "update_voting_engine_contract_hash"
                    || new_variable_key_value.0.clone() == "update_reputation_contract_hash")
                {
                    let value: ContractHash = ContractHash::from_formatted_str(
                        &("contract-".to_owned() + &new_variable_key_value.1),
                    )
                    .unwrap();
                    governance_args.insert(new_variable_key_value.0.clone(), value);
                }
                // Execute proposal.
                let execution_result: bool = runtime::call_contract(
                    voting_engine_contract_hash,
                    &new_variable_key_value.0,
                    governance_args,
                );
            }
            outcome = result;
        }
        outcome as u8
    }

    // Winning voters call this function to claim their reputation (staked + losers pro-rata)
    #[casperlabs_method]
    fn claim_reputation(vote_index: U256) {
        assert_member();
        if (!runtime::has_key(&voting_key(vote_index))) {
            runtime::revert(Error::VoteDoesNotExist);
        }
        let vote_serialized = get_key(&voting_key(vote_index));
        let mut vote: Voting = Voting::deserialize(vote_serialized);
        let gained_reputation: U256 = vote
            .claim_reputation(runtime::get_caller())
            .map_err(|e| runtime::revert(Error::from(e)))
            .unwrap_or(U256::from(0));
        save_voting(vote_index, vote);

        if (gained_reputation > U256::from(0)) {
            let reputation_contract_hash: ContractHash = internal_get_reputation_contract_hash();
            let mut transfer_args: RuntimeArgs = RuntimeArgs::new();
            transfer_args.insert("recipient", runtime::get_caller());
            transfer_args.insert("amount", gained_reputation);
            let transfer_result: bool =
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
    let reputation_contract_hash: ContractHash = internal_get_reputation_contract_hash();

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
