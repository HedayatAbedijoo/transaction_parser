use bigdecimal::BigDecimal;
use bigdecimal::*;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
const SCALE: i64 = 10_000;

#[derive(Debug, Clone, Copy, Default)]
/// A struct representing monetary value in the smallest currency unit (e.g., cents).
///
/// # Why Use Money? It is a Value Object.
/// Using `Money` as a wrapper around `i64` provides type safety and prevents confusion
/// with other numeric values. It makes the code more readable and prevents accidental
/// mixing of different numeric types. By storing money as an integer (in the smallest
/// unit), we avoid floating-point precision issues that can occur with monetary calculations.
///
/// # Examples
/// ```
/// use transaction_parser::common::money::Money;
///
/// let amount = Money::new(1000); // Represents 0.1000 in currency
/// assert_eq!(amount.as_i64(), 1000);
/// assert_eq!(amount.to_string_4dp(), "0.1000");
/// ```
pub struct Money(i64);

impl Money {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn from_i64(value: i64) -> Self {
        Money(value)
    }

    pub fn zero() -> Self {
        Money(0)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }

    pub fn to_string_4dp(&self) -> String {
        let bd = BigDecimal::from(self.0) / BigDecimal::from(SCALE);
        format!("{:.4}", bd)
    }
}

impl std::str::FromStr for Money {
    type Err = ParseBigDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t.is_empty() {
            return Err(ParseBigDecimalError::Other("empty amount".into()));
        }

        let bd: BigDecimal = t.parse()?;

        // Scale to 4 decimal places
        let scaled = (bd * BigDecimal::from(SCALE)).round(0);
        let value: i64 = scaled
            .to_i64()
            .ok_or_else(|| ParseBigDecimalError::Other("amount overflow".into()))?;

        Ok(Money(value))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_4dp())
    }
}

impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Money {}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Money) {
        *self = *self - rhs;
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(Money::zero(), Money(0));
    }

    #[test]
    fn test_as_i64() {
        assert_eq!(Money(12345).as_i64(), 12345);
        assert_eq!(Money::zero().as_i64(), 0);
        assert_eq!(Money(-999).as_i64(), -999);
    }

    #[test]
    fn test_from_str_valid() {
        assert_eq!(Money::from_str("1").unwrap(), Money(10000));
        assert_eq!(Money::from_str("1.5").unwrap(), Money(15000));
        assert_eq!(Money::from_str("1.2345").unwrap(), Money(12345));
        assert_eq!(Money::from_str("0.0001").unwrap(), Money(1));
        assert_eq!(Money::from_str("  2.0000 ").unwrap(), Money(20000));
    }

    #[test]
    fn test_from_str_rounding() {
        assert_eq!(Money::from_str("1.99999").unwrap(), Money(20000));
        assert_eq!(Money::from_str("0.00001").unwrap(), Money(0));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(Money::from_str("").is_err());
        assert!(Money::from_str("   ").is_err());
        assert!(Money::from_str("abc").is_err());
    }

    #[test]
    fn test_to_string_4dp() {
        assert_eq!(Money(10000).to_string_4dp(), "1.0000");
        assert_eq!(Money(12345).to_string_4dp(), "1.2345");
        assert_eq!(Money(1).to_string_4dp(), "0.0001");
        assert_eq!(Money(0).to_string_4dp(), "0.0000");
    }

    #[test]
    fn test_display() {
        assert_eq!(Money(10000).to_string(), "1.0000");
        assert_eq!(Money(5000).to_string(), "0.5000");
    }

    #[test]
    fn test_add() {
        assert_eq!(Money(10000) + Money(5000), Money(15000));
        assert_eq!(Money::zero() + Money(100), Money(100));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Money(15000) - Money(5000), Money(10000));
        assert_eq!(Money(100) - Money(100), Money::zero());
    }

    #[test]
    fn test_add_assign() {
        let mut m = Money(10000);
        m += Money(5000);
        assert_eq!(m, Money(15000));
    }

    #[test]
    fn test_sub_assign() {
        let mut m = Money(15000);
        m -= Money(5000);
        assert_eq!(m, Money(10000));
    }

    #[test]
    fn test_ordering() {
        assert!(Money(10000) < Money(15000));
        assert!(Money(15000) > Money(10000));
        assert!(Money(10000) <= Money(10000));
        assert!(Money(10000) >= Money(10000));
    }

    #[test]
    fn test_equality() {
        assert_eq!(Money(10000), Money(10000));
        assert_ne!(Money(10000), Money(5000));
    }
}
