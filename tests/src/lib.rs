#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, SessionBuilder, TestContextBuilder};
    use casper_types::{account::AccountHash, runtime_args, PublicKey, RuntimeArgs, U512};

    #[test]
    fn should_initialize_to_zero() {
        let ali: PublicKey = PublicKey::ed25519([3u8; 32]).unwrap();
        let MY_ACCOUNT: AccountHash = ali.to_account_hash();
        pub const FAILSAFE: AccountHash = AccountHash::new([1u8; 32]);
        pub const COMPLIANCE: AccountHash = AccountHash::new([2u8; 32]);
        pub const VOTING_ENGINE: AccountHash = AccountHash::new([3u8; 32]);
        const KEY: &str = "_name";
        const CONTRACT: &str = "Reputation";

        let mut context = TestContextBuilder::new()
            .with_public_key(
                ali,
                ali.to_account_hash(),
                U512::from(500_000_000_000_000_000u64),
            )
            .build();
        let session_code = Code::from("Contract.wasm");
        let session_args = runtime_args! {
            "tokenName" => "REPUTATION",
            "tokenSymbol" => "REP",
            "voting_engine" => VOTING_ENGINE,
            "failsafe" => FAILSAFE,
            "compliance" => COMPLIANCE
        };
        let session = SessionBuilder::new(session_code, session_args)
            .with_address(MY_ACCOUNT)
            .with_authorization_keys(&[MY_ACCOUNT])
            .build();
        context.run(session);
        let check: String =
            match context.query(MY_ACCOUNT, &[CONTRACT.to_string(), KEY.to_string()]) {
                Err(_) => panic!("Error"),
                Ok(maybe_value) => maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", KEY)),
            };
        assert_eq!(check, "REPUTATION");
    }
}
