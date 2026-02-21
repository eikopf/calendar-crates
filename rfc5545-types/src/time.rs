//! Basic time types.

use calendar_types::{
    duration::{Duration, SignedDuration},
    primitive::Sign,
    time::{Date, DateTime, Hour, Minute, NonLeapSecond, Utc},
};

pub use calendar_types::time::TimeFormat;

/// Either a full datetime or a date-only value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<M = TimeFormat> {
    /// A full datetime value.
    DateTime(DateTime<M>),
    /// A date-only value.
    Date(Date),
}

impl<M> DateTimeOrDate<M> {
    /// Returns `true` if this is a date-only value.
    pub fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// Returns `true` if this is a full datetime value.
    pub fn is_date_time(&self) -> bool {
        matches!(self, Self::DateTime(_))
    }

    /// Converts the marker type of the inner datetime.
    pub fn map_marker<N>(self, f: impl FnOnce(M) -> N) -> DateTimeOrDate<N> {
        match self {
            Self::DateTime(dt) => DateTimeOrDate::DateTime(DateTime {
                date: dt.date,
                time: dt.time,
                marker: f(dt.marker),
            }),
            Self::Date(d) => DateTimeOrDate::Date(d),
        }
    }
}

/// An offset from UTC to some local time (RFC 5545 §3.3.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset {
    /// The sign of the offset (positive = east of UTC).
    pub sign: Sign,
    /// The hour component of the offset.
    pub hour: Hour,
    /// The minute component of the offset.
    pub minute: Minute,
    /// The second component of the offset.
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
    /// A period defined by explicit start and end datetimes.
    Explicit {
        /// The start of the period.
        start: DateTime<M>,
        /// The end of the period.
        end: DateTime<M>,
    },
    /// A period defined by a start datetime and a duration.
    Start {
        /// The start of the period.
        start: DateTime<M>,
        /// The duration of the period.
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
    /// A duration offset relative to the event start or end.
    Duration(SignedDuration),
    /// An absolute UTC datetime.
    DateTime(DateTime<Utc>),
}
