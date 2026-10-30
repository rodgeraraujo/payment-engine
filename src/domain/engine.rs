//! The transaction engine: pure business logic, no I/O.

use crate::domain::account::Account;
use crate::domain::amount::Amount;
use crate::error::EngineError;
use std::collections::HashMap;

/// A validated domain transaction ready to be applied to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transaction {
    /// Funds credited to a client's available balance.
    Deposit {
        /// Client identifier.
        client: u16,
        /// Transaction identifier.
        tx: u32,
        /// Amount deposited.
        amount: Amount,
    },
    /// Funds debited from a client's available balance.
    Withdrawal {
        /// Client identifier.
        client: u16,
        /// Transaction identifier.
        tx: u32,
        /// Amount withdrawn.
        amount: Amount,
    },
    /// Dispute initiated against a prior deposit.
    Dispute {
        /// Disputing client identifier.
        client: u16,
        /// Referenced deposit transaction identifier.
        tx: u32,
    },
    /// Resolution of an active dispute, releasing held funds back to available.
    Resolve {
        /// Client identifier.
        client: u16,
        /// Referenced deposit transaction identifier.
        tx: u32,
    },
    /// Chargeback finalizing a dispute, deducting held funds and freezing the account.
    Chargeback {
        /// Client identifier.
        client: u16,
        /// Referenced deposit transaction identifier.
        tx: u32,
    },
}

/// Lifecycle of a deposit with respect to disputes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisputeState {
    Normal,
    Disputed,
    ChargedBack,
}

impl DisputeState {
    fn name(self) -> &'static str {
        match self {
            DisputeState::Normal => "normal",
            DisputeState::Disputed => "disputed",
            DisputeState::ChargedBack => "charged back",
        }
    }
}

/// Only deposits are remembered (they are the only disputable transactions).
/// Kept small on purpose: ~24 bytes of payload per deposit.
#[derive(Debug)]
struct StoredDeposit {
    client: u16,
    amount: Amount,
    state: DisputeState,
}

/// Stateful ledger processing financial transactions.
#[derive(Debug, Default)]
pub struct Engine {
    accounts: HashMap<u16, Account>,
    deposits: HashMap<u32, StoredDeposit>,
}

impl Engine {
    /// Creates a new, empty transaction engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies one transaction. An `Err` means the transaction was rejected
    /// and state is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use payments_engine::amount::Amount;
    /// use payments_engine::engine::Engine;
    /// use payments_engine::model::Transaction;
    ///
    /// let mut engine = Engine::new();
    /// let amount = Amount::parse_positive("50.0000")?;
    /// engine.apply(Transaction::Deposit {
    ///     client: 1,
    ///     tx: 1,
    ///     amount,
    /// })?;
    /// assert_eq!(engine.accounts_sorted().len(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn apply(&mut self, transaction: Transaction) -> Result<(), EngineError> {
        match transaction {
            Transaction::Deposit { client, tx, amount } => self.deposit(client, tx, amount),
            Transaction::Withdrawal { client, tx, amount } => self.withdraw(client, tx, amount),
            Transaction::Dispute { client, tx } => self.dispute(client, tx),
            Transaction::Resolve { client, tx } => self.resolve(client, tx),
            Transaction::Chargeback { client, tx } => self.chargeback(client, tx),
        }
    }

    /// Accounts ordered by client id, for deterministic output.
    pub fn accounts_sorted(&self) -> Vec<(u16, &Account)> {
        let mut rows: Vec<_> = self.accounts.iter().map(|(id, a)| (*id, a)).collect();
        rows.sort_unstable_by_key(|(id, _)| *id);
        rows
    }

    fn deposit(&mut self, client: u16, tx: u32, amount: Amount) -> Result<(), EngineError> {
        if self.deposits.contains_key(&tx) {
            return Err(EngineError::DuplicateTx(tx));
        }
        self.accounts
            .entry(client)
            .or_default()
            .deposit(client, amount)?;
        self.deposits.insert(
            tx,
            StoredDeposit {
                client,
                amount,
                state: DisputeState::Normal,
            },
        );
        Ok(())
    }

    fn withdraw(&mut self, client: u16, tx: u32, amount: Amount) -> Result<(), EngineError> {
        self.accounts
            .entry(client)
            .or_default()
            .withdraw(client, tx, amount)
    }

    fn dispute(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let (account, deposit) = self.lookup(client, tx)?;
        require_state(deposit, tx, "dispute", DisputeState::Normal)?;
        account.hold(client, deposit.amount)?;
        deposit.state = DisputeState::Disputed;
        Ok(())
    }

    fn resolve(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let (account, deposit) = self.lookup(client, tx)?;
        require_state(deposit, tx, "resolve", DisputeState::Disputed)?;
        account.release(client, deposit.amount)?;
        deposit.state = DisputeState::Normal;
        Ok(())
    }

    fn chargeback(&mut self, client: u16, tx: u32) -> Result<(), EngineError> {
        let (account, deposit) = self.lookup(client, tx)?;
        require_state(deposit, tx, "chargeback", DisputeState::Disputed)?;
        account.chargeback(client, deposit.amount)?;
        deposit.state = DisputeState::ChargedBack;
        Ok(())
    }

    /// Finds the deposit and its owner's account, enforcing that the
    /// referencing client actually owns the transaction.
    fn lookup(
        &mut self,
        client: u16,
        tx: u32,
    ) -> Result<(&mut Account, &mut StoredDeposit), EngineError> {
        let deposit = self
            .deposits
            .get_mut(&tx)
            .ok_or(EngineError::UnknownTx(tx))?;
        if deposit.client != client {
            return Err(EngineError::ClientMismatch { tx, client });
        }
        let account = self
            .accounts
            .get_mut(&client)
            .ok_or(EngineError::UnknownTx(tx))?;
        Ok((account, deposit))
    }
}

fn require_state(
    deposit: &StoredDeposit,
    tx: u32,
    action: &'static str,
    expected: DisputeState,
) -> Result<(), EngineError> {
    if deposit.state == expected {
        Ok(())
    } else {
        Err(EngineError::InvalidState {
            tx,
            action,
            state: deposit.state.name(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amt(s: &str) -> Amount {
        Amount::parse_positive(s).unwrap()
    }
    fn dep(client: u16, tx: u32, a: &str) -> Transaction {
        Transaction::Deposit {
            client,
            tx,
            amount: amt(a),
        }
    }
    fn wd(client: u16, tx: u32, a: &str) -> Transaction {
        Transaction::Withdrawal {
            client,
            tx,
            amount: amt(a),
        }
    }
    fn state(e: &Engine, client: u16) -> (String, String, String, bool) {
        let a = &e.accounts[&client];
        (
            a.available().to_string(),
            a.held().to_string(),
            a.total().expect("overflow in total").to_string(),
            a.is_locked(),
        )
    }
    fn s(a: &str, h: &str, t: &str, l: bool) -> (String, String, String, bool) {
        (a.into(), h.into(), t.into(), l)
    }

    #[test]
    fn deposit_and_withdrawal() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "3.0")).unwrap();
        e.apply(wd(1, 2, "1.25")).unwrap();
        assert_eq!(state(&e, 1), s("1.7500", "0.0000", "1.7500", false));
    }

    #[test]
    fn insufficient_funds_leaves_balance_unchanged() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "1.0")).unwrap();
        assert_eq!(
            e.apply(wd(1, 2, "1.0001")),
            Err(EngineError::InsufficientFunds { client: 1, tx: 2 })
        );
        assert_eq!(state(&e, 1), s("1.0000", "0.0000", "1.0000", false));
    }

    #[test]
    fn dispute_then_resolve() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        assert_eq!(state(&e, 1), s("0.0000", "10.0000", "10.0000", false));
        e.apply(Transaction::Resolve { client: 1, tx: 1 }).unwrap();
        assert_eq!(state(&e, 1), s("10.0000", "0.0000", "10.0000", false));
    }

    #[test]
    fn chargeback_freezes_and_blocks_further_activity() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        e.apply(Transaction::Chargeback { client: 1, tx: 1 })
            .unwrap();
        assert_eq!(state(&e, 1), s("0.0000", "0.0000", "0.0000", true));
        assert_eq!(e.apply(dep(1, 2, "5")), Err(EngineError::AccountLocked(1)));
        assert_eq!(e.apply(wd(1, 3, "1")), Err(EngineError::AccountLocked(1)));
    }

    #[test]
    fn dispute_after_spending_goes_negative_but_total_is_consistent() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(wd(1, 2, "8")).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        assert_eq!(state(&e, 1), s("-8.0000", "10.0000", "2.0000", false));
    }

    #[test]
    fn invalid_references_are_rejected() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(dep(2, 2, "10")).unwrap();
        // unknown tx
        assert_eq!(
            e.apply(Transaction::Dispute { client: 1, tx: 99 }),
            Err(EngineError::UnknownTx(99))
        );
        // another client's tx
        assert_eq!(
            e.apply(Transaction::Dispute { client: 2, tx: 1 }),
            Err(EngineError::ClientMismatch { tx: 1, client: 2 })
        );
        // resolve / chargeback without a dispute
        assert!(matches!(
            e.apply(Transaction::Resolve { client: 1, tx: 1 }),
            Err(EngineError::InvalidState { .. })
        ));
        assert!(matches!(
            e.apply(Transaction::Chargeback { client: 1, tx: 1 }),
            Err(EngineError::InvalidState { .. })
        ));
        // double dispute
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        assert!(matches!(
            e.apply(Transaction::Dispute { client: 1, tx: 1 }),
            Err(EngineError::InvalidState { .. })
        ));
        assert_eq!(state(&e, 1), s("0.0000", "10.0000", "10.0000", false));
    }

    #[test]
    fn withdrawals_cannot_be_disputed() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(wd(1, 2, "4")).unwrap();
        assert_eq!(
            e.apply(Transaction::Dispute { client: 1, tx: 2 }),
            Err(EngineError::UnknownTx(2))
        );
    }

    #[test]
    fn duplicate_deposit_id_is_ignored() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        assert_eq!(e.apply(dep(1, 1, "10")), Err(EngineError::DuplicateTx(1)));
        assert_eq!(state(&e, 1), s("10.0000", "0.0000", "10.0000", false));
    }

    #[test]
    fn charged_back_deposit_cannot_be_disputed_again() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        e.apply(Transaction::Chargeback { client: 1, tx: 1 })
            .unwrap();
        assert!(matches!(
            e.apply(Transaction::Dispute { client: 1, tx: 1 }),
            Err(EngineError::InvalidState { .. })
        ));
    }

    #[test]
    fn resolved_deposit_can_be_disputed_again() {
        let mut e = Engine::new();
        e.apply(dep(1, 1, "10")).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        e.apply(Transaction::Resolve { client: 1, tx: 1 }).unwrap();
        e.apply(Transaction::Dispute { client: 1, tx: 1 }).unwrap();
        assert_eq!(state(&e, 1), s("0.0000", "10.0000", "10.0000", false));
    }
}
