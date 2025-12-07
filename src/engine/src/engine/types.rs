use rust_decimal::prelude::ToPrimitive;
use std::fmt::Display;
use std::ops::Mul;

macro_rules! wrapper_type {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name($inner);

        impl From<$inner> for $name {
            fn from(id: $inner) -> Self {
                $name(id)
            }
        }

        impl $name {
            pub fn as_inner(&self) -> $inner {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_inner())
            }
        }
    };
}

wrapper_type!(ClientId, u16);
wrapper_type!(TransactionId, u32);

/// Represents a monetary amount.
///
/// Can be constructed from minor units (e.g., cents) using `from_minor`.
/// Or from a [rust_decimal::Decimal] directly - which allows negative amount.
///
/// Internaly uses `rust_decimal::Decimal` for precise decimal representation
/// and stores upto 4 decimal places.
///
/// Exposes `checked_sub` method to safely subtract another amount without going negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(rust_decimal::Decimal);

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl Amount {
    pub fn from_minor(value: u32) -> Self {
        Amount(rust_decimal::Decimal::new(value as _, 2))
    }

    pub fn as_decimal(&self) -> rust_decimal::Decimal {
        self.0
    }

    /// Returns the amount in minor units (e.g., cents) as u32.
    /// Returns None if the amount is too large to fit u32 (> ~4.29 billion)
    /// or is negative amount.
    pub fn in_minor(&self) -> Option<u32> {
        self.0.to_f32().and_then(|major| major.mul(100.0).to_u32())
    }

    /// Subtract another amount (in-place) from this one only if resulting amount does not go below 0.
    /// Returns None if the subtraction would result in a negative value.
    pub fn try_subtract(&mut self, other: Amount) -> Option<()> {
        self.0.checked_sub(other.0).and_then(|value| {
            if value.is_sign_negative() {
                None
            } else {
                self.0 = value;
                Some(())
            }
        })
    }
}

impl Default for Amount {
    fn default() -> Self {
        Amount(rust_decimal::Decimal::ZERO)
    }
}

impl From<rust_decimal::Decimal> for Amount {
    fn from(value: rust_decimal::Decimal) -> Self {
        Amount(value.round_dp(4))
    }
}

impl std::ops::AddAssign for Amount {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

#[cfg(test)]
mod amount_tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_amount_try_sub() {
        let mut a1 = Amount::from_minor(150); // 1.50
        let a2 = Amount::from_minor(50); // 0.50

        a1.try_subtract(a2).expect("Subtraction should succeed");
        assert_eq!(a1.as_decimal(), Decimal::new(100, 2)); //
    }

    #[test]
    fn amount_in_minor() {
        let a1 = Amount::from_minor(150); // 1.50

        assert_eq!(a1.in_minor().unwrap(), 150);
    }

    #[test]
    fn try_subtract_prevent_negative() {
        let mut a1 = Amount::from_minor(1); // 0.01
        assert!(a1.try_subtract(Amount::from_minor(2)).is_none());

        assert_eq!(a1.in_minor().unwrap(), 1);
    }

    #[test]
    fn display_upto_4_decimals() {
        let a1 = Amount::from(rust_decimal::Decimal::new(123456, 4)); // 0.01

        assert_eq!(a1.to_string(), "12.3456");
    }
}
