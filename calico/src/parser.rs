//! A [`winnow`]-based RFC 5545 parser implementation.

use std::borrow::Cow;

use winnow::{
    Parser,
    ascii::Caseless,
    error::ParserError,
    stream::{AsBytes, Compare, Stream, StreamIsPartial},
    token::take_until,
};

use error::CalendarParseError;

use crate::parser::escaped::split_fold_prefix;

pub mod component;
pub mod config;
pub mod error;
pub mod escaped;
pub mod parameter;
pub mod primitive;
pub mod property;
pub mod rrule;

/// An input stream compatible with the parsers in [`calico::parser`](crate::parser).
pub trait InputStream
where
    Self: StreamIsPartial + Stream + Compare<char> + AsRef<[u8]>,
    for<'a> Self: Compare<Caseless<&'a str>> + Compare<&'a str>,
{
    type Str: Clone + AsRef<str> + Into<String> + Into<Box<str>>;

    fn try_into_str(slice: &Self::Slice) -> Result<Self::Str, CalendarParseError<Self::Slice>>;
    fn try_into_string(slice: &Self::Slice) -> Result<String, CalendarParseError<Self::Slice>>;
    fn as_bytes(slice: &Self::Slice) -> &[u8];
    fn str_from_static_str(s: &'static str) -> Self::Str;

    /// Removes as many line folds from the prefix of `self` as possible, and returns the number of
    /// bytes removed (this will always be a multiple of three).
    fn strip_line_fold_prefix(&mut self) -> usize;

    /// Returns the longest contiguous prefix of `self`. A slice is contiguous if it does not
    /// contain line folds or newlines. If the input does not contain the sequence `\r\n` anywhere,
    /// the entire input is returned as a slice.
    fn next_contiguous_slice<E>(input: &mut Self) -> Result<Self::Slice, E>
    where
        E: ParserError<Self>;
}

impl InputStream for &str {
    type Str = Self;

    #[inline(always)]
    fn try_into_str(slice: &Self::Slice) -> Result<Self::Str, CalendarParseError<Self::Slice>> {
        Ok(slice)
    }

    #[inline(always)]
    fn try_into_string(slice: &Self::Slice) -> Result<String, CalendarParseError<Self::Slice>> {
        Ok(slice.to_string())
    }

    #[inline(always)]
    fn as_bytes(slice: &Self::Slice) -> &[u8] {
        slice.as_bytes()
    }

    #[inline(always)]
    fn str_from_static_str(s: &'static str) -> Self::Str {
        s
    }

    #[inline(always)]
    fn strip_line_fold_prefix(&mut self) -> usize {
        let (prefix, tail) = split_fold_prefix(self.as_bytes());
        *self = str::from_utf8(tail).expect("tail represents a valid UTF-8 string slice");
        prefix.len()
    }

    #[inline(always)]
    fn next_contiguous_slice<E>(input: &mut Self) -> Result<Self::Slice, E>
    where
        E: ParserError<Self>,
    {
        let _ = input.strip_line_fold_prefix();

        match take_until(0.., "\r\n").parse_next(input) {
            Ok(slice) => Ok(slice),
            Err(()) => Ok(input.finish()),
        }
    }
}

impl<'a> InputStream for &'a [u8] {
    type Str = &'a str;

    #[inline(always)]
    fn try_into_str(slice: &Self::Slice) -> Result<Self::Str, CalendarParseError<Self::Slice>> {
        std::str::from_utf8(slice).map_err(Into::into)
    }

    #[inline(always)]
    fn try_into_string(slice: &Self::Slice) -> Result<String, CalendarParseError<Self::Slice>> {
        Self::try_into_str(slice).map(ToString::to_string)
    }

    #[inline(always)]
    fn as_bytes(slice: &Self::Slice) -> &[u8] {
        slice
    }

    #[inline(always)]
    fn str_from_static_str(s: &'static str) -> Self::Str {
        s
    }

    #[inline(always)]
    fn strip_line_fold_prefix(&mut self) -> usize {
        let (prefix, tail) = split_fold_prefix(self);
        *self = tail;
        prefix.len()
    }

    #[inline(always)]
    fn next_contiguous_slice<E>(input: &mut Self) -> Result<Self::Slice, E>
    where
        E: ParserError<Self>,
    {
        let _ = input.strip_line_fold_prefix();

        match take_until(0.., "\r\n").parse_next(input) {
            Ok(slice) => Ok(slice),
            Err(()) => Ok(input.finish()),
        }
    }
}

impl<'a> InputStream for escaped::Escaped<'a> {
    type Str = Cow<'a, str>;

    #[inline(always)]
    fn try_into_str(slice: &Self::Slice) -> Result<Self::Str, CalendarParseError<Self::Slice>> {
        slice.try_into_cow_str().map_err(Into::into)
    }

    #[inline(always)]
    fn try_into_string(slice: &Self::Slice) -> Result<String, CalendarParseError<Self::Slice>> {
        Self::try_into_str(slice).map(|s| s.into_owned())
    }

    #[inline(always)]
    fn as_bytes(slice: &Self::Slice) -> &[u8] {
        slice.as_bytes()
    }

    #[inline(always)]
    fn str_from_static_str(s: &'static str) -> Self::Str {
        Cow::Borrowed(s)
    }

    #[inline(always)]
    fn strip_line_fold_prefix(&mut self) -> usize {
        let (prefix, tail) = split_fold_prefix(self.0);
        self.0 = tail;
        prefix.len()
    }

    #[inline(always)]
    fn next_contiguous_slice<E>(input: &mut Self) -> Result<Self::Slice, E>
    where
        E: ParserError<Self>,
    {
        // the semantics are slightly different here, because the line fold sequence does not occur
        // as part of the Escaped stream; rather it occurs as part of the underlying byte slice.

        let _ = input.strip_line_fold_prefix();
        let indices = <&[u8] as winnow::stream::FindSlice<_>>::find_slice(&input.0, "\r\n");

        match indices {
            Some(i) => Ok(input.next_slice(i.start)),
            None => Ok(input.finish()),
        }
    }
}
