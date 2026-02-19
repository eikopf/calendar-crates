//! Parsers for types which are encoded as strings by JSCalendar.

use std::{cmp::Ordering, convert::Infallible};

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

// # Implementation
//
// An incremental parser is a function which takes &mut &str (and potentially other parameters as
// well) and returns a Result. The parser *succeeds* if it returns Ok, and *fails* otherwise. We
// require that a parser function must not (visibly) write through the &mut &str parameter unless
// it succeeds.
//
// This pattern is quite similar to winnow, although winnow also supports non-function parsers. The
// reason we're not using winnow is because we want total control over the error types produced by
// parsers, and in particular we want to avoid producing opaque string-based errors.
//
// In this module, errors fall into three classes:
//
// 1. Non-specific parsing errors.
// 2. Specific syntactic errors.
// 3. Specific semantic errors.
//
// The first class contains all general parsing errors, represented by GeneralParseError. This is
// mostly used by lower-level combinators and primitives to return errors that can't be given more
// specific context based on the type being parsed (e.g. the unexpected end of input or a malformed
// string slice).
//
// The second and third classes are specific to the type being parsed, and are distinguished from
// one another by their relative scopes. The second kind of error can *only* appear when a parser
// in this module produces it, and is fundamentally still about parsing; by contrast the third kind
// may have any scope and appear anywhere in the codebase. For example, Year::new might be called
// anywhere to return an InvalidYearError (including the downstream code of users), whereas
// YearParseError can only occur because someone called the year parser in this module.

/// Converts an incremental parser into a complete parser, which will return an error if the input
/// string is not completely consumed.
pub fn parse_full<'i, T, Sy, Se>(
    parser: impl FnOnce(&mut &'i str) -> ParseResult<'i, T, Sy, Se>,
) -> impl FnOnce(&'i str) -> Result<T, OwnedParseError<Sy, Se>> {
    |s| {
        let mut input = s;
        let result = parser(&mut input)
            .map_err(|error| OwnedParseError::from_parse_error(error, s.into()))?;

        match input.is_empty() {
            true => Ok(result),
            false => {
                let parse_error = ParseError::general(input, GeneralParseError::UnconsumedInput);
                let error = OwnedParseError::from_parse_error(parse_error, s.into());
                Err(error)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedParseError<Sy, Se> {
    complete_input: Box<str>,
    index: usize,
    error: ParseErrorKind<Sy, Se>,
}

impl<Sy, Se> OwnedParseError<Sy, Se> {
    fn from_parse_error(error: ParseError<&str, Sy, Se>, complete_input: Box<str>) -> Self {
        debug_assert!(complete_input.contains(error.input));
        let index = complete_input.len() - error.input.len();

        Self {
            complete_input,
            index,
            error: error.error,
        }
    }

    fn into_semantic(self) -> Option<Se> {
        match self.error {
            ParseErrorKind::Semantic(error) => Some(error),
            _ => None,
        }
    }
}

impl<Sy: std::fmt::Display, Se: std::fmt::Display> std::fmt::Display for OwnedParseError<Sy, Se> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at index {} of {:?})", self.error, self.index, self.complete_input)
    }
}

/// The result of applying a parser, which is either a `T` or a [`ParseError<&'i str, Sy, Se>`].
pub type ParseResult<'i, T, Sy, Se> = Result<T, ParseError<&'i str, Sy, Se>>;

/// The error produced by a parser, holding some input `I` and an error which may be general (of
/// type [`GeneralParseError`]), syntactic (of type `Sy`), or semantic (of type `Se`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError<I, Sy, Se> {
    input: I,
    error: ParseErrorKind<Sy, Se>,
}

impl<I, Sy, Se> From<Infallible> for ParseError<I, Sy, Se> {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl<I, Sy, Se> ParseError<I, Sy, Se> {
    #[inline(always)]
    pub fn unexpected_eof() -> Self
    where
        &'static str: Into<I>,
    {
        Self {
            input: "".into(),
            error: ParseErrorKind::General(GeneralParseError::UnexpectedEof),
        }
    }

    #[inline(always)]
    pub const fn invalid_split_index(input: I, index: usize) -> Self {
        Self {
            input,
            error: ParseErrorKind::General(GeneralParseError::InvalidSplitIndex(index)),
        }
    }

    #[inline(always)]
    pub const fn insufficient_input(input: I, count: usize) -> Self {
        Self {
            input,
            error: ParseErrorKind::General(GeneralParseError::InvalidSplitIndex(count)),
        }
    }

    #[inline(always)]
    pub const fn general(input: I, error: GeneralParseError) -> Self {
        Self {
            input,
            error: ParseErrorKind::General(error),
        }
    }

    #[inline(always)]
    pub const fn syntax(input: I, error: Sy) -> Self {
        Self {
            input,
            error: ParseErrorKind::Syntax(error),
        }
    }

    #[inline(always)]
    pub const fn semantic(input: I, error: Se) -> Self {
        Self {
            input,
            error: ParseErrorKind::Semantic(error),
        }
    }

    #[inline(always)]
    pub fn into_semantic(self) -> Option<Se> {
        match self.error {
            ParseErrorKind::Semantic(error) => Some(error),
            _ => None,
        }
    }

    pub fn coerce<Sy2, Se2>(self) -> ParseError<I, Sy2, Se2>
    where
        Sy: Into<Sy2>,
        Se: Into<Se2>,
    {
        let Self { input, error } = self;

        match error {
            ParseErrorKind::General(error) => ParseError::general(input, error),
            ParseErrorKind::Syntax(error) => ParseError::syntax(input, error.into()),
            ParseErrorKind::Semantic(error) => ParseError::semantic(input, error.into()),
        }
    }

    #[inline(always)]
    pub fn coerce_semantic<Se2>(self) -> ParseError<I, Sy, Se2>
    where
        Se: Into<Se2>,
    {
        self.coerce()
    }

    #[inline(always)]
    pub fn coerce_syntax<Sy2>(self) -> ParseError<I, Sy2, Se>
    where
        Sy: Into<Sy2>,
    {
        self.coerce()
    }
}

/// The kind of a [`ParseError`].
#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum ParseErrorKind<Sy, Se> {
    /// A non-specific parsing error.
    #[error("parse error: {0}")]
    General(GeneralParseError),
    /// A syntactic error specific to the type being parsed.
    #[error("syntax error: {0}")]
    Syntax(Sy),
    /// A semantic error specific to the type being parsed.
    #[error("semantic error: {0}")]
    Semantic(Se),
}

/// A non-specific parsing error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GeneralParseError {
    /// The input was completely empty.
    #[error("unexpected end of input")]
    UnexpectedEof,
    /// The input did not have enough data to proceed.
    #[error("insufficient input (expected {0} bytes)")]
    InsufficientInput(usize),
    /// The input was split in the middle of a UTF-8 character.
    #[error("attempted to split input at an invalid index ({0})")]
    InvalidSplitIndex(usize),
    /// After parsing, the input was not completely consumed.
    #[error("unconsumed input")]
    UnconsumedInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SignedDurationParseError {
    #[error("expected +, -, or P, but got {0} instead")]
    InvalidPrefix(char),
    #[error(transparent)]
    Duration(#[from] DurationParseError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DurationParseError {
    #[error("expected P but got {0} instead")]
    InvalidPrefix(char),
    #[error("expected W or D but got {0} instead")]
    InvalidNominalTerminator(char),
    #[error("expected T or an ASCII digit but got {0} instead")]
    InvalidTimePrefix(char),
    #[error("expected D but got {0} instead")]
    InvalidDayTerminator(char),
    #[error("expected H but got {0} instead")]
    InvalidHourTerminator(char),
    #[error("expected M but got {0} instead")]
    InvalidMinuteTerminator(char),
    #[error("expected S but got {0} instead")]
    InvalidSecondTerminator(char),
    #[error("expected some data after T")]
    NoDataAfterTimePrefix,
    #[error("exact time contains hours and seconds but not minutes")]
    HourAndSecondWithoutMinute,
    #[error(transparent)]
    U32(#[from] U32ParseError),
    #[error(transparent)]
    FractionalSecond(#[from] FractionalSecondParseError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum UtcDateTimeParseError {
    #[error(transparent)]
    DateTime(#[from] DateTimeParseError),
    #[error("expected Z but got {0} instead")]
    InvalidMarker(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DateTimeParseError {
    #[error("invalid date: {0}")]
    Date(#[from] DateParseError),
    #[error("invalid time: {0}")]
    Time(#[from] TimeParseError),
    #[error("expected T but got {0} instead")]
    InvalidSeparator(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DateParseError {
    #[error("invalid year: {0}")]
    Year(#[from] YearParseError),
    #[error("invalid month: {0}")]
    Month(#[from] MonthParseError),
    #[error("invalid day: {0}")]
    Day(#[from] DayParseError),
    #[error("expected a hyphen but got {0} instead")]
    InvalidSeparator(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimeParseError {
    #[error("invalid hour: {0}")]
    Hour(#[from] HourParseError),
    #[error("invalid minute: {0}")]
    Minute(#[from] MinuteParseError),
    #[error("invalid second: {0}")]
    Second(#[from] SecondParseError),
    #[error("invalid fractional second: {0}")]
    FractionalSecond(#[from] FractionalSecondParseError),
    #[error("expected a colon but got {0} instead")]
    InvalidSeparator(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct YearParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct MonthParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct DayParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct HourParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct MinuteParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct SecondParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FractionalSecondParseError {
    #[error(transparent)]
    Digit(#[from] DigitParseError),
    #[error("a trailing zero was encountered")]
    TrailingZeros,
    #[error("no digits after the decimal point")]
    NoDigits,
    #[error("more than 9 digits")]
    TooManyDigits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum U32ParseError {
    #[error("expected an ASCII decimal digit")]
    NoDigits,
    #[error(transparent)]
    Digit(#[from] DigitParseError),
    #[error("encountered an integer greater than u32::MAX")]
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("expected an ASCII digit but got the byte {0:02X} instead")]
pub struct DigitParseError(u8);

pub fn signed_duration<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, SignedDuration, SignedDurationParseError, InvalidDurationError> {
    let sign = match peek(input)? {
        '+' => {
            let () = skip(input, 1)?;
            Ok(Sign::Pos)
        }
        '-' => {
            let () = skip(input, 1)?;
            Ok(Sign::Neg)
        }
        'P' => Ok(Sign::default()),
        c => Err(ParseError::syntax(
            *input,
            SignedDurationParseError::InvalidPrefix(c),
        )),
    }?;

    let duration = duration(input).map_err(ParseError::coerce)?;
    Ok(SignedDuration { sign, duration })
}

pub fn duration<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, Duration, DurationParseError, InvalidDurationError> {
    // # Grammar (from RFC 8984 §1.4.6).
    //
    // dur-secfrac = "." 1*DIGIT
    // dur-second  = 1*DIGIT [dur-secfrac] "S"
    // dur-minute  = 1*DIGIT "M" [dur-second]
    // dur-hour    = 1*DIGIT "H" [dur-minute]
    // dur-time    = "T" (dur-hour / dur-minute / dur-second)
    // dur-day     = 1*DIGIT "D"
    // dur-week    = 1*DIGIT "W"
    // dur-cal     = (dur-week [dur-day] / dur-day)
    //
    // duration    = "P" (dur-cal [dur-time] / dur-time)
    //
    // # Implementation
    //
    // After looking at the first character and checking it's "P", we need to be a little clever
    // with how we check what grammar rule we should follow; we can also avoid throwing away work
    // in a few places.
    //
    // - duration: check for the literal "T" to decide between the two branches
    // - dur-cal: parse a number unconditionally and check the next byte ("W" or "D")
    // - dur-time: parse a number unconditionally and check the next byte ("H", "M", "S", or ".")

    #[inline(always)]
    fn dur_second<'i>(
        input: &mut &'i str,
    ) -> ParseResult<'i, (u32, Option<FractionalSecond>), DurationParseError, InvalidDurationError>
    {
        let seconds = u32(input).map_err(ParseError::coerce_syntax)?;
        let frac = fractional_second(input).map_err(ParseError::coerce)?;
        let () = separator('S', DurationParseError::InvalidSecondTerminator)(input)?;
        Ok((seconds, frac))
    }

    fn u32_char<'i>(
        terminator: char,
        f: impl Fn(char) -> DurationParseError,
    ) -> impl FnOnce(&mut &'i str) -> ParseResult<'i, u32, DurationParseError, InvalidDurationError>
    {
        move |input| {
            let value = u32(input).map_err(ParseError::coerce_syntax)?;
            let () = separator(terminator, f)(input)?;
            Ok(value)
        }
    }

    /// A modified version of the `dur-time` rule without the leading `T`.
    fn dur_time<'i>(
        input: &mut &'i str,
    ) -> ParseResult<'i, ExactDuration, DurationParseError, InvalidDurationError> {
        let checkpoint = *input;

        let hours = optional(u32_char('H', DurationParseError::InvalidHourTerminator))(input)?;
        let minutes = optional(u32_char('M', DurationParseError::InvalidMinuteTerminator))(input)?;
        let seconds = optional(dur_second)(input)?;

        match (hours, minutes, seconds) {
            (Some(_), None, Some(_)) => {
                *input = checkpoint;
                Err(ParseError::syntax(
                    input,
                    DurationParseError::HourAndSecondWithoutMinute,
                ))
            }
            (None, None, None) => {
                *input = checkpoint;
                Err(ParseError::syntax(
                    input,
                    DurationParseError::NoDataAfterTimePrefix,
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

    fn dur_cal<'i>(
        input: &mut &'i str,
    ) -> ParseResult<'i, (u32, u32), DurationParseError, InvalidDurationError> {
        let value = u32(input).map_err(ParseError::coerce_syntax)?;

        match char(input)? {
            'W' => match peek::<(), ()>(input).ok() {
                Some('0'..='9') => {
                    let weeks = value;
                    let days = u32(input).map_err(ParseError::coerce_syntax)?;
                    let () = separator('D', DurationParseError::InvalidDayTerminator)(input)?;

                    Ok((weeks, days))
                }
                _ => Ok((value, 0)),
            },
            'D' => Ok((0, value)),
            c => Err(ParseError::syntax(
                input,
                DurationParseError::InvalidNominalTerminator(c),
            )),
        }
    }

    let () = separator('P', DurationParseError::InvalidPrefix)(input)?;
    match peek(input)? {
        'T' => {
            let () = skip(input, 1)?;
            Ok(Duration::Exact(dur_time(input)?))
        }
        '0'..='9' => {
            let (weeks, days) = dur_cal(input)?;

            let exact = match peek::<(), ()>(input).ok() {
                Some('T') => {
                    let () = skip(input, 1)?;
                    Some(dur_time(input)?)
                }
                _ => None,
            };

            Ok(Duration::Nominal(NominalDuration { weeks, days, exact }))
        }
        c => Err(ParseError::syntax(
            input,
            DurationParseError::InvalidTimePrefix(c),
        )),
    }
}

pub fn utc_date_time<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, DateTime<Utc>, UtcDateTimeParseError, InvalidDateTimeError> {
    let DateTime { date, time, .. } = date_time(input).map_err(ParseError::coerce)?;
    let () = separator('Z', UtcDateTimeParseError::InvalidMarker)(input)?;

    Ok(DateTime {
        date,
        time,
        marker: Utc,
    })
}

pub fn local_date_time<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, DateTime<Local>, DateTimeParseError, InvalidDateTimeError> {
    let DateTime { date, time, .. } = date_time(input).map_err(ParseError::coerce)?;

    Ok(DateTime {
        date,
        time,
        marker: Local,
    })
}

pub fn date_time<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, DateTime<()>, DateTimeParseError, InvalidDateTimeError> {
    let date = date(input).map_err(ParseError::coerce)?;
    let () = separator('T', DateTimeParseError::InvalidSeparator)(input)?;
    let time = time(input).map_err(ParseError::coerce)?;

    Ok(DateTime {
        date,
        time,
        marker: (),
    })
}

fn date<'i>(input: &mut &'i str) -> ParseResult<'i, Date, DateParseError, InvalidDateError> {
    let checkpoint = *input;
    let hyphen = separator('-', DateParseError::InvalidSeparator);

    let year = year(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let month = month(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let day = day(input).map_err(ParseError::coerce)?;

    match Date::new(year, month, day) {
        Ok(date) => Ok(date),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error.into()))
        }
    }
}

/// Parses a [`Year`].
fn year<'i>(input: &mut &'i str) -> ParseResult<'i, Year, YearParseError, InvalidYearError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)? as u16;
    let b = digit(input).map_err(ParseError::coerce_syntax)? as u16;
    let c = digit(input).map_err(ParseError::coerce_syntax)? as u16;
    let d = digit(input).map_err(ParseError::coerce_syntax)? as u16;
    let value = (a * 1000) + (b * 100) + (c * 10) + d;

    match Year::new(value) {
        Ok(year) => Ok(year),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Month`].
fn month<'i>(input: &mut &'i str) -> ParseResult<'i, Month, MonthParseError, InvalidMonthError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)?;
    let b = digit(input).map_err(ParseError::coerce_syntax)?;
    let value = (a * 10) + b;

    match Month::new(value) {
        Ok(month) => Ok(month),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Day`].
fn day<'i>(input: &mut &'i str) -> ParseResult<'i, Day, DayParseError, InvalidDayError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)?;
    let b = digit(input).map_err(ParseError::coerce_syntax)?;
    let value = (a * 10) + b;

    match Day::new(value) {
        Ok(day) => Ok(day),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Time`].
fn time<'i>(input: &mut &'i str) -> ParseResult<'i, Time, TimeParseError, InvalidTimeError> {
    let checkpoint = *input;
    let colon = separator(':', TimeParseError::InvalidSeparator);

    let hour = hour(input).map_err(ParseError::coerce)?;
    let () = colon(input)?;
    let minute = minute(input).map_err(ParseError::coerce)?;
    let () = colon(input)?;
    let second = second(input).map_err(ParseError::coerce)?;
    let frac = fractional_second(input).map_err(ParseError::coerce)?;

    match Time::new(hour, minute, second, frac) {
        Ok(time) => Ok(time),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses an [`Hour`].
fn hour<'i>(input: &mut &'i str) -> ParseResult<'i, Hour, HourParseError, InvalidHourError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)?;
    let b = digit(input).map_err(ParseError::coerce_syntax)?;
    let value = (a * 10) + b;

    match Hour::new(value) {
        Ok(hour) => Ok(hour),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Minute`].
fn minute<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, Minute, MinuteParseError, InvalidMinuteError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)?;
    let b = digit(input).map_err(ParseError::coerce_syntax)?;
    let value = (a * 10) + b;

    match Minute::new(value) {
        Ok(minute) => Ok(minute),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Second`].
fn second<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, Second, SecondParseError, InvalidSecondError> {
    let checkpoint = *input;
    let a = digit(input).map_err(ParseError::coerce_syntax)?;
    let b = digit(input).map_err(ParseError::coerce_syntax)?;
    let value = (a * 10) + b;

    match Second::new(value) {
        Ok(second) => Ok(second),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses an optional [`FractionalSecond`], including its initial `.` separator.
fn fractional_second<'i>(
    input: &mut &'i str,
) -> ParseResult<
    'i,
    Option<FractionalSecond>,
    FractionalSecondParseError,
    InvalidFractionalSecondError,
> {
    if input.is_empty() || !input.starts_with('.') {
        return Ok(None);
    }

    let checkpoint = *input;
    let () = skip(input, 1)?;
    let digits = digits0(input)?;

    match digits.len() {
        0 => Err(ParseError::syntax(
            input,
            FractionalSecondParseError::NoDigits,
        )),
        10.. => Err(ParseError::syntax(
            input,
            FractionalSecondParseError::TooManyDigits,
        )),
        1..=9 => {
            const PLACE_VALUES: [u32; 9] = [
                100_000_000, // 100ms
                10_000_000,  // 10ms
                1_000_000,   // 1ms
                100_000,     // 100μs
                10_000,      // 10μs
                1000,        // 1μs
                100,         // 100ns
                10,          // 10ns
                1,           // 1ns
            ];

            if digits.as_bytes().last() == Some(&b'0') {
                return Err(ParseError::syntax(
                    input,
                    FractionalSecondParseError::TrailingZeros,
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
                Err(error) => {
                    *input = checkpoint;
                    Err(ParseError::semantic(input, error))
                }
            }
        }
    }
}

// COMBINATORS

/// Converts a fallible parser of `T` into an infallible parser of [`Option<T>`].
#[inline(always)]
fn optional<'i, T, Sy, Se>(
    parser: impl FnOnce(&mut &'i str) -> ParseResult<'i, T, Sy, Se>,
) -> impl FnOnce(&mut &'i str) -> Result<Option<T>, Infallible> {
    |input| Ok(parser(input).ok())
}

/// Returns the next character of the input without advancing.
#[inline(always)]
fn peek<'i, Sy, Se>(input: &&'i str) -> ParseResult<'i, char, Sy, Se> {
    input
        .chars()
        .next()
        .ok_or_else(|| ParseError::unexpected_eof())
}

/// Constructs a parser that tries to parse `sep`, and constructs an error message from the parsed
/// character using `f` if it fails.
fn separator<'i, Sy, Se>(
    sep: char,
    f: impl Fn(char) -> Sy,
) -> impl Fn(&mut &'i str) -> ParseResult<'i, (), Sy, Se> {
    move |input| {
        let checkpoint = *input;
        let c = char(input)?;

        if c == sep {
            Ok(())
        } else {
            *input = checkpoint;
            Err(ParseError::syntax(input, f(c)))
        }
    }
}

/// Parses a [`u32`].
fn u32<'i, Se>(input: &mut &'i str) -> ParseResult<'i, u32, U32ParseError, Se> {
    let checkpoint = *input;
    let digits = digits0(input)?;

    if digits.is_empty() {
        Err(ParseError::syntax(input, U32ParseError::NoDigits))
    } else {
        str::parse::<u32>(digits).map_err(|error| {
            debug_assert_eq!(error.kind(), &std::num::IntErrorKind::PosOverflow);
            *input = checkpoint;
            ParseError::syntax(*input, U32ParseError::Overflow)
        })
    }
}

/// Parses zero or more digits. Unlike the [`digit`] parser, the resulting slice contains ASCII
/// digits rather than the literal values of each digit.
fn digits0<'i, Sy, Se>(input: &mut &'i str) -> ParseResult<'i, &'i str, Sy, Se> {
    match input.split_once(|c: char| !c.is_ascii_digit()) {
        None => {
            let (head, tail) = (*input, "");
            *input = tail;
            Ok(head)
        }
        Some((head, _)) => {
            let () = skip(input, head.len())?;
            Ok(head)
        }
    }
}

/// Parses a single digit.
fn digit<'i, Se>(input: &mut &'i str) -> ParseResult<'i, u8, DigitParseError, Se> {
    let checkpoint = *input;
    let byte = byte(input)?;

    match byte.is_ascii_digit() {
        true => Ok(byte - b'0'),
        false => {
            *input = checkpoint;
            Err(ParseError::syntax(input, DigitParseError(byte)))
        }
    }
}

/// Parses a single character.
fn char<'i, Sy, Se>(input: &mut &'i str) -> ParseResult<'i, char, Sy, Se> {
    let mut chars = input.chars();

    match chars.next() {
        None => Err(ParseError::unexpected_eof()),
        Some(c) => {
            *input = chars.as_str();
            Ok(c)
        }
    }
}

/// Parses a single byte.
fn byte<'i, Sy, Se>(input: &mut &'i str) -> ParseResult<'i, u8, Sy, Se> {
    match input.as_bytes().first() {
        None => Err(ParseError::unexpected_eof()),
        Some(&b) => match input.split_at_checked(1) {
            None => Err(ParseError::invalid_split_index(input, 1)),
            Some((_, tail)) => {
                *input = tail;
                Ok(b)
            }
        },
    }
}

/// Skips the next `count` bytes in the `input`.
fn skip<'i, Sy, Se>(input: &mut &'i str, count: usize) -> ParseResult<'i, (), Sy, Se> {
    take_str(input, count).and(Ok(()))
}

/// Takes the first `count` bytes from the `input`, returning an error if removing these inputs
/// would leave the `input` in an invalid state.
fn take_str<'i, Sy, Se>(input: &mut &'i str, count: usize) -> ParseResult<'i, &'i str, Sy, Se> {
    match input.len().cmp(&count) {
        Ordering::Less => Err(ParseError::insufficient_input(input, count)),
        Ordering::Equal => {
            let (head, tail) = (*input, "");
            *input = tail;
            Ok(head)
        }
        Ordering::Greater => match input.split_at_checked(count) {
            None => Err(ParseError::invalid_split_index(input, count)),
            Some((head, tail)) => {
                *input = tail;
                Ok(head)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_duration_parser() {
        assert_eq!(duration(&mut ""), Err(ParseError::unexpected_eof()));

        assert_eq!(
            signed_duration(&mut "+P7W"),
            Ok(Duration::Nominal(NominalDuration {
                weeks: 7,
                ..Default::default()
            })
            .into())
        );

        assert_eq!(
            signed_duration(&mut "-P15DT5H0M20S"),
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
        assert_eq!(duration(&mut ""), Err(ParseError::unexpected_eof()));

        assert_eq!(
            duration(&mut "P7W"),
            Ok(Duration::Nominal(NominalDuration {
                weeks: 7,
                ..Default::default()
            }))
        );

        assert_eq!(
            duration(&mut "P15DT5H0M20S"),
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
        assert_eq!(utc_date_time(&mut ""), Err(ParseError::unexpected_eof()));

        assert_eq!(
            utc_date_time(&mut "2025-03-15T12:00:00"),
            Err(ParseError::unexpected_eof())
        );

        assert_eq!(
            utc_date_time(&mut "2025-03-15T12:00:00z"),
            Err(ParseError::syntax(
                "z",
                UtcDateTimeParseError::InvalidMarker('z')
            ))
        );

        assert!(utc_date_time(&mut "2025-03-15T12:00:00Z").is_ok());
    }

    #[test]
    fn date_time_parser() {
        assert_eq!(date_time(&mut ""), Err(ParseError::unexpected_eof()));

        let input = &mut "2025-03-15T12:00:00";

        assert_eq!(
            date_time(input),
            Ok(DateTime {
                date: Date::new(Year::new(2025).unwrap(), Month::Mar, Day::D15).unwrap(),
                time: Time::new(Hour::H12, Minute::M00, Second::S00, None).unwrap(),
                marker: ()
            })
        );

        assert!(input.is_empty());
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

                    let parser = parse_full(date);
                    assert_eq!(
                        parser(&input).map_err(|e| e.into_semantic().unwrap()),
                        Date::new(year, month, day).map_err(Into::into)
                    );
                }
            }
        }
    }

    #[test]
    fn year_parser() {
        for i in 0..=9999 {
            let buf = format!("{i:04}");
            let mut input = buf.as_str();
            assert_eq!(year(&mut input), Ok(Year::new(i).unwrap()));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn month_parser() {
        assert_eq!(month(&mut ""), Err(ParseError::unexpected_eof()));
        assert_eq!(month(&mut "0"), Err(ParseError::unexpected_eof()));
        assert_eq!(month(&mut "1"), Err(ParseError::unexpected_eof()));
        assert!(month(&mut "00").is_err());

        for i in 1..=12 {
            let buf = format!("{i:02}");
            let mut input = buf.as_str();
            assert_eq!(month(&mut input), Ok(Month::new(i).unwrap()));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn day_parser() {
        assert_eq!(day(&mut ""), Err(ParseError::unexpected_eof()));
        assert_eq!(day(&mut "0"), Err(ParseError::unexpected_eof()));
        assert_eq!(day(&mut "1"), Err(ParseError::unexpected_eof()));
        assert!(day(&mut "00").is_err());

        for i in 1..=31 {
            let buf = format!("{i:02}");
            let mut input = buf.as_str();
            assert_eq!(day(&mut input), Ok(Day::new(i).unwrap()));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn time_parser() {
        assert_eq!(time(&mut ""), Err(ParseError::unexpected_eof()));

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

                    let mut input = buf.as_str();

                    let expected = Time::new(
                        Hour::new(hour).unwrap(),
                        Minute::new(minute).unwrap(),
                        Second::new(second).unwrap(),
                        frac,
                    );

                    dbg![&input];
                    dbg![&expected];

                    assert_eq!(
                        time(&mut input).map_err(|e| e.into_semantic().unwrap()),
                        expected
                    );

                    assert!(input.is_empty());
                }
            }
        }
    }

    #[test]
    fn fractional_second_parser() {
        assert_eq!(fractional_second(&mut ""), Ok(None));
        assert_eq!(fractional_second(&mut "1.00001"), Ok(None));

        assert_eq!(
            fractional_second(&mut ".000000001"),
            Ok(Some(FractionalSecond::MIN))
        );
        assert_eq!(
            fractional_second(&mut ".999999999"),
            Ok(Some(FractionalSecond::MAX))
        );

        assert_eq!(
            fractional_second(&mut ".001"),
            Ok(FractionalSecond::new(1_000_000).ok())
        );

        assert!(fractional_second(&mut ".00000").is_err());
        assert!(fractional_second(&mut ".00100").is_err());
        assert!(fractional_second(&mut ".1111111111").is_err());
    }

    #[test]
    fn digit_parser() {
        assert_eq!(digit::<()>(&mut ""), Err(ParseError::unexpected_eof()));

        assert_eq!(digit::<()>(&mut "0"), Ok(0));
        assert_eq!(digit::<()>(&mut "1"), Ok(1));
        assert_eq!(digit::<()>(&mut "2"), Ok(2));
        assert_eq!(digit::<()>(&mut "3"), Ok(3));
        assert_eq!(digit::<()>(&mut "4"), Ok(4));
        assert_eq!(digit::<()>(&mut "5"), Ok(5));
        assert_eq!(digit::<()>(&mut "6"), Ok(6));
        assert_eq!(digit::<()>(&mut "7"), Ok(7));
        assert_eq!(digit::<()>(&mut "8"), Ok(8));
        assert_eq!(digit::<()>(&mut "9"), Ok(9));
        assert_eq!(
            digit::<()>(&mut "A"),
            Err(ParseError::syntax("A", DigitParseError(b'A')))
        );

        assert_eq!(digit::<()>(&mut "0dgsahjk"), Ok(0));
        assert_eq!(digit::<()>(&mut "15674352756743"), Ok(1));
        assert_eq!(digit::<()>(&mut "2    "), Ok(2));
        assert_eq!(digit::<()>(&mut "3\t\t\t\t"), Ok(3));
        assert_eq!(digit::<()>(&mut "4cbzxnmbc"), Ok(4));
        assert_eq!(digit::<()>(&mut "59888988"), Ok(5));
        assert_eq!(
            digit::<()>(&mut "A0"),
            Err(ParseError::syntax("A0", DigitParseError(b'A')))
        );
    }

    #[test]
    fn skip_parser() {
        let input = &mut "0123456789ABCDEF";

        assert_eq!(skip::<(), ()>(input, 0), Ok(()));
        assert_eq!(*input, "0123456789ABCDEF");
        assert_eq!(skip::<(), ()>(input, 4), Ok(()));
        assert_eq!(*input, "456789ABCDEF");
        assert_eq!(skip::<(), ()>(input, 6), Ok(()));
        assert_eq!(*input, "ABCDEF");
        assert_eq!(skip::<(), ()>(input, 5), Ok(()));
        assert_eq!(*input, "F");
        assert_eq!(skip::<(), ()>(input, 1), Ok(()));
        assert_eq!(*input, "");
        assert_eq!(skip::<(), ()>(input, 0), Ok(()));
        assert_eq!(*input, "");
        assert_eq!(
            skip::<(), ()>(input, 1),
            Err(ParseError::insufficient_input("", 1))
        );
    }

    #[test]
    fn take_str_parser() {
        let input = &mut "abcdαβγδ";

        assert_eq!(take_str::<(), ()>(input, 0), Ok(""));
        assert_eq!(*input, "abcdαβγδ");
        assert_eq!(take_str::<(), ()>(input, 2), Ok("ab"));
        assert_eq!(*input, "cdαβγδ");
        assert_eq!(take_str::<(), ()>(input, 2), Ok("cd"));
        assert_eq!(*input, "αβγδ");
        assert_eq!(take_str::<(), ()>(input, 2), Ok("α"));
        assert_eq!(*input, "βγδ");

        assert_eq!(
            take_str::<(), ()>(input, 3),
            Err(ParseError::invalid_split_index("βγδ", 3))
        );
        assert_eq!(*input, "βγδ");

        assert_eq!(take_str::<(), ()>(input, 4), Ok("βγ"));
        assert_eq!(*input, "δ");

        assert_eq!(
            take_str::<(), ()>(input, 1),
            Err(ParseError::invalid_split_index("δ", 1))
        );
        assert_eq!(*input, "δ");

        assert_eq!(
            take_str::<(), ()>(input, 3),
            Err(ParseError::insufficient_input("δ", 3))
        );
        assert_eq!(*input, "δ");
        assert_eq!(take_str::<(), ()>(input, 2), Ok("δ"));
        assert_eq!(*input, "");
        assert_eq!(take_str::<(), ()>(input, 0), Ok(""));
        assert_eq!(*input, "");
    }
}
