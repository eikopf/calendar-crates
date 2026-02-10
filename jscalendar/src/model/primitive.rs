//! Primitive data model types.

use crate::json::{
    ConstructibleJsonValue, DestructibleJsonValue, IntoIntError, IntoJson, IntoUnsignedIntError,
    TryFromJson, TypeErrorOr,
};

/// A signed integer in the inclusive range `[-2^53 + 1, 2^53 - 1]` (RFC 8984 §1.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Int(i64);

impl Int {
    pub const MIN: Self = Int(-(1 << 53) + 1);
    pub const MAX: Self = Int((1 << 53) - 1);

    #[inline(always)]
    pub const fn new(value: i64) -> Option<Self> {
        match Self::MIN.get() <= value && value <= Self::MAX.get() {
            true => Some(Self(value)),
            false => None,
        }
    }

    #[inline(always)]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl IntoJson for Int {
    fn into_json<V: ConstructibleJsonValue>(self) -> V {
        V::int(self)
    }
}

impl TryFromJson for Int {
    type Error = TypeErrorOr<IntoIntError>;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
        value.try_as_int()
    }
}

/// An unsigned integer in the inclusive range `[0, 2^53 - 1]` (RFC 8984 §1.4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct UnsignedInt(u64);

impl UnsignedInt {
    pub const MIN: Self = UnsignedInt(0);
    pub const MAX: Self = UnsignedInt((1 << 53) - 1);

    #[inline(always)]
    pub const fn new(value: u64) -> Option<Self> {
        match Self::MIN.get() <= value && value <= Self::MAX.get() {
            true => Some(Self(value)),
            false => None,
        }
    }

    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl IntoJson for UnsignedInt {
    fn into_json<V: ConstructibleJsonValue>(self) -> V {
        V::unsigned_int(self)
    }
}

impl TryFromJson for UnsignedInt {
    type Error = TypeErrorOr<IntoUnsignedIntError>;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
        value.try_as_unsigned_int()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde_json")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn int_from_serde_json() {
        use serde_json::Value;

        use crate::json::{TypeError, ValueType};

        let parse = |s| Int::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert_eq!(parse("0"), Ok(Int::new(0).unwrap()));
        assert_eq!(parse("-9007199254740991"), Ok(Int::MIN));
        assert_eq!(parse("9007199254740991"), Ok(Int::MAX));

        assert_eq!(
            parse("2.718281"),
            Err(TypeErrorOr::Other(IntoIntError::NotAnInteger(2.718281)))
        );

        // Int::MIN - 1
        assert_eq!(
            parse("-9007199254740992"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeSigned(
                -9007199254740992
            )))
        );
        // Int::MAX + 1
        assert_eq!(
            parse("9007199254740992"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeSigned(
                9007199254740992
            )))
        );
        // u64::MAX
        assert_eq!(
            parse("18446744073709551615"),
            Err(TypeErrorOr::Other(IntoIntError::OutsideRangeUnsigned(
                u64::MAX
            )))
        );

        assert_eq!(
            parse("true"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Bool
            }
            .into())
        );

        assert_eq!(
            parse("{}"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Object
            }
            .into())
        );
    }

    #[cfg(feature = "serde_json")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn unsigned_int_from_serde_json() {
        use serde_json::Value;

        use crate::json::{TypeError, ValueType};

        let parse = |s| UnsignedInt::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        assert_eq!(parse("0"), Ok(UnsignedInt::MIN));
        assert_eq!(parse("9007199254740991"), Ok(UnsignedInt::MAX));

        assert_eq!(
            parse("-1"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::NegativeInteger(
                -1
            )))
        );
        assert_eq!(
            parse("3.141592"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::NotAnInteger(
                3.141592
            )))
        );
        // UnsignedInt::MAX + 1
        assert_eq!(
            parse("9007199254740992"),
            Err(TypeErrorOr::Other(IntoUnsignedIntError::OutsideRange(
                9007199254740992
            )))
        );

        assert_eq!(
            parse("false"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Bool
            }
            .into())
        );

        assert_eq!(
            parse("[]"),
            Err(TypeError {
                expected: ValueType::Number,
                received: ValueType::Array
            }
            .into())
        );
    }
}
