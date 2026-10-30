//! A client account. Fields are private: the only way to change balances is
//! through methods that preserve `total == available + held`.

use crate::amount::Amount;
use crate::error::EngineError;

#[derive(Debug, Clone, Default)]
pub struct Account {
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    pub fn available(&self) -> Amount {
        self.available
    }
    pub fn held(&self) -> Amount {
        self.held
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
    /// Derived, never stored, so it can never drift from its components.
    pub fn total(&self) -> Amount {
        // Both operands came from checked operations; if their sum overflowed
        // we could never have stored them. Saturating to ZERO is unreachable.
        self.available
            .checked_add(self.held)
            .unwrap_or(Amount::ZERO)
    }

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

    fn ensure_unlocked(&self, client: u16) -> Result<(), EngineError> {
        if self.locked {
            Err(EngineError::AccountLocked(client))
        } else {
            Ok(())
        }
    }
}
