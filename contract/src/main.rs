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
    runtime_args, CLType, CLTyped, CLValue, Group, Parameter, RuntimeArgs, URef, U256,Key
};

const KEY: &str = "special_value";

// macro to set up the contract

#[casperlabs_contract]
mod tutorial {
    use super::*;

// constructor macro that sets up the methods, values and keys required for the contract.

    #[casperlabs_constructor]
    fn init_counter(initial_value: u64) {
        let value_ref: URef = storage::new_uref(initial_value);
        let value_key: Key = value_ref.into();
        runtime::put_key(KEY, value_key);
    }

// method macro that defines a new entry point for the contract.

    #[casperlabs_method]
    fn update_counter() {
        let old_value: u64 = key(KEY).unwrap();
        let new_value = old_value + 1;
        set_key(KEY, new_value);
    }

// method macro that defines a new entry point for the contract.

    #[casperlabs_method]
    fn get_counter_value() -> u64 {
        key(KEY).unwrap()
    }

    fn key<T: FromBytes + CLTyped>(name: &str) -> Option<T> {
        match runtime::get_key(name) {
            None => None,
            Some(maybe_key) => {
                let key = maybe_key.try_into().unwrap_or_revert();
                let value = storage::read(key).unwrap_or_revert().unwrap_or_revert();
                Some(value)
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
}