//! Fixed-precision money type.
//!
//! Floats are never used for money. `Amount` wraps [`Decimal`] (exact base-10
//! arithmetic) and only exposes *checked* operations, so overflow becomes an
//! error value instead of a panic.

use crate::error::AmountError;
use rust_decimal::Decimal;
use std::fmt;
use std::str::FromStr;

/// Maximum number of digits after the decimal point accepted on input.
const MAX_SCALE: u32 = 4;

/// A monetary value. Balances may be negative (e.g. after a dispute on funds
/// that were already withdrawn); *input* amounts are always positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Amount(pub(crate) Decimal);

impl Amount {
    /// Zero monetary amount.
    pub const ZERO: Amount = Amount(Decimal::ZERO);

    /// Parses an input amount: strictly positive, at most 4 decimal places.
    ///
    /// # Examples
    ///
    /// ```
    /// use payments_engine::amount::Amount;
    ///
    /// let amt = Amount::parse_positive(" 1.2500 ")?;
    /// assert_eq!(amt.to_string(), "1.2500");
    /// # Ok::<(), payments_engine::error::AmountError>(())
    /// ```
    pub fn parse_positive(s: &str) -> Result<Self, AmountError> {
        let value = Decimal::from_str(s.trim()).map_err(|_| AmountError::Invalid(s.to_owned()))?;
        if value <= Decimal::ZERO {
            return Err(AmountError::NotPositive);
        }
        if value.normalize().scale() > MAX_SCALE {
            return Err(AmountError::TooPrecise);
        }
        Ok(Amount(value))
    }

    /// Checked addition. Returns `None` if overflow occurs.
    pub fn checked_add(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_add(rhs.0).map(Amount)
    }

    /// Checked subtraction. Returns `None` if overflow occurs.
    pub fn checked_sub(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_sub(rhs.0).map(Amount)
    }
}

/// Always prints exactly four decimal places, e.g. `1.5000`.
impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_up_to_four_decimals() {
        assert_eq!(
            Amount::parse_positive("1.2345").unwrap().to_string(),
            "1.2345"
        );
        assert_eq!(Amount::parse_positive(" 2 ").unwrap().to_string(), "2.0000");
        assert_eq!(
            Amount::parse_positive("1.50000").unwrap().to_string(),
            "1.5000"
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(
            Amount::parse_positive("1.23456"),
            Err(AmountError::TooPrecise)
        );
        assert_eq!(Amount::parse_positive("0"), Err(AmountError::NotPositive));
        assert_eq!(Amount::parse_positive("-1"), Err(AmountError::NotPositive));
        assert!(matches!(
            Amount::parse_positive("abc"),
            Err(AmountError::Invalid(_))
        ));
        assert!(matches!(
            Amount::parse_positive(""),
            Err(AmountError::Invalid(_))
        ));
    }

    #[test]
    fn checked_ops_do_not_panic_on_overflow() {
        let big = Amount(Decimal::MAX);
        assert!(big.checked_add(big).is_none());
    }
}
