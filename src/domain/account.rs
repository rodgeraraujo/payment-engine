//! A client account. Fields are private: the only way to change balances is
//! through methods that preserve `total == available + held`.

use crate::domain::amount::Amount;
use crate::error::EngineError;

/// An account tracking client funds and locked status.
#[derive(Debug, Clone, Default)]
pub struct Account {
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    /// Available funds for trading or withdrawal.
    pub fn available(&self) -> Amount {
        self.available
    }

    /// Funds held due to open disputes.
    pub fn held(&self) -> Amount {
        self.held
    }

    /// Whether the account has been frozen following a chargeback.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Derived, never stored, so it can never drift from its components.
    /// Returns `None` on arithmetic overflow rather than silently falling back.
    pub fn total(&self) -> Option<Amount> {
        self.available.checked_add(self.held)
    }

    /// Deposits funds into the available balance of the account.
    pub fn deposit(&mut self, client: u16, amount: Amount) -> Result<(), EngineError> {
        self.ensure_unlocked(client)?;
        let available = self
            .available
            .checked_add(amount)
            .ok_or(EngineError::Overflow(client))?;
        // Guard the derived total as well.
        available
            .checked_add(self.held)
            .ok_or(EngineError::Overflow(client))?;
        self.available = available;
        Ok(())
    }

    /// Withdraws funds from the available balance if sufficient funds exist.
    pub fn withdraw(&mut self, client: u16, tx: u32, amount: Amount) -> Result<(), EngineError> {
        self.ensure_unlocked(client)?;
        if self.available < amount {
            return Err(EngineError::InsufficientFunds { client, tx });
        }
        self.available = self
            .available
            .checked_sub(amount)
            .ok_or(EngineError::Overflow(client))?;
        Ok(())
    }

    /// Dispute: move funds from available to held. Total is unchanged.
    /// `available` may go negative if the funds were already spent.
    pub fn hold(&mut self, client: u16, amount: Amount) -> Result<(), EngineError> {
        let available = self
            .available
            .checked_sub(amount)
            .ok_or(EngineError::Overflow(client))?;
        let held = self
            .held
            .checked_add(amount)
            .ok_or(EngineError::Overflow(client))?;
        self.available = available;
        self.held = held;
        Ok(())
    }

    /// Resolve: move funds from held back to available. Total is unchanged.
    pub fn release(&mut self, client: u16, amount: Amount) -> Result<(), EngineError> {
        let available = self
            .available
            .checked_add(amount)
            .ok_or(EngineError::Overflow(client))?;
        let held = self
            .held
            .checked_sub(amount)
            .ok_or(EngineError::Overflow(client))?;
        self.available = available;
        self.held = held;
        Ok(())
    }

    /// Chargeback: held funds leave the account and the account is frozen.
    pub fn chargeback(&mut self, client: u16, amount: Amount) -> Result<(), EngineError> {
        self.held = self
            .held
            .checked_sub(amount)
            .ok_or(EngineError::Overflow(client))?;
        self.locked = true;
        Ok(())
    }

    /// Checks whether the account is unlocked.
    ///
    /// Assumption (I9): A chargeback freezes an account (`locked = true`). We follow
    /// a bank-freeze model: new deposits and withdrawals are rejected, whereas
    /// dispute lifecycle events (dispute, resolve, chargeback) on previously recorded
    /// transactions can still conclude. Centralizing the policy here ensures that
    /// changing this assumption is a single-line modification.
    fn ensure_unlocked(&self, client: u16) -> Result<(), EngineError> {
        if self.locked {
            Err(EngineError::AccountLocked(client))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn total_returns_none_on_overflow() {
        let account = Account {
            available: Amount(Decimal::MAX),
            held: Amount(Decimal::ONE),
            locked: false,
        };
        // Explicit Option avoids silent zero fallback.
        assert_eq!(account.total(), None);
    }
}
