//! Property tests comparing the optimized engine against a naive reference model.
//!
//! The reference model stores an unindexed log of transactions and replays the
//! entire history sequentially.

use payments_engine::amount::Amount;
use payments_engine::engine::Engine;
use payments_engine::model::Transaction;
use proptest::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefDisputeState {
    Normal,
    Disputed,
    ChargedBack,
}

#[derive(Debug, Clone)]
struct RefDeposit {
    client: u16,
    amount: Amount,
    state: RefDisputeState,
}

#[derive(Debug, Default, Clone)]
struct RefAccount {
    available: Amount,
    held: Amount,
    locked: bool,
}

#[derive(Debug, Default)]
struct ReferenceModel {
    accounts: HashMap<u16, RefAccount>,
    deposits: HashMap<u32, RefDeposit>,
}

impl ReferenceModel {
    fn new() -> Self {
        Self::default()
    }

    fn apply(&mut self, tx: Transaction) {
        match tx {
            Transaction::Deposit { client, tx, amount } => {
                if self.deposits.contains_key(&tx) {
                    return;
                }
                let account = self.accounts.entry(client).or_default();
                if account.locked {
                    return;
                }
                if let Some(new_avail) = account.available.checked_add(amount) {
                    if new_avail.checked_add(account.held).is_some() {
                        account.available = new_avail;
                        self.deposits.insert(
                            tx,
                            RefDeposit {
                                client,
                                amount,
                                state: RefDisputeState::Normal,
                            },
                        );
                    }
                }
            }
            Transaction::Withdrawal {
                client,
                tx: _,
                amount,
            } => {
                let account = self.accounts.entry(client).or_default();
                if account.locked || account.available < amount {
                    return;
                }
                if let Some(new_avail) = account.available.checked_sub(amount) {
                    account.available = new_avail;
                }
            }
            Transaction::Dispute { client, tx } => {
                if let Some(dep) = self.deposits.get_mut(&tx) {
                    if dep.client == client && dep.state == RefDisputeState::Normal {
                        let account = self.accounts.entry(client).or_default();
                        if let (Some(new_avail), Some(new_held)) = (
                            account.available.checked_sub(dep.amount),
                            account.held.checked_add(dep.amount),
                        ) {
                            account.available = new_avail;
                            account.held = new_held;
                            dep.state = RefDisputeState::Disputed;
                        }
                    }
                }
            }
            Transaction::Resolve { client, tx } => {
                if let Some(dep) = self.deposits.get_mut(&tx) {
                    if dep.client == client && dep.state == RefDisputeState::Disputed {
                        let account = self.accounts.entry(client).or_default();
                        if let (Some(new_avail), Some(new_held)) = (
                            account.available.checked_add(dep.amount),
                            account.held.checked_sub(dep.amount),
                        ) {
                            account.available = new_avail;
                            account.held = new_held;
                            dep.state = RefDisputeState::Normal;
                        }
                    }
                }
            }
            Transaction::Chargeback { client, tx } => {
                if let Some(dep) = self.deposits.get_mut(&tx) {
                    if dep.client == client && dep.state == RefDisputeState::Disputed {
                        let account = self.accounts.entry(client).or_default();
                        if let Some(new_held) = account.held.checked_sub(dep.amount) {
                            account.held = new_held;
                            account.locked = true;
                            dep.state = RefDisputeState::ChargedBack;
                        }
                    }
                }
            }
        }
    }
}

fn arb_tx() -> impl Strategy<Value = Transaction> {
    let client = 1u16..4;
    let tx = 1u32..10;
    let amount = (1u32..100_000).prop_map(|n| {
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
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn engine_matches_reference_model(ops in prop::collection::vec(arb_tx(), 0..100)) {
        let mut engine = Engine::new();
        let mut ref_model = ReferenceModel::new();

        for op in ops {
            let _ = engine.apply(op);
            ref_model.apply(op);
        }

        // Compare all accounts
        let engine_accounts = engine.accounts_sorted();
        for (client, ref_acc) in &ref_model.accounts {
            if let Some((_, eng_acc)) = engine_accounts.iter().find(|(c, _)| c == client) {
                prop_assert_eq!(eng_acc.available(), ref_acc.available, "client {} available", client);
                prop_assert_eq!(eng_acc.held(), ref_acc.held, "client {} held", client);
                prop_assert_eq!(eng_acc.total(), ref_acc.available.checked_add(ref_acc.held), "client {} total", client);
                prop_assert_eq!(eng_acc.is_locked(), ref_acc.locked, "client {} locked", client);
            } else {
                // If the account was never created in engine, it must be default in ref_model
                prop_assert_eq!(ref_acc.available, Amount::ZERO);
                prop_assert_eq!(ref_acc.held, Amount::ZERO);
                prop_assert_eq!(ref_acc.locked, false);
            }
        }
    }
}
