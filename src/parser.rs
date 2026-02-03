//! Parsers for types which are encoded as strings by JSCalendar.

use std::cmp::Ordering;

use thiserror::Error;

use crate::model::time::{
    Date, Day, InvalidDateError, InvalidDayError, InvalidMonthError, InvalidTimeError,
    InvalidYearError, Month, Time, Year,
};

pub fn parse_full<'i, T, Sy, Se>(
    parser: impl FnOnce(&mut &'i str) -> ParseResult<'i, T, Sy, Se>,
) -> impl FnOnce(&'i str) -> ParseResult<'i, T, Sy, Se> {
    |s| {
        let mut input = s;
        let result = parser(&mut input)?;

        match input.is_empty() {
            true => Ok(result),
            false => Err(ParseError::general(
                input,
                GeneralParseError::UnconsumedInput,
            )),
        }
    }
}

pub type ParseResult<'i, T, Sy, Se> = Result<T, ParseError<&'i str, Sy, Se>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError<I, Sy, Se> {
    input: I,
    error: ParseErrorKind<Sy, Se>,
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

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum ParseErrorKind<Sy, Se> {
    #[error("parse error: {0}")]
    General(GeneralParseError),
    #[error("syntax error: {0}")]
    Syntax(Sy),
    #[error("semantic error: {0}")]
    Semantic(Se),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GeneralParseError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("insufficient input (expected {0} bytes)")]
    InsufficientInput(usize),
    #[error("attempted to split input at an invalid index ({0})")]
    InvalidSplitIndex(usize),
    #[error("unconsumed input")]
    UnconsumedInput,
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
#[error(transparent)]
pub struct YearParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct MonthParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct DayParseError(#[from] DigitParseError);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("expected an ASCII digit but got the byte {0:02X} instead")]
pub struct DigitParseError(u8);

// TODO: implement separate parsers for LocalDateTime and UtcDateTime

// pub fn date_time<M: DateTimeMarker>(
//     input: &mut &str,
// ) -> Result<DateTime<M>, ParseError<InvalidDateTimeError<M>>> {
//     let date = date(input).map_err(ParseError::coerce)?;
//     let _ = byte_where(input, |b| b == b'T')?;
//     let time = time(input).map_err(ParseError::coerce)?;
//     let marker = M::parser(input)?;
//     Ok(DateTime { date, time, marker })
// }

pub fn date<'i>(input: &mut &'i str) -> ParseResult<'i, Date, DateParseError, InvalidDateError> {
    fn hyphen<'i>(input: &mut &'i str) -> ParseResult<'i, (), DateParseError, InvalidDateError> {
        let checkpoint = *input;

        match char(input)? {
            '-' => Ok(()),
            c => {
                *input = checkpoint;
                Err(ParseError::syntax(
                    input,
                    DateParseError::InvalidSeparator(c),
                ))
            }
        }
    }

    let checkpoint = *input;
    let year = year(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let month = month(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let day = day(input).map_err(ParseError::coerce)?;

    match Date::new(year, month, day) {
        Ok(date) => Ok(date),
        Err(error) => {
            *input = checkpoint;
            Err(ParseError::semantic(input, error))
        }
    }
}

/// Parses a [`Year`].
pub fn year<'i>(input: &mut &'i str) -> ParseResult<'i, Year, YearParseError, InvalidYearError> {
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
pub fn month<'i>(
    input: &mut &'i str,
) -> ParseResult<'i, Month, MonthParseError, InvalidMonthError> {
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
pub fn day<'i>(input: &mut &'i str) -> ParseResult<'i, Day, DayParseError, InvalidDayError> {
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
pub fn time<'i>(input: &mut &'i str) -> ParseResult<'i, Time, (), InvalidTimeError> {
    todo!()
}

// COMBINATORS

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
                        Date::new(year, month, day)
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
