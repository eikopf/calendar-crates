//! Types representing the members of finite sets.

use strum::EnumString;

/// An iTIP method (RFC 5546 §1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum Method {
    #[strum(serialize = "PUBLISH")]
    Publish,
    #[strum(serialize = "REQUEST")]
    Request,
    #[strum(serialize = "REPLY")]
    Reply,
    #[strum(serialize = "ADD")]
    Add,
    #[strum(serialize = "CANCEL")]
    Cancel,
    #[strum(serialize = "REFRESH")]
    Refresh,
    #[strum(serialize = "COUNTER")]
    Counter,
    #[strum(serialize = "DECLINECOUNTER")]
    DeclineCounter,
}

/// An unsigned integer in the range `0..=100` (RFC 5545 §3.8.1.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percent(u8);

impl Percent {
    pub const MIN: Self = Percent(0);
    pub const MAX: Self = Percent(100);

    #[inline(always)]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn new(value: u8) -> Option<Self> {
        match value {
            0..=100 => Some(Self(value)),
            _ => None,
        }
    }
}

/// A priority value in the range `0..=9` (RFC 5545 §3.8.1.9).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    #[default]
    Zero,
    A1,
    A2,
    A3,
    B1,
    B2,
    B3,
    C1,
    C2,
    C3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityClass {
    Low,
    Medium,
    High,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self, Self::Zero) || matches!(other, Self::Zero) {
            None
        } else {
            let a = (*self) as u8;
            let b = (*other) as u8;

            match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(std::cmp::Ordering::Greater),
                std::cmp::Ordering::Equal => Some(std::cmp::Ordering::Equal),
                std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Less),
            }
        }
    }
}

impl Priority {
    pub const fn is_low(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::Low))
    }

    pub const fn is_medium(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::Medium))
    }

    pub const fn is_high(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::High))
    }

    pub const fn into_class(self) -> Option<PriorityClass> {
        match self {
            Self::Zero => None,
            Self::A1 | Self::A2 | Self::A3 | Self::B1 => Some(PriorityClass::High),
            Self::B2 => Some(PriorityClass::Medium),
            Self::B3 | Self::C1 | Self::C2 | Self::C3 => Some(PriorityClass::Low),
        }
    }
}
