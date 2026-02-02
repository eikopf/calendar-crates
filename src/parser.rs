//! Parsers for types which are encoded as strings by JSCalendar.

use std::{cmp::Ordering, num::NonZero};

use crate::model::time::{
    Date, DateTime, Day, InvalidDateError, InvalidDateTimeError, InvalidDayError,
    InvalidMonthError, InvalidTimeError, InvalidYearError, Local, Month, Time, Utc, Year,
};

pub fn parse_full<T, E>(
    parser: impl FnOnce(&mut &str) -> Result<T, ParseError<E>>,
) -> impl FnOnce(&str) -> Result<Result<T, E>, ParseFullError> {
    |s| {
        let mut input = s;
        match parser(&mut input) {
            Ok(value) => match BytesRemaining::of(input) {
                Some(br) => Err(ParseFullError::RemainingInput(br)),
                None => Ok(Ok(value)),
            },
            Err(error) => error.into_parse_full_error(input).map(Err),
        }
    }
}

/// A non-zero number describing the number of bytes remaining in a string slice. This can be seen
/// as the complement of a 0-based index, and for such an index `i` into a string `s` the
/// expression `s.len() - i` will return the corresponding number of remaining bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytesRemaining(NonZero<usize>);

impl From<NonZero<usize>> for BytesRemaining {
    #[inline(always)]
    fn from(value: NonZero<usize>) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for BytesRemaining {
    type Error = ();

    #[inline(always)]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        NonZero::new(value).ok_or(()).map(Self)
    }
}

impl BytesRemaining {
    #[inline(always)]
    pub fn of(value: impl AsRef<[u8]>) -> Option<Self> {
        NonZero::new(value.as_ref().len()).map(BytesRemaining)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFullError {
    UnexpectedEof,
    UnexpectedByte(u8, BytesRemaining),
    InvalidSplitIndex(usize, BytesRemaining),
    RemainingInput(BytesRemaining),
}

/// An error which may relate to parsing or else be semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError<E> {
    UnexpectedEof,
    UnexpectedByte(u8, BytesRemaining),
    InvalidSplitIndex(usize, BytesRemaining),
    Semantic(E),
}

impl<E> From<E> for ParseError<E> {
    fn from(v: E) -> Self {
        Self::Semantic(v)
    }
}

impl<E> ParseError<E> {
    fn coerce<E2>(self) -> ParseError<E2>
    where
        E: Into<E2>,
    {
        match self {
            ParseError::UnexpectedEof => ParseError::UnexpectedEof,
            ParseError::UnexpectedByte(b, br) => ParseError::UnexpectedByte(b, br),
            ParseError::InvalidSplitIndex(i, br) => ParseError::InvalidSplitIndex(i, br),
            ParseError::Semantic(error) => ParseError::Semantic(error.into()),
        }
    }

    fn into_parse_full_error(self, tail: &str) -> Result<E, ParseFullError> {
        match NonZero::new(tail.len()) {
            Some(n) => Err(ParseFullError::RemainingInput(BytesRemaining(n))),
            None => match self {
                ParseError::UnexpectedEof => Err(ParseFullError::UnexpectedEof),
                ParseError::UnexpectedByte(b, br) => Err(ParseFullError::UnexpectedByte(b, br)),
                ParseError::InvalidSplitIndex(i, br) => {
                    Err(ParseFullError::InvalidSplitIndex(i, br))
                }
                ParseError::Semantic(error) => Ok(error),
            },
        }
    }
}

pub trait DateTimeMarker: Sized {
    fn parser(input: &mut &str) -> Result<Self, ParseError<InvalidDateTimeError<Self>>>;
}

impl DateTimeMarker for Utc {
    fn parser(input: &mut &str) -> Result<Self, ParseError<InvalidDateTimeError<Self>>> {
        let [byte] = take_bytes(input).copied()?;

        match byte {
            b'Z' => Ok(Utc),
            _ => Err(ParseError::Semantic(InvalidDateTimeError::Marker(
                Default::default(),
            ))),
        }
    }
}

impl DateTimeMarker for Local {
    #[inline(always)]
    fn parser(_input: &mut &str) -> Result<Self, ParseError<InvalidDateTimeError<Self>>> {
        // immediately succeed without advancing the input
        Ok(Local)
    }
}

pub fn date_time<M: DateTimeMarker>(
    input: &mut &str,
) -> Result<DateTime<M>, ParseError<InvalidDateTimeError<M>>> {
    let date = date(input).map_err(ParseError::coerce)?;
    let _ = byte_where(input, |b| b == b'T')?;
    let time = time(input).map_err(ParseError::coerce)?;
    let marker = M::parser(input)?;
    Ok(DateTime { date, time, marker })
}

pub fn date(input: &mut &str) -> Result<Date, ParseError<InvalidDateError>> {
    let year = year(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let month = month(input).map_err(ParseError::coerce)?;
    let () = hyphen(input)?;
    let day = day(input).map_err(ParseError::coerce)?;

    Date::new(year, month, day).map_err(Into::into)
}

pub fn year(input: &mut &str) -> Result<Year, ParseError<InvalidYearError>> {
    let a = digit(input)? as u16;
    let b = digit(input)? as u16;
    let c = digit(input)? as u16;
    let d = digit(input)? as u16;

    let value = (a * 1000) + (b * 100) + (c * 10) + d;
    Year::new(value).map_err(Into::into)
}

pub fn month(input: &mut &str) -> Result<Month, ParseError<InvalidMonthError>> {
    let a = digit(input)?;
    let b = digit(input)?;
    let value = (a * 10) + b;
    Month::new(value).map_err(Into::into)
}

pub fn day(input: &mut &str) -> Result<Day, ParseError<InvalidDayError>> {
    let a = digit(input)?;
    let b = digit(input)?;
    let value = (a * 10) + b;
    Day::new(value).map_err(Into::into)
}

pub fn time(input: &mut &str) -> Result<Time, ParseError<InvalidTimeError>> {
    todo!()
}

#[inline(always)]
fn hyphen<E>(input: &mut &str) -> Result<(), ParseError<E>> {
    let _ = byte_where(input, |b| b == b'-')?;
    Ok(())
}

// COMBINATORS

/// Parses a single digit.
fn digit<E>(input: &mut &str) -> Result<u8, ParseError<E>> {
    let byte = byte_where(input, |b| b.is_ascii_digit())?;
    Ok(byte - b'0')
}

/// Parses a single byte matching the given predicate. If removing the first byte from the input
/// would produce an invalid string, the parser will panic.
fn byte_where<E>(input: &mut &str, f: impl FnOnce(u8) -> bool) -> Result<u8, ParseError<E>> {
    let bytes_remaining = BytesRemaining::of(&input).ok_or(ParseError::UnexpectedEof)?;
    let [byte] = take_bytes(input).copied()?;

    match f(byte) {
        true => Ok(byte),
        false => Err(ParseError::UnexpectedByte(byte, bytes_remaining)),
    }
}

/// Takes the first `N` bytes from the `input`.
fn take_bytes<'a, E, const N: usize>(input: &mut &'a str) -> Result<&'a [u8; N], ParseError<E>> {
    let bytes = input
        .as_bytes()
        .first_chunk::<N>()
        .ok_or(ParseError::UnexpectedEof)?;
    let () = skip(input, N)?;
    Ok(bytes)
}

/// Skips the next `count` bytes in the `input`.
fn skip<E>(input: &mut &str, count: usize) -> Result<(), ParseError<E>> {
    take_str(input, count).and(Ok(()))
}

/// Takes the first `count` bytes from the `input`, returning an error if removing these inputs
/// would leave the `input` in an invalid state.
fn take_str<'i, E>(input: &mut &'i str, count: usize) -> Result<&'i str, ParseError<E>> {
    match input.len().cmp(&count) {
        Ordering::Less => Err(ParseError::UnexpectedEof),
        Ordering::Equal => {
            let (head, tail) = (*input, "");
            *input = tail;
            Ok(head)
        }
        Ordering::Greater => {
            let bytes_remaining = BytesRemaining::from(NonZero::new(input.len()).unwrap());

            match input.split_at_checked(count) {
                None => Err(ParseError::InvalidSplitIndex(count, bytes_remaining)),
                Some((head, tail)) => {
                    *input = tail;
                    Ok(head)
                }
            }
        }
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
                    assert_eq!(parser(&input), Ok(Date::new(year, month, day)));
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
        assert_eq!(month(&mut ""), Err(ParseError::UnexpectedEof));
        assert_eq!(month(&mut "0"), Err(ParseError::UnexpectedEof));
        assert_eq!(month(&mut "1"), Err(ParseError::UnexpectedEof));
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
        assert_eq!(day(&mut ""), Err(ParseError::UnexpectedEof));
        assert_eq!(day(&mut "0"), Err(ParseError::UnexpectedEof));
        assert_eq!(day(&mut "1"), Err(ParseError::UnexpectedEof));
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
        assert_eq!(digit::<()>(&mut ""), Err(ParseError::UnexpectedEof));

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
            Err(ParseError::UnexpectedByte(b'A', 1.try_into().unwrap()))
        );

        assert_eq!(digit::<()>(&mut "0dgsahjk"), Ok(0));
        assert_eq!(digit::<()>(&mut "15674352756743"), Ok(1));
        assert_eq!(digit::<()>(&mut "2    "), Ok(2));
        assert_eq!(digit::<()>(&mut "3\t\t\t\t"), Ok(3));
        assert_eq!(digit::<()>(&mut "4cbzxnmbc"), Ok(4));
        assert_eq!(digit::<()>(&mut "59888988"), Ok(5));
        assert_eq!(
            digit::<()>(&mut "A0"),
            Err(ParseError::UnexpectedByte(b'A', 2.try_into().unwrap()))
        );
    }

    #[test]
    fn skip_parser() {
        let input = &mut "0123456789ABCDEF";

        assert_eq!(skip::<()>(input, 0), Ok(()));
        assert_eq!(*input, "0123456789ABCDEF");
        assert_eq!(skip::<()>(input, 4), Ok(()));
        assert_eq!(*input, "456789ABCDEF");
        assert_eq!(skip::<()>(input, 6), Ok(()));
        assert_eq!(*input, "ABCDEF");
        assert_eq!(skip::<()>(input, 5), Ok(()));
        assert_eq!(*input, "F");
        assert_eq!(skip::<()>(input, 1), Ok(()));
        assert_eq!(*input, "");
        assert_eq!(skip::<()>(input, 0), Ok(()));
        assert_eq!(*input, "");
        assert_eq!(skip::<()>(input, 1), Err(ParseError::UnexpectedEof));
    }

    #[test]
    fn take_str_parser() {
        let input = &mut "abcdαβγδ";

        assert_eq!(take_str::<()>(input, 0), Ok(""));
        assert_eq!(*input, "abcdαβγδ");
        assert_eq!(take_str::<()>(input, 2), Ok("ab"));
        assert_eq!(*input, "cdαβγδ");
        assert_eq!(take_str::<()>(input, 2), Ok("cd"));
        assert_eq!(*input, "αβγδ");
        assert_eq!(take_str::<()>(input, 2), Ok("α"));
        assert_eq!(*input, "βγδ");

        assert_eq!(
            take_str::<()>(input, 3),
            Err(ParseError::InvalidSplitIndex(3, 6.try_into().unwrap()))
        );
        assert_eq!(*input, "βγδ");

        assert_eq!(take_str::<()>(input, 4), Ok("βγ"));
        assert_eq!(*input, "δ");

        assert_eq!(
            take_str::<()>(input, 1),
            Err(ParseError::InvalidSplitIndex(1, 2.try_into().unwrap()))
        );
        assert_eq!(*input, "δ");

        assert_eq!(take_str::<()>(input, 3), Err(ParseError::UnexpectedEof));
        assert_eq!(*input, "δ");
        assert_eq!(take_str::<()>(input, 2), Ok("δ"));
        assert_eq!(*input, "");
        assert_eq!(take_str::<()>(input, 0), Ok(""));
        assert_eq!(*input, "");
    }
}
