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
    runtime_args, ApiError, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,
};
mod errors;
use errors::Error;
const CURRENT_SUPPLY_KEY: &str = "_currentSupply";
#[casperlabs_contract]
mod Reputation {
    use types::account::UpdateKeyFailure;

    #[casperlabs_constructor]
    fn constructor(
        tokenName: String,
        tokenSymbol: String,
        voting_engine: AccountHash,
        failsafe: AccountHash,
        compliance: AccountHash,
    ) {
        set_key("_name", tokenName);
        set_key("_symbol", tokenSymbol);
        set_key("_granularity", 1);
        set_key("_currentSupply", 0);
        set_key("_votingEngine", voting_engine);
        set_key("_failSafe", failsafe);
        set_key("_compliance", compliance);
        set_key(&member_key(&compliance), true);
        set_key(&member_key(&failsafe), true);
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
    fn hasLimit() -> bool {
        false
    }

    #[casperlabs_method]
    fn balance_of(account: AccountHash) -> U256 {
        get_key(&balance_key(&account))
    }
    #[casperlabs_method]
    fn transfer(recipient: AccountHash, amount: U256) {
        _transfer(runtime::get_caller(), recipient, amount);
    }

    fn _transfer(sender: AccountHash, recipient: AccountHash, amount: U256) {
        let sender_key = balance_key(&sender);
        let recipient_key = balance_key(&recipient);
        let new_sender_balance: U256 = (get_key::<U256>(&sender_key) - amount);
        set_key(&sender_key, new_sender_balance);
        let new_recipient_balance: U256 = (get_key::<U256>(&recipient_key) + amount);
        set_key(&recipient_key, new_recipient_balance);
    }

    #[casperlabs_method]
    fn transferFrom(from: AccountHash, to: AccountHash, amount: U256) {
        assert_admin();
        let mut sender_balance: U256 = get_key(&balance_key(&from));
        let mut receiver_balance: U256 = get_key(&balance_key(&to));
        sender_balance = sender_balance - amount;
        receiver_balance = receiver_balance + amount;
        set_key(&balance_key(&from), sender_balance);
        set_key(&balance_key(&to), receiver_balance);
    }
    #[casperlabs_method]
    fn mint(account: AccountHash, weight: Weight) {
        // assert_admin();
        // let mut currentSupply: U256 = get_key(CURRENT_SUPPLY_KEY);
        // currentSupply = currentSupply + amount;
        // set_key(&CURRENT_SUPPLY_KEY, currentSupply);
        // let new_balance: U256 = get_key(&balance_key(&account));
        // set_key(&balance_key(&account), new_balance + amount);
        match account::update_associated_key(account, weight) {
            Ok(_) => (),
            Err(UpdateKeyFailure::MissingKey) => add_key(account, weight).unwrap_or_default(),
            Err(UpdateKeyFailure::PermissionDenied) => runtime::revert(Error::PermissionDenied),
            Err(UpdateKeyFailure::ThresholdViolation) => runtime::revert(Error::ThresholdViolation),
        };
    }
    fn add_key(key: AccountHash, weight: Weight) -> Result<(), Error> {
        match account::add_associated_key(key, weight) {
            Ok(()) => Ok(()),
            Err(AddKeyFailure::MaxKeysLimit) => runtime::revert(Error::MaxKeysLimit),
            Err(AddKeyFailure::DuplicateKey) => runtime::revert(Error::DuplicateKey), // Should never happen.
            Err(AddKeyFailure::PermissionDenied) => runtime::revert(Error::PermissionDenied),
        }
    }
    #[casperlabs_method]
    fn burn(account: AccountHash, amount: U256) {
        assert_admin();
        let mut currentSupply: U256 = get_key(CURRENT_SUPPLY_KEY);
        currentSupply = currentSupply - amount;
        set_key(&CURRENT_SUPPLY_KEY, currentSupply);
        let new_balance: U256 = get_key(&balance_key(&account));
        set_key(&balance_key(&account), new_balance - amount);
    }

    #[casperlabs_method]
    fn is_member(account: AccountHash) -> bool {
        runtime::has_key(&member_key(&account))
    }

    #[casperlabs_method]
    fn add_member(account: AccountHash) {
        assert_admin();
        set_key(&member_key(&account), true);
    }
    #[casperlabs_method]
    fn remove_member(account: AccountHash) {
        assert_admin();
        runtime::remove_key(&member_key(&account));
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

fn balance_key(account: &AccountHash) -> String {
    format!("_balances_{}", account)
}

fn member_key(account: &AccountHash) -> String {
    format!("_members_{}", account)
}
