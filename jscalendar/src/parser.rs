//! Parsers for types which are encoded as strings by JSCalendar.
//!
//! All parsers in this module use [winnow](https://docs.rs/winnow) and are generic over the error
//! type `E`, requiring `ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>`.
//! At the public API boundary, [`parse_full`] instantiates these parsers with
//! [`ContextError`](winnow::error::ContextError) and converts the result into an
//! [`OwnedParseError`].

use calendar_types::{
    duration::{Duration, ExactDuration, InvalidDurationError, NominalDuration, SignedDuration},
    primitive::Sign,
    time::{
        Date, DateTime, Day, FractionalSecond, Hour, InvalidDateError, InvalidDateTimeError,
        InvalidDayError, InvalidFractionalSecondError, InvalidHourError, InvalidMinuteError,
        InvalidMonthError, InvalidSecondError, InvalidTimeError, InvalidYearError, Local, Minute,
        Month, Second, Time, Utc, Year,
    },
};
use thiserror::Error;
use winnow::{
    Parser,
    combinator::{alt, opt, preceded, terminated},
    error::{ContextError, FromExternalError, ParserError},
    stream::Stream,
    token::{any, one_of, take_while},
};

/// Converts an incremental parser into a complete parser, which will return an error if the input
/// string is not completely consumed.
pub fn parse_full<'i, T>(
    mut parser: impl Parser<&'i str, T, ContextError> + 'i,
) -> impl FnOnce(&'i str) -> Result<T, OwnedParseError> {
    move |input| {
        parser
            .parse(input)
            .map_err(|e| OwnedParseError::from_winnow(e))
    }
}

/// A unified error type for all domain-specific parse errors in JSCalendar.
///
/// This covers both syntactic validation errors (like structural constraints on durations) and
/// semantic errors (like out-of-range values from calendar-types constructors). Simple mismatches
/// (e.g. expected 'P' but got 'X') are handled by winnow's native backtracking and do not appear
/// here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum JsCalendarParseError {
    // Duration syntax
    /// The `T` prefix in a duration was followed by no time components.
    #[error("expected some data after T")]
    NoDataAfterTimePrefix,
    /// A duration has hours and seconds but no minutes component.
    #[error("exact time contains hours and seconds but not minutes")]
    HourAndSecondWithoutMinute,
    // Fractional second syntax
    /// A fractional second had trailing zeros (e.g. `.100`).
    #[error("trailing zeros in fractional second")]
    FractionalSecondTrailingZeros,
    /// A fractional second had more than 9 digits.
    #[error("more than 9 digits in fractional second")]
    FractionalSecondTooManyDigits,
    // Semantic errors (from calendar-types)
    /// An invalid year value.
    #[error(transparent)]
    InvalidYear(#[from] InvalidYearError),
    /// An invalid month value.
    #[error(transparent)]
    InvalidMonth(#[from] InvalidMonthError),
    /// An invalid day value.
    #[error(transparent)]
    InvalidDay(#[from] InvalidDayError),
    /// An invalid date (e.g. February 30).
    #[error(transparent)]
    InvalidDate(#[from] InvalidDateError),
    /// An invalid hour value.
    #[error(transparent)]
    InvalidHour(#[from] InvalidHourError),
    /// An invalid minute value.
    #[error(transparent)]
    InvalidMinute(#[from] InvalidMinuteError),
    /// An invalid second value.
    #[error(transparent)]
    InvalidSecond(#[from] InvalidSecondError),
    /// An invalid time value.
    #[error(transparent)]
    InvalidTime(#[from] InvalidTimeError),
    /// An invalid date-time value.
    #[error(transparent)]
    InvalidDateTime(#[from] InvalidDateTimeError),
    /// An invalid fractional second value.
    #[error(transparent)]
    InvalidFractionalSecond(#[from] InvalidFractionalSecondError),
    /// An invalid duration value.
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDurationError),
}

/// A parse error with an owned copy of the complete input string.
///
/// # Note
///
/// This type is a placeholder: its representation is likely to change before the 1.0 release.
/// See <https://github.com/eikopf/calendar-crates/issues/25> for details.
// TODO(#25): refine this error type before 1.0
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", match &self.kind {
    Some(e) => format!("{e} (at index {index} of {complete_input:?})", index = self.offset, complete_input = self.complete_input),
    None => format!("parse error at index {index} of {complete_input:?}", index = self.offset, complete_input = self.complete_input),
})]
pub struct OwnedParseError {
    complete_input: Box<str>,
    offset: usize,
    kind: Option<JsCalendarParseError>,
}

impl OwnedParseError {
    fn from_winnow(e: winnow::error::ParseError<&str, ContextError>) -> Self {
        let complete_input: Box<str> = (*e.input()).into();
        let offset = e.offset();

        // Try to extract a JsCalendarParseError from the ContextError's cause.
        let kind = e
            .into_inner()
            .cause()
            .and_then(|c| c.downcast_ref::<JsCalendarParseError>())
            .copied();

        Self {
            complete_input,
            offset,
            kind,
        }
    }

    /// Returns the byte offset at which the error occurred.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the domain-specific error kind, if one is available.
    pub fn kind(&self) -> Option<&JsCalendarParseError> {
        self.kind.as_ref()
    }
}

// impl std::error::Error for JsCalendarParseError to satisfy ContextError's FromExternalError bound
// (this is done by thiserror's #[derive(Error)])

// ---------------------------------------------------------------------------
// Private combinators
// ---------------------------------------------------------------------------

/// Parses a single ASCII decimal digit, returning its numeric value (0-9).
fn digit<'i, E>(input: &mut &'i str) -> Result<u8, E>
where
    E: ParserError<&'i str>,
{
    any.verify_map(|c: char| c.to_digit(10).map(|d| d as u8))
        .parse_next(input)
}

/// Parses a u32 from one or more ASCII digits.
fn parse_u32<'i, E>(input: &mut &'i str) -> Result<u32, E>
where
    E: ParserError<&'i str>,
{
    take_while(1.., |c: char| c.is_ascii_digit())
        .verify_map(|s: &str| s.parse::<u32>().ok())
        .parse_next(input)
}

// ---------------------------------------------------------------------------
// Component parsers (date/time primitives)
// ---------------------------------------------------------------------------

/// Parses a [`Year`] (four digits).
fn year<'i, E>(input: &mut &'i str) -> Result<Year, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b, c, d) = (digit, digit, digit, digit).parse_next(input)?;
    let value = (a as u16) * 1000 + (b as u16) * 100 + (c as u16) * 10 + d as u16;
    Year::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses a [`Month`] (two digits).
fn month<'i, E>(input: &mut &'i str) -> Result<Month, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b) = (digit, digit).parse_next(input)?;
    let value = a * 10 + b;
    Month::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses a [`Day`] (two digits).
fn day<'i, E>(input: &mut &'i str) -> Result<Day, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b) = (digit, digit).parse_next(input)?;
    let value = a * 10 + b;
    Day::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses an [`Hour`] (two digits).
fn hour<'i, E>(input: &mut &'i str) -> Result<Hour, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b) = (digit, digit).parse_next(input)?;
    let value = a * 10 + b;
    Hour::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses a [`Minute`] (two digits).
fn minute<'i, E>(input: &mut &'i str) -> Result<Minute, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b) = (digit, digit).parse_next(input)?;
    let value = a * 10 + b;
    Minute::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses a [`Second`] (two digits).
fn second<'i, E>(input: &mut &'i str) -> Result<Second, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (a, b) = (digit, digit).parse_next(input)?;
    let value = a * 10 + b;
    Second::new(value).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

/// Parses an optional [`FractionalSecond`], including its initial `.` separator.
fn fractional_second<'i, E>(input: &mut &'i str) -> Result<Option<FractionalSecond>, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    // If there's no '.', no fractional second is present.
    if !input.starts_with('.') {
        return Ok(None);
    }

    let checkpoint = input.checkpoint();

    // Consume the '.'
    '.'.parse_next(input)?;

    // Consume all digit characters after the dot.
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    match digits.len() {
        10.. => {
            input.reset(&checkpoint);
            Err(E::from_external_error(
                input,
                JsCalendarParseError::FractionalSecondTooManyDigits,
            ))
        }
        1..=9 => {
            const PLACE_VALUES: [u32; 9] = [
                100_000_000, // 100ms
                10_000_000,  // 10ms
                1_000_000,   // 1ms
                100_000,     // 100us
                10_000,      // 10us
                1000,        // 1us
                100,         // 100ns
                10,          // 10ns
                1,           // 1ns
            ];

            if digits.as_bytes().last() == Some(&b'0') {
                input.reset(&checkpoint);
                return Err(E::from_external_error(
                    input,
                    JsCalendarParseError::FractionalSecondTrailingZeros,
                ));
            }

            let value = digits
                .as_bytes()
                .iter()
                .zip(PLACE_VALUES)
                .map(|(&d, p)| ((d - b'0') as u32) * p)
                .sum::<u32>();

            match FractionalSecond::new(value) {
                Ok(frac) => Ok(Some(frac)),
                Err(e) => {
                    input.reset(&checkpoint);
                    Err(E::from_external_error(input, e.into()))
                }
            }
        }
        // digits.len() == 0 is impossible since we used take_while(1.., ...)
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------------------
// Composite parsers (date, time, datetime)
// ---------------------------------------------------------------------------

/// Parses a [`Date`] (YYYY-MM-DD).
fn date<'i, E>(input: &mut &'i str) -> Result<Date, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (y, _, m, _, d) = (year, '-', month, '-', day).parse_next(input)?;
    Date::new(y, m, d).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, JsCalendarParseError::InvalidDate(e.into()))
    })
}

/// Parses a [`Time`] (HH:MM:SS[.frac]).
fn time<'i, E>(input: &mut &'i str) -> Result<Time, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let checkpoint = input.checkpoint();
    let (h, _, mi, _, s) = (hour, ':', minute, ':', second).parse_next(input)?;
    let frac = fractional_second(input)?;
    Time::new(h, mi, s, frac).map_err(|e| {
        input.reset(&checkpoint);
        E::from_external_error(input, e.into())
    })
}

// ---------------------------------------------------------------------------
// Top-level datetime parsers
// ---------------------------------------------------------------------------

/// Incrementally parses a datetime (no trailing marker) from `input`.
pub fn date_time<'i, E>(input: &mut &'i str) -> Result<DateTime<()>, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let (d, _, t) = (date, 'T', time).parse_next(input)?;
    Ok(DateTime {
        date: d,
        time: t,
        marker: (),
    })
}

/// Incrementally parses a UTC datetime (ending with `Z`) from `input`.
pub fn utc_date_time<'i, E>(input: &mut &'i str) -> Result<DateTime<Utc>, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let dt = terminated(date_time, 'Z').parse_next(input)?;
    Ok(DateTime {
        date: dt.date,
        time: dt.time,
        marker: Utc,
    })
}

/// Incrementally parses a local datetime (no trailing marker) from `input`.
pub fn local_date_time<'i, E>(input: &mut &'i str) -> Result<DateTime<Local>, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let dt = date_time.parse_next(input)?;
    Ok(DateTime {
        date: dt.date,
        time: dt.time,
        marker: Local,
    })
}

// ---------------------------------------------------------------------------
// Duration parsers
// ---------------------------------------------------------------------------

/// Incrementally parses a duration from `input`.
///
/// Grammar (from RFC 8984 section 1.4.6):
/// ```text
/// dur-secfrac = "." 1*DIGIT
/// dur-second  = 1*DIGIT [dur-secfrac] "S"
/// dur-minute  = 1*DIGIT "M" [dur-second]
/// dur-hour    = 1*DIGIT "H" [dur-minute]
/// dur-time    = "T" (dur-hour / dur-minute / dur-second)
/// dur-day     = 1*DIGIT "D"
/// dur-week    = 1*DIGIT "W"
/// dur-cal     = (dur-week [dur-day] / dur-day)
///
/// duration    = "P" (dur-cal [dur-time] / dur-time)
/// ```
pub fn duration<'i, E>(input: &mut &'i str) -> Result<Duration, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    /// Parses optional seconds with optional fractional part, terminated by 'S'.
    fn dur_second<'i, E>(input: &mut &'i str) -> Result<(u32, Option<FractionalSecond>), E>
    where
        E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
    {
        let seconds = parse_u32(input)?;
        let frac = fractional_second(input)?;
        'S'.parse_next(input)?;
        Ok((seconds, frac))
    }

    /// Parses the time component after the 'T' prefix.
    fn dur_time<'i, E>(input: &mut &'i str) -> Result<ExactDuration, E>
    where
        E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
    {
        let checkpoint = input.checkpoint();

        let hours = opt(terminated(parse_u32, 'H')).parse_next(input)?;
        let minutes = opt(terminated(parse_u32, 'M')).parse_next(input)?;
        let seconds = opt(dur_second).parse_next(input)?;

        match (hours, minutes, seconds) {
            (Some(_), None, Some(_)) => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    JsCalendarParseError::HourAndSecondWithoutMinute,
                ))
            }
            (None, None, None) => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    JsCalendarParseError::NoDataAfterTimePrefix,
                ))
            }
            (_, _, _) => {
                let hours = hours.unwrap_or_default();
                let minutes = minutes.unwrap_or_default();
                let (seconds, frac) = seconds.unwrap_or((0, None));

                Ok(ExactDuration {
                    hours,
                    minutes,
                    seconds,
                    frac,
                })
            }
        }
    }

    /// Parses the calendar component (weeks and/or days).
    fn dur_cal<'i, E>(input: &mut &'i str) -> Result<(u32, u32), E>
    where
        E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
    {
        let value = parse_u32(input)?;
        let terminator: char = one_of(['W', 'D']).parse_next(input)?;

        match terminator {
            'W' => {
                // After weeks, optionally parse days.
                let days = opt(terminated(parse_u32, 'D')).parse_next(input)?;
                Ok((value, days.unwrap_or(0)))
            }
            'D' => Ok((0, value)),
            _ => unreachable!(),
        }
    }

    // duration = "P" (dur-cal [dur-time] / dur-time)
    preceded(
        'P',
        alt((
            // dur-time (starts with 'T')
            preceded('T', dur_time).map(Duration::Exact),
            // dur-cal [dur-time]
            (dur_cal, opt(preceded('T', dur_time))).map(|((weeks, days), exact)| {
                Duration::Nominal(NominalDuration { weeks, days, exact })
            }),
        )),
    )
    .parse_next(input)
}

/// Incrementally parses a signed duration from `input`.
pub fn signed_duration<'i, E>(input: &mut &'i str) -> Result<SignedDuration, E>
where
    E: ParserError<&'i str> + FromExternalError<&'i str, JsCalendarParseError>,
{
    let sign = opt(one_of(['+', '-']))
        .map(|c| match c {
            Some('+') => Sign::Pos,
            Some('-') => Sign::Neg,
            _ => Sign::default(),
        })
        .parse_next(input)?;

    let dur = duration(input)?;
    Ok(SignedDuration {
        sign,
        duration: dur,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: run a parser through parse_full for convenience in tests.
    fn full<'i, T>(
        parser: impl Parser<&'i str, T, ContextError> + 'i,
        input: &'i str,
    ) -> Result<T, OwnedParseError> {
        parse_full(parser)(input)
    }

    #[test]
    fn signed_duration_parser() {
        assert!(full(duration, "").is_err());

        assert_eq!(
            full(signed_duration, "+P7W"),
            Ok(Duration::Nominal(NominalDuration {
                weeks: 7,
                ..Default::default()
            })
            .into())
        );

        assert_eq!(
            full(signed_duration, "-P15DT5H0M20S"),
            Ok(SignedDuration {
                sign: Sign::Neg,
                duration: Duration::Nominal(NominalDuration {
                    weeks: 0,
                    days: 15,
                    exact: Some(ExactDuration {
                        hours: 5,
                        minutes: 0,
                        seconds: 20,
                        frac: None
                    }),
                })
            })
        );
    }

    #[test]
    fn duration_parser() {
        assert!(full(duration, "").is_err());

        assert_eq!(
            full(duration, "P7W"),
            Ok(Duration::Nominal(NominalDuration {
                weeks: 7,
                ..Default::default()
            }))
        );

        assert_eq!(
            full(duration, "P15DT5H0M20S"),
            Ok(Duration::Nominal(NominalDuration {
                weeks: 0,
                days: 15,
                exact: Some(ExactDuration {
                    hours: 5,
                    minutes: 0,
                    seconds: 20,
                    frac: None
                }),
            }))
        );
    }

    #[test]
    fn utc_date_time_parser() {
        assert!(full(utc_date_time, "").is_err());
        assert!(full(utc_date_time, "2025-03-15T12:00:00").is_err());
        assert!(full(utc_date_time, "2025-03-15T12:00:00z").is_err());
        assert!(full(utc_date_time, "2025-03-15T12:00:00Z").is_ok());
    }

    #[test]
    fn date_time_parser() {
        assert!(full(date_time, "").is_err());

        assert_eq!(
            full(date_time, "2025-03-15T12:00:00"),
            Ok(DateTime {
                date: Date::new(Year::new(2025).unwrap(), Month::Mar, Day::D15).unwrap(),
                time: Time::new(Hour::H12, Minute::M00, Second::S00, None).unwrap(),
                marker: ()
            })
        );
    }

    #[test]
    fn date_parser() {
        for y in 0..=9999 {
            for m in 1..=12 {
                for d in 1..=12 {
                    let input = format!("{y:04}-{m:02}-{d:02}");

                    let year = Year::new(y).unwrap();
                    let month = Month::new(m).unwrap();
                    let day = Day::new(d).unwrap();

                    let expected = Date::new(year, month, day);

                    let result = full(date, &input);

                    match (result, expected) {
                        (Ok(got), Ok(exp)) => assert_eq!(got, exp),
                        (Err(_), Err(_)) => {} // both errored, fine
                        (Ok(got), Err(exp_err)) => {
                            panic!("expected error {exp_err:?} but got {got:?} for {input}")
                        }
                        (Err(err), Ok(exp)) => {
                            panic!("expected {exp:?} but got error {err:?} for {input}")
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn year_parser() {
        for i in 0..=9999 {
            let buf = format!("{i:04}");
            assert_eq!(full(year, &buf), Ok(Year::new(i).unwrap()));
        }
    }

    #[test]
    fn month_parser() {
        assert!(full(month, "").is_err());
        assert!(full(month, "0").is_err());
        assert!(full(month, "1").is_err());
        assert!(full(month, "00").is_err());

        for i in 1..=12 {
            let buf = format!("{i:02}");
            assert_eq!(full(month, &buf), Ok(Month::new(i).unwrap()));
        }
    }

    #[test]
    fn day_parser() {
        assert!(full(day, "").is_err());
        assert!(full(day, "0").is_err());
        assert!(full(day, "1").is_err());
        assert!(full(day, "00").is_err());

        for i in 1..=31 {
            let buf = format!("{i:02}");
            assert_eq!(full(day, &buf), Ok(Day::new(i).unwrap()));
        }
    }

    #[test]
    fn time_parser() {
        assert!(full(time, "").is_err());

        for hour in 0..=23 {
            for minute in 0..=59 {
                for second in 0..=60 {
                    // create an arbitrary fractional second from the other parameters
                    let frac = FractionalSecond::new({
                        let mut x: u32 = 0xafbc1234;
                        x = x.wrapping_shr(hour as u32);
                        x = x.wrapping_shr(minute as u32);
                        x = x.wrapping_shl(second as u32);
                        x / 10
                    })
                    .ok()
                    .filter(|_| second % 2 == 0);

                    // build the basic input
                    let mut buf = format!("{hour:02}:{minute:02}:{second:02}");

                    // append any fractional second
                    if let Some(frac) = frac {
                        // push all nine digits into the string
                        let tail = format!(".{:09}", frac.get());
                        buf.push_str(&tail);

                        // strip off trailing zeros
                        while buf.ends_with('0') {
                            buf.pop();
                        }
                    }

                    let expected = Time::new(
                        Hour::new(hour).unwrap(),
                        Minute::new(minute).unwrap(),
                        Second::new(second).unwrap(),
                        frac,
                    );

                    let result = full(time, &buf);

                    match (result, expected) {
                        (Ok(got), Ok(exp)) => assert_eq!(got, exp, "input: {buf}"),
                        (Err(_), Err(_)) => {}
                        (Ok(got), Err(exp_err)) => {
                            panic!("expected error {exp_err:?} but got {got:?} for {buf}")
                        }
                        (Err(err), Ok(exp)) => {
                            panic!("expected {exp:?} but got error {err:?} for {buf}")
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fractional_second_parser() {
        assert_eq!(full(fractional_second, ""), Ok(None));

        assert_eq!(
            full(fractional_second, ".000000001"),
            Ok(Some(FractionalSecond::MIN))
        );
        assert_eq!(
            full(fractional_second, ".999999999"),
            Ok(Some(FractionalSecond::MAX))
        );

        assert_eq!(
            full(fractional_second, ".001"),
            Ok(FractionalSecond::new(1_000_000).ok())
        );

        assert!(full(fractional_second, ".00000").is_err());
        assert!(full(fractional_second, ".00100").is_err());
        assert!(full(fractional_second, ".1111111111").is_err());
    }

    #[test]
    fn digit_parser() {
        // Test all digits
        for d in 0..=9u8 {
            let buf = d.to_string();
            assert_eq!(full(digit, &buf), Ok(d));
        }

        // Non-digit should fail
        assert!(full(digit, "A").is_err());
        assert!(full(digit, "").is_err());
    }
}
