//! Date and time types, largely from RFC 3339.

use std::{convert::Infallible, num::NonZero};

use thiserror::Error;

/// One of the seven weekdays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub const fn from_repr(repr: u8) -> Option<Self> {
        match repr {
            0..=6 => {
                // SAFETY: the valid discriminants of Self are exactly the
                // values of the range 0..=6.
                Some(unsafe { std::mem::transmute::<u8, Self>(repr) })
            }
            _ => None,
        }
    }

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        const VARIANTS: [Weekday; 7] = [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];

        VARIANTS.iter().copied()
    }
}

/// An ISO week ranging from W1 to W53.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IsoWeek {
    W1 = 1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
    W8,
    W9,
    W10,
    W11,
    W12,
    W13,
    W14,
    W15,
    W16,
    W17,
    W18,
    W19,
    W20,
    W21,
    W22,
    W23,
    W24,
    W25,
    W26,
    W27,
    W28,
    W29,
    W30,
    W31,
    W32,
    W33,
    W34,
    W35,
    W36,
    W37,
    W38,
    W39,
    W40,
    W41,
    W42,
    W43,
    W44,
    W45,
    W46,
    W47,
    W48,
    W49,
    W50,
    W51,
    W52,
    W53,
}

impl IsoWeek {
    pub const fn index(&self) -> NonZero<u8> {
        NonZero::new(*self as u8).unwrap()
    }

    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            1..=53 => {
                let week: Self = unsafe { std::mem::transmute(index) };
                Some(week)
            }
            _ => None,
        }
    }
}

/// A marker struct for the UTC timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;

/// A marker struct for the implicit local timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidDateTimeError {
    #[error("invalid date: {0}")]
    Date(#[from] InvalidDateError),
    #[error("invalid time: {0}")]
    Time(#[from] InvalidTimeError),
}

/// An ISO 8601 datetime with the timezone marker `M` (RFC 3339 §5.6).
///
/// This type makes no guarantees about the relationship between its fields, and in particular does
/// not guarantee that the [`time`] field represents a time that actually occurred on the date
/// represented by the [`date`] field; that is, it does not encode any information about leap
/// seconds.
///
/// [`time`]: DateTime::time
/// [`date`]: DateTime::date
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime<M> {
    pub date: Date,
    pub time: Time,
    pub marker: M,
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
    pub const fn new(year: Year, month: Month, day: Day) -> Result<Self, ImpossibleDateError> {
        if (day as u8) <= (Date::maximum_day(year, month) as u8) {
            Ok(Self { year, month, day })
        } else {
            Err(ImpossibleDateError { year, month, day })
        }
    }

    /// Returns the maximum day of `month` in `year`, based on the table given in RFC 3339 §5.7.
    pub const fn maximum_day(year: Year, month: Month) -> Day {
        match month {
            Month::Feb if year.is_leap_year() => Day::D29,
            Month::Feb => Day::D28,
            Month::Jan
            | Month::Mar
            | Month::May
            | Month::Jul
            | Month::Aug
            | Month::Oct
            | Month::Dec => Day::D31,
            Month::Apr | Month::Jun | Month::Sep | Month::Nov => Day::D30,
        }
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
    #[error(transparent)]
    ImpossibleDate(#[from] ImpossibleDateError),
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("the given date is impossible")]
pub struct ImpossibleDateError {
    year: Year,
    month: Month,
    day: Day,
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

    pub const fn is_leap_year(self) -> bool {
        let year = self.0;
        // as given by RFC 3339, Appendix C
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    }

    #[inline(always)]
    pub const fn new(value: u16) -> Result<Self, InvalidYearError> {
        if value <= 9999 {
            Ok(Year(value))
        } else {
            Err(InvalidYearError(value))
        }
    }

    #[inline(always)]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", *self as u8)
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", *self as u8)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

impl std::fmt::Display for Hour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", *self as u8)
    }
}

impl std::fmt::Display for Minute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", *self as u8)
    }
}

impl std::fmt::Display for Second {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", *self as u8)
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.hour, self.minute, self.second)?;
        if let Some(frac) = self.frac {
            // Format as ".NNN..." with trailing zeros stripped
            let nanos = frac.get().get();
            let mut s = format!("{nanos:09}");
            let trimmed = s.trim_end_matches('0');
            s.truncate(trimmed.len());
            write!(f, ".{s}")?;
        }
        Ok(())
    }
}

impl<M> std::fmt::Display for DateTime<M>
where
    M: DateTimeMarker,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}T{}{}", self.date, self.time, M::SUFFIX)
    }
}

/// Marker trait for timezone markers in [`DateTime`] formatting.
pub trait DateTimeMarker {
    const SUFFIX: &'static str;
}

impl DateTimeMarker for Utc {
    const SUFFIX: &'static str = "Z";
}

impl DateTimeMarker for Local {
    const SUFFIX: &'static str = "";
}

impl DateTimeMarker for () {
    const SUFFIX: &'static str = "";
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

    /// Returns the month number of `self`, which lies in the range `1..=12`.
    pub const fn number(self) -> NonZero<u8> {
        // SAFETY: the value of (self as u8) can never be zero
        unsafe { NonZero::new_unchecked(self as u8) }
    }

    pub fn iter() -> impl ExactSizeIterator<Item = Month> {
        [
            Self::Jan,
            Self::Feb,
            Self::Mar,
            Self::Apr,
            Self::May,
            Self::Jun,
            Self::Jul,
            Self::Aug,
            Self::Sep,
            Self::Oct,
            Self::Nov,
            Self::Dec,
        ]
        .into_iter()
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

impl Time {
    pub const fn new(
        hour: Hour,
        minute: Minute,
        second: Second,
        frac: Option<FractionalSecond>,
    ) -> Result<Self, InvalidTimeError> {
        // refer to RFC 3339 §5.7 for details about when leap seconds are valid. for now, we're
        // just going to unconditionally construct a Time

        Ok(Self {
            hour,
            minute,
            second,
            frac,
        })
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidTimeError {
    #[error("invalid hour: {0}")]
    Hour(#[from] InvalidHourError),
    #[error("invalid minute: {0}")]
    Minute(#[from] InvalidMinuteError),
    #[error("invalid second: {0}")]
    Second(#[from] InvalidSecondError),
    #[error("invalid fractional second: {0}")]
    FractionalSecond(#[from] InvalidFractionalSecondError),
}

impl From<Infallible> for InvalidTimeError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
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

impl Hour {
    pub const fn new(value: u8) -> Result<Self, InvalidHourError> {
        match NonZero::new(value) {
            None => Ok(Self::H00),
            Some(value) => match value.get() <= 23 {
                false => Err(InvalidHourError(value)),
                true => Ok({
                    // SAFETY: `value` must be less than 24 in this branch, so it is a valid hour,
                    // and Hour is repr(u8)
                    unsafe { std::mem::transmute::<u8, Hour>(value.get()) }
                }),
            },
        }
    }
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

impl Minute {
    pub const fn new(value: u8) -> Result<Self, InvalidMinuteError> {
        match NonZero::new(value) {
            None => Ok(Self::M00),
            Some(value) => match value.get() <= 59 {
                false => Err(InvalidMinuteError(value)),
                true => Ok({
                    // SAFETY: `value` must be less than 59 in this branch, so it is a valid minute,
                    // and Minute is repr(u8)
                    unsafe { std::mem::transmute::<u8, Minute>(value.get()) }
                }),
            },
        }
    }
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

impl Second {
    pub const fn new(value: u8) -> Result<Self, InvalidSecondError> {
        match NonZero::new(value) {
            None => Ok(Self::S00),
            Some(value) => match value.get() <= 60 {
                false => Err(InvalidSecondError(value)),
                true => Ok({
                    // SAFETY: `value` must be less than 60 in this branch, so it is a valid second,
                    // and Second is repr(u8)
                    unsafe { std::mem::transmute::<u8, Second>(value.get()) }
                }),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 0 and 60 but received {0}")]
pub struct InvalidSecondError(NonZero<u8>);

/// One of the 60 seconds in a minute which are not leap seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum NonLeapSecond {
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
}

impl NonLeapSecond {
    pub const fn new(value: u8) -> Result<Self, InvalidNonLeapSecondError> {
        match NonZero::new(value) {
            None => Ok(Self::S00),
            Some(value) => match value.get() < 60 {
                false => Err(InvalidNonLeapSecondError(value)),
                true => Ok({
                    // SAFETY: `value` must be less than 60 in this branch, so it is a valid second
                    // and not a leap second, and NonLeapSecond is repr(u8)
                    unsafe { std::mem::transmute::<u8, NonLeapSecond>(value.get()) }
                }),
            },
        }
    }

    #[inline(always)]
    pub const fn to_second(self) -> Second {
        match Second::new(self as u8) {
            Ok(second) => second,
            Err(_) => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("expected an integer between 0 and 59 but received {0}")]
pub struct InvalidNonLeapSecondError(NonZero<u8>);

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

    #[inline(always)]
    pub const fn get(self) -> NonZero<u32> {
        self.0
    }

    pub const fn new(value: u32) -> Result<Self, InvalidFractionalSecondError> {
        match NonZero::new(value) {
            None => Err(InvalidFractionalSecondError::AllZero),
            Some(value) => match value.get() <= Self::MAX.0.get() {
                true => Ok(Self(value)),
                false => Err(InvalidFractionalSecondError::TooManyDigits(value)),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum InvalidFractionalSecondError {
    #[error("at least one fractional second digit must be non-zero")]
    AllZero,
    #[error("{0} has more than nine decimal digits")]
    TooManyDigits(NonZero<u32>),
}
