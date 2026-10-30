//! Property tests: whatever sequence of operations arrives, core invariants hold.

use payments_engine::amount::Amount;
use payments_engine::engine::Engine;
use payments_engine::model::Transaction;
use proptest::prelude::*;

fn arb_tx() -> impl Strategy<Value = Transaction> {
    let client = 1u16..4;
    let tx = 1u32..12;
    let amount = (1u32..1_000_000).prop_map(|n| {
        Amount::parse_positive(&format!("{}.{:04}", n / 10_000, n % 10_000)).unwrap()
    });
    prop_oneof![
        (client.clone(), tx.clone(), amount.clone())
            .prop_map(|(client, tx, amount)| Transaction::Deposit { client, tx, amount }),
        (client.clone(), tx.clone(), amount)
            .prop_map(|(client, tx, amount)| Transaction::Withdrawal { client, tx, amount }),
        (client.clone(), tx.clone()).prop_map(|(client, tx)| Transaction::Dispute { client, tx }),
        (client.clone(), tx.clone()).prop_map(|(client, tx)| Transaction::Resolve { client, tx }),
        (client, tx).prop_map(|(client, tx)| Transaction::Chargeback { client, tx }),
    ]
}

proptest! {
    #[test]
    fn total_always_equals_available_plus_held(ops in prop::collection::vec(arb_tx(), 0..200)) {
        let mut engine = Engine::new();
        for op in ops {
            let _ = engine.apply(op); // rejections are fine; panics are not
        }
        for (_, a) in engine.accounts_sorted() {
            prop_assert_eq!(a.total(), a.available().checked_add(a.held()).unwrap());
        }
    }

    #[test]
    fn held_funds_are_never_negative(ops in prop::collection::vec(arb_tx(), 0..200)) {
        let mut engine = Engine::new();
        for op in ops {
            let _ = engine.apply(op);
        }
        for (_, a) in engine.accounts_sorted() {
            prop_assert!(a.held() >= Amount::ZERO);
        }
    }
}
