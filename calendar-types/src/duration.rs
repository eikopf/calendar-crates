//! RFC 5545 duration types.

use thiserror::Error;

use crate::{
    primitive::Sign,
    time::{FractionalSecond, InvalidFractionalSecondError},
};

/// An unsigned length of time (RFC 8984 §1.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Duration {
    Nominal(NominalDuration),
    Exact(ExactDuration),
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidDurationError {
    #[error("invalid fractional second: {0}")]
    FractionalSecond(#[from] InvalidFractionalSecondError),
}

/// A [`Duration`] which may be positive or negative (RFC 8984 §1.4.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedDuration {
    pub sign: Sign,
    pub duration: Duration,
}

impl From<Duration> for SignedDuration {
    fn from(value: Duration) -> Self {
        Self {
            sign: Default::default(),
            duration: value,
        }
    }
}

/// A [`Duration`] measured in terms of weeks, days, hours, minutes, seconds, and fractional
/// seconds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NominalDuration {
    pub weeks: u32,
    pub days: u32,
    pub exact: Option<ExactDuration>,
}

/// A [`Duration`] measured only in terms of hours, minutes, seconds, and fractional seconds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExactDuration {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frac: Option<FractionalSecond>,
}
