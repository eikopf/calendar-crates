//! Date and time data model types.

use std::num::NonZero;

use thiserror::Error;

use crate::model::primitive::Sign;

/// A marker struct for the UTC timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;

/// A marker struct for the implicit local timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local;

/// A [`DateTime`] in the [`Utc`] timezone (RFC 8984 §1.4.4).
pub type UtcDateTime = DateTime<Utc>;

/// A [`DateTime`] in the implicit [`Local`] timezone (RFC 8984 §1.4.4).
pub type LocalDateTime = DateTime<Local>;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidDateTimeError<F> {
    #[error("invalid date: {0}")]
    Date(#[from] InvalidDateError),
    #[error("invalid time: {0}")]
    Time(#[from] InvalidTimeError),
    #[error("invalid offset marker")]
    Marker(F),
}

/// An ISO 8601 datetime with the timezone format `F` (RFC 3339 §5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime<F> {
    date: Date,
    time: Time,
    format: F,
}

/// An ISO 8601 date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: Year,
    month: Month,
    day: Day,
}

impl Date {
    #[inline(always)]
    pub const fn new(year: Year, month: Month, day: Day) -> Result<Self, InvalidDateError> {
        Ok(Self { year, month, day })
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidDateError {
    #[error("invalid year: {0}")]
    Year(#[from] InvalidYearError),
    #[error("invalid month: {0}")]
    Month(#[from] InvalidMonthError),
    #[error("invalid day: {0}")]
    Day(#[from] InvalidDayError),
}

/// A four-digit year ranging from 0 CE through 9999 CE.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Year(u16);

impl std::fmt::Debug for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.0 <= 9999);
        write!(f, "{:04} CE", self.0)
    }
}

impl Year {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(9999);

    #[inline(always)]
    pub const fn new(value: u16) -> Result<Self, InvalidYearError> {
        if value <= 9999 {
            Ok(Year(value))
        } else {
            Err(InvalidYearError(value))
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer of at most 9999 but received {0} instead")]
pub struct InvalidYearError(u16);

/// One of the twelve Gregorian months.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Month {
    Jan = 1,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    pub const fn new(value: u8) -> Result<Self, InvalidMonthError> {
        match value {
            1..=12 => Ok({
                // SAFETY: Month is repr(u8) and takes the values in the range 1..=12, which are
                // the only possible values in this branch
                unsafe { std::mem::transmute::<u8, Month>(value) }
            }),
            _ => Err(InvalidMonthError(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 1 and 12 but received {0} instead")]
pub struct InvalidMonthError(u8);

/// One of the 31 days of the Gregorian month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Day {
    D01 = 1,
    D02,
    D03,
    D04,
    D05,
    D06,
    D07,
    D08,
    D09,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
    D16,
    D17,
    D18,
    D19,
    D20,
    D21,
    D22,
    D23,
    D24,
    D25,
    D26,
    D27,
    D28,
    D29,
    D30,
    D31,
}

impl Day {
    #[inline(always)]
    pub const fn new(value: u8) -> Result<Self, InvalidDayError> {
        match value {
            1..=31 => Ok({
                // SAFETY: Day is repr(u8) and takes the values in the range 1..=31, which are
                // the only possible values in this branch
                unsafe { std::mem::transmute::<u8, Self>(value) }
            }),
            _ => Err(InvalidDayError(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 1 and 31 but received {0} instead")]
pub struct InvalidDayError(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: Hour,
    minute: Minute,
    second: Second,
    frac: Option<FractionalSecond>,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidTimeError {
    #[error("invalid hour: {0}")]
    Hour(#[from] InvalidHourError),
    #[error("invalid minute: {0}")]
    Minute(#[from] InvalidMinuteError),
    #[error("invalid second: {0}")]
    Second(#[from] InvalidSecondError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Hour {
    #[default]
    H00,
    H01,
    H02,
    H03,
    H04,
    H05,
    H06,
    H07,
    H08,
    H09,
    H10,
    H11,
    H12,
    H13,
    H14,
    H15,
    H16,
    H17,
    H18,
    H19,
    H20,
    H21,
    H22,
    H23,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 0 and 23 but received {0}")]
pub struct InvalidHourError(NonZero<u8>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Minute {
    #[default]
    M00,
    M01,
    M02,
    M03,
    M04,
    M05,
    M06,
    M07,
    M08,
    M09,
    M10,
    M11,
    M12,
    M13,
    M14,
    M15,
    M16,
    M17,
    M18,
    M19,
    M20,
    M21,
    M22,
    M23,
    M24,
    M25,
    M26,
    M27,
    M28,
    M29,
    M30,
    M31,
    M32,
    M33,
    M34,
    M35,
    M36,
    M37,
    M38,
    M39,
    M40,
    M41,
    M42,
    M43,
    M44,
    M45,
    M46,
    M47,
    M48,
    M49,
    M50,
    M51,
    M52,
    M53,
    M54,
    M55,
    M56,
    M57,
    M58,
    M59,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 0 and 59 but received {0}")]
pub struct InvalidMinuteError(NonZero<u8>);

/// One of the 61 possible seconds in a minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Second {
    #[default]
    S00,
    S01,
    S02,
    S03,
    S04,
    S05,
    S06,
    S07,
    S08,
    S09,
    S10,
    S11,
    S12,
    S13,
    S14,
    S15,
    S16,
    S17,
    S18,
    S19,
    S20,
    S21,
    S22,
    S23,
    S24,
    S25,
    S26,
    S27,
    S28,
    S29,
    S30,
    S31,
    S32,
    S33,
    S34,
    S35,
    S36,
    S37,
    S38,
    S39,
    S40,
    S41,
    S42,
    S43,
    S44,
    S45,
    S46,
    S47,
    S48,
    S49,
    S50,
    S51,
    S52,
    S53,
    S54,
    S55,
    S56,
    S57,
    S58,
    S59,
    S60,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 0 and 60 but received {0}")]
pub struct InvalidSecondError(NonZero<u8>);

/// A non-zero fractional second, represented as an integer multiple of nanoseconds. This
/// guarantees nine digits of decimal precision and a maximum error of 10^-9.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FractionalSecond(NonZero<u32>);

impl std::fmt::Debug for FractionalSecond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ns", self.0.get())
    }
}

impl FractionalSecond {
    /// The smallest fractional second; this value is exactly 1 nanosecond.
    pub const MIN: Self = Self(NonZero::new(1).unwrap());
    /// The largest fractional second; this value is 10^9 - 1 nanoseconds.
    pub const MAX: Self = Self(NonZero::new(10u32.pow(9) - 1).unwrap());
}

/// An unsigned length of time (RFC 8984 §1.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Duration {
    Nominal(NominalDuration),
    Exact(ExactDuration),
}

/// A [`Duration`] which may be positive or negative (RFC 8984 §1.4.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedDuration {
    sign: Sign,
    duration: Duration,
}

/// A [`Duration`] measured in terms of weeks, days, hours, minutes, seconds, and fractional
/// seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NominalDuration {
    inner: NominalDurationInner,
    exact: Option<ExactDuration>,
}

/// The nominal component of a [`NominalDuration`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NominalDurationInner {
    Days(u32),
    Weeks(u32),
    WeeksAndDays(u32, u32),
}

/// A [`Duration`] measured only in terms of hours, minutes, seconds, and fractional seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExactDuration {
    hours: u32,
    minutes: u32,
    seconds: u32,
    frac: Option<FractionalSecond>,
}
