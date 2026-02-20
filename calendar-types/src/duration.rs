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

impl std::fmt::Display for ExactDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hours > 0 {
            write!(f, "{}H", self.hours)?;
        }
        if self.minutes > 0 || (self.hours > 0 && (self.seconds > 0 || self.frac.is_some())) {
            write!(f, "{}M", self.minutes)?;
        }
        if self.seconds > 0 || self.frac.is_some() {
            write!(f, "{}", self.seconds)?;
            if let Some(frac) = self.frac {
                let nanos = frac.get().get();
                let mut s = format!("{nanos:09}");
                let trimmed = s.trim_end_matches('0');
                s.truncate(trimmed.len());
                write!(f, ".{s}")?;
            }
            write!(f, "S")?;
        }
        // Handle the zero case: if nothing was written, write "0S"
        if self.hours == 0 && self.minutes == 0 && self.seconds == 0 && self.frac.is_none() {
            write!(f, "0S")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for NominalDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.weeks > 0 {
            write!(f, "{}W", self.weeks)?;
        }
        if self.days > 0 {
            write!(f, "{}D", self.days)?;
        }
        if let Some(exact) = &self.exact {
            write!(f, "T{exact}")?;
        }
        // Handle the zero case
        if self.weeks == 0 && self.days == 0 && self.exact.is_none() {
            write!(f, "0D")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P")?;
        match self {
            Duration::Nominal(n) => write!(f, "{n}"),
            Duration::Exact(e) => write!(f, "T{e}"),
        }
    }
}

impl std::fmt::Display for SignedDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sign {
            Sign::Neg => write!(f, "-{}", self.duration),
            Sign::Pos => write!(f, "{}", self.duration),
        }
    }
}
