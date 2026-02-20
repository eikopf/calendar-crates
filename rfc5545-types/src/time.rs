//! Basic time types.

use calendar_types::{
    duration::{Duration, SignedDuration},
    primitive::Sign,
    time::{Date, DateTime, Hour, Minute, NonLeapSecond, Utc},
};

pub use calendar_types::time::TimeFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<M = TimeFormat> {
    DateTime(DateTime<M>),
    Date(Date),
}

impl<M> DateTimeOrDate<M> {
    pub fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    pub fn is_date_time(&self) -> bool {
        matches!(self, Self::DateTime(_))
    }
}

/// An offset from UTC to some local time (RFC 5545 §3.3.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset {
    pub sign: Sign,
    pub hour: Hour,
    pub minute: Minute,
    pub second: NonLeapSecond,
}

impl std::fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:02}:{:02}",
            self.sign.as_char(),
            self.hour as u8,
            self.minute as u8
        )?;
        let sec = self.second as u8;
        if sec != 0 {
            write!(f, ":{sec:02}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Period
// ============================================================================

/// A period of time (RFC 5545 §3.3.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period<M = TimeFormat> {
    Explicit {
        start: DateTime<M>,
        end: DateTime<M>,
    },
    Start {
        start: DateTime<M>,
        duration: Duration,
    },
}

// ============================================================================
// RDate / ExDate sequences
// ============================================================================

/// A single RDATE value (RFC 5545 §3.8.5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RDate<M = TimeFormat> {
    DateTime(DateTime<M>),
    Date(Date),
    Period(Period<M>),
}

/// A homogeneous sequence of RDATE values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RDateSeq<M = TimeFormat> {
    DateTime(Vec<DateTime<M>>),
    Date(Vec<Date>),
    Period(Vec<Period<M>>),
}

/// A homogeneous sequence of EXDATE values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExDateSeq<M = TimeFormat> {
    DateTime(Vec<DateTime<M>>),
    Date(Vec<Date>),
}

// ============================================================================
// TriggerValue
// ============================================================================

/// The value of a TRIGGER property (RFC 5545 §3.8.6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerValue {
    Duration(SignedDuration),
    DateTime(DateTime<Utc>),
}
