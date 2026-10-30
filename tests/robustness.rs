//! Robustness tests: engine and parser must never panic on hostile or arbitrary input,
//! and output must remain valid CSV.

use payments_engine::engine::Engine;
use payments_engine::processor::{process, write_accounts};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parser_never_panics_on_arbitrary_bytes(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        let mut engine = Engine::new();
        let _ = process(&mut engine, &bytes[..]);

        let mut output = Vec::new();
        let write_res = write_accounts(&engine, &mut output);
        prop_assert!(write_res.is_ok());

        let out_str = String::from_utf8(output).expect("output should be valid utf-8 CSV");
        prop_assert!(out_str.starts_with("client,available,held,total,locked\n"));
    }

    #[test]
    fn parser_never_panics_on_csv_shaped_fuzz(
        rows in prop::collection::vec(
            (
                prop::string::string_regex("[a-zA-Z0-9_ -]{0,10}").unwrap(),
                prop::string::string_regex("[ -~]{0,10}").unwrap(),
                prop::string::string_regex("[ -~]{0,10}").unwrap(),
                prop::string::string_regex("[ -~]{0,10}").unwrap(),
            ),
            0..50
        )
    ) {
        let mut csv_input = String::from("type, client, tx, amount\n");
        for (col1, col2, col3, col4) in rows {
            csv_input.push_str(&format!("{col1},{col2},{col3},{col4}\n"));
        }

        let mut engine = Engine::new();
        let _ = process(&mut engine, csv_input.as_bytes());

        let mut output = Vec::new();
        prop_assert!(write_accounts(&engine, &mut output).is_ok());

        let out_str = String::from_utf8(output).expect("valid utf-8");
        prop_assert!(out_str.starts_with("client,available,held,total,locked\n"));
    }
}
