//! Primitive data model types.

use crate::json::{
    ConstructibleJsonValue, DestructibleJsonValue, IntoIntError, IntoJson, IntoUnsignedIntError,
    TryFromJson,
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
    type Error = IntoIntError;

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
    type Error = IntoUnsignedIntError;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error> {
        value.try_as_unsigned_int()
    }
}

/// A numeric sign, which may be either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Sign {
    Neg,
    #[default]
    Pos,
}

impl Sign {
    pub const fn as_char(self) -> char {
        match self {
            Sign::Neg => '-',
            Sign::Pos => '+',
        }
    }
}
