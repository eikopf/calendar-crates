//! A custom [`Stream`] for skipping line folds.

use std::borrow::Cow;

use winnow::{
    ascii::Caseless,
    error::Needed,
    stream::{
        AsBStr, AsBytes, Checkpoint, Compare, CompareResult, Offset, SliceLen, Stream,
        StreamIsPartial,
    },
};

// TODO: refactor this module to provide EscapedStr and EscapedBytes newtypes of str and [u8], and
// update AsEscaped to return a reference to an associated type which can be set to one of these
// two

pub trait AsEscaped {
    fn as_escaped<'a>(&'a self) -> Escaped<'a>;
}

impl AsEscaped for str {
    fn as_escaped<'a>(&'a self) -> Escaped<'a> {
        Escaped(self.as_bytes())
    }
}

impl AsEscaped for [u8] {
    fn as_escaped<'a>(&'a self) -> Escaped<'a> {
        Escaped(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Escaped<'a>(pub(crate) &'a [u8]);

impl<'a> Escaped<'a> {
    pub fn len(&self) -> usize {
        // If the only remaining bytes are fold sequences with no content
        // after them, report 0 length (effectively EOF).
        let (_, tail) = split_fold_prefix(self.0);
        if tail.is_empty() {
            0
        } else {
            self.0.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts `self` into an escaped, possibly owned sequence of bytes.
    pub fn into_escaped_bytes(&self) -> Cow<'a, [u8]> {
        let (_, input) = split_fold_prefix(self.0);
        let mut bytes = None;
        let mut i = 0;

        while i < input.len() {
            // Check for CRLF fold: \r\n followed by SP or HTAB
            if i + 2 < input.len()
                && input[i] == b'\r'
                && input[i + 1] == b'\n'
                && (input[i + 2] == b' ' || input[i + 2] == b'\t')
            {
                bytes.get_or_insert_with(|| {
                    let mut v = Vec::with_capacity(input.len());
                    v.extend_from_slice(&input[..i]);
                    v
                });
                i += 3;
            // Check for bare LF fold: \n followed by SP or HTAB
            } else if i + 1 < input.len()
                && input[i] == b'\n'
                && (input[i + 1] == b' ' || input[i + 1] == b'\t')
            {
                bytes.get_or_insert_with(|| {
                    let mut v = Vec::with_capacity(input.len());
                    v.extend_from_slice(&input[..i]);
                    v
                });
                i += 2;
            } else {
                if let Some(ref mut v) = bytes {
                    v.push(input[i]);
                }
                i += 1;
            }
        }

        bytes.map(Cow::Owned).unwrap_or(Cow::Borrowed(input))
    }

    /// Converts `self` into an escaped, possibly owned string. If the underlying bytes do not
    /// escape as a valid UTF-8 string, this method will fail.
    pub fn try_into_cow_str(&self) -> Result<Cow<'a, str>, std::str::Utf8Error> {
        match self.into_escaped_bytes() {
            Cow::Borrowed(bytes) => str::from_utf8(bytes).map(Cow::Borrowed),
            Cow::Owned(bytes) => String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|e| e.utf8_error()),
        }
    }
}

impl<'a> std::fmt::Debug for Escaped<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <winnow::BStr as std::fmt::Debug>::fmt(winnow::BStr::new(self.0), f)
    }
}

impl<'a> AsRef<[u8]> for Escaped<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> AsBStr for Escaped<'a> {
    fn as_bstr(&self) -> &[u8] {
        self.0.as_bstr()
    }
}

impl<'a> AsBytes for Escaped<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.0
    }
}

impl<'a> SliceLen for Escaped<'a> {
    fn slice_len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> Offset<Checkpoint<&'a [u8], &'a [u8]>> for Escaped<'a> {
    fn offset_from(&self, other: &Checkpoint<&'a [u8], &'a [u8]>) -> usize {
        self.checkpoint().offset_from(other)
    }
}

impl<'a> Stream for Escaped<'a> {
    type Token = u8;

    type Slice = Self;

    type IterOffsets = IterOffsets<'a>;

    type Checkpoint = Checkpoint<&'a [u8], &'a [u8]>;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        IterOffsets::new(self.0)
    }

    #[inline(always)]
    fn eof_offset(&self) -> usize {
        // If the only remaining bytes are fold sequences with no content
        // after them, report EOF (0) so that `eof` works correctly.
        let (_, tail) = split_fold_prefix(self.0);
        if tail.is_empty() {
            0
        } else {
            self.0.len()
        }
    }

    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        let (_escapes, tail) = split_fold_prefix(self.0);

        match tail {
            [] => {
                // Consume any trailing fold bytes so eof_offset() reports 0
                self.0 = tail;
                None
            }
            [t, tail @ ..] => {
                self.0 = tail;
                Some(*t)
            }
        }
    }

    #[inline(always)]
    fn peek_token(&self) -> Option<Self::Token> {
        split_fold_prefix(self.0).1.first().copied()
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.iter_offsets()
            .find_map(|(offset, token)| match predicate(token) {
                true => Some(offset),
                false => None,
            })
    }

    #[inline(always)]
    fn offset_at(&self, ntokens: usize) -> Result<usize, Needed> {
        match ntokens {
            0 => Ok(0),
            n => self
                .iter_offsets()
                .nth(n - 1)
                .map(|(offset, _)| offset + 1)
                .ok_or(Needed::Unknown),
        }
    }

    #[inline(always)]
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        let (head, tail) = self.0.split_at(offset);
        self.0 = tail;
        Escaped(head)
    }

    #[inline(always)]
    fn peek_slice(&self, offset: usize) -> Self::Slice {
        let (head, _tail) = self.0.split_at(offset);
        Escaped(head)
    }

    #[inline(always)]
    fn checkpoint(&self) -> Self::Checkpoint {
        self.0.checkpoint()
    }

    #[inline(always)]
    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        self.0.reset(checkpoint)
    }

    #[inline(always)]
    fn raw(&self) -> &dyn std::fmt::Debug {
        self
    }
}

impl<'a> StreamIsPartial for Escaped<'a> {
    type PartialState = ();

    fn complete(&mut self) -> Self::PartialState {
        // already complete
    }

    fn restore_partial(&mut self, _state: Self::PartialState) {
        // already complete
    }

    fn is_partial_supported() -> bool {
        false
    }
}

impl<'a, 'b> Compare<&'b [u8]> for Escaped<'a> {
    fn compare(&self, t: &'b [u8]) -> CompareResult {
        // TODO: hacky impl, should really use a custom iterator type
        let bytes = self.iter_offsets().map(|(_, c)| c);

        if t.iter().zip(bytes).any(|(&l, r)| l != r) {
            CompareResult::Error
        // WARN: this is probably incorrect (but only for streaming)!
        } else if self.0.len() < t.slice_len() {
            CompareResult::Incomplete
        } else {
            match self.offset_at(t.len()) {
                Ok(size) => CompareResult::Ok(size),
                Err(Needed::Unknown) => CompareResult::Incomplete,
                Err(Needed::Size(_)) => unreachable!(),
            }
        }
    }
}

impl<'a, 'b> Compare<Caseless<&'b [u8]>> for Escaped<'a> {
    fn compare(&self, t: Caseless<&'b [u8]>) -> CompareResult {
        // TODO: hacky impl, should really use a custom iterator type
        let bytes = self.iter_offsets().map(|(_, c)| c);

        if t.0
            .iter()
            .zip(bytes)
            .any(|(l, r)| !r.eq_ignore_ascii_case(l))
        {
            CompareResult::Error
        // WARN: this is probably incorrect (but only for streaming)!
        } else if self.0.len() < t.slice_len() {
            CompareResult::Incomplete
        } else {
            match self.offset_at(t.0.len()) {
                Ok(size) => CompareResult::Ok(size),
                Err(Needed::Unknown) => CompareResult::Incomplete,
                Err(Needed::Size(_)) => unreachable!(),
            }
        }
    }
}

impl<'a, 'b> Compare<&'b str> for Escaped<'a> {
    fn compare(&self, t: &'b str) -> CompareResult {
        self.compare(t.as_bytes())
    }
}

impl<'a, 'b> Compare<Caseless<&'b str>> for Escaped<'a> {
    fn compare(&self, t: Caseless<&'b str>) -> CompareResult {
        self.compare(t.as_bytes())
    }
}

impl<'a> Compare<char> for Escaped<'a> {
    fn compare(&self, c: char) -> winnow::stream::CompareResult {
        self.compare(c.encode_utf8(&mut [0; 4]).as_bytes())
    }
}

impl<'a> Compare<Caseless<u8>> for Escaped<'a> {
    fn compare(&self, t: Caseless<u8>) -> CompareResult {
        match self.iter_offsets().next() {
            Some((size, c)) if t.0.eq_ignore_ascii_case(&c) => CompareResult::Ok(size),
            Some(_) => CompareResult::Error,
            None => CompareResult::Incomplete,
        }
    }
}

impl<'a> Compare<Caseless<char>> for Escaped<'a> {
    fn compare(&self, t: Caseless<char>) -> CompareResult {
        self.compare(Caseless(t.0.encode_utf8(&mut [0; 4]).as_bytes()))
    }
}

impl<'a, 'b> Compare<Caseless<Escaped<'b>>> for Escaped<'a> {
    fn compare(&self, t: Caseless<Escaped<'b>>) -> CompareResult {
        let mut lhs = self.iter_offsets();
        let mut rhs = t.0.iter_offsets().map(|(_, t)| t);

        loop {
            match (lhs.next(), rhs.next()) {
                (None, None) => break CompareResult::Ok(self.len()),
                (None, Some(_)) => break CompareResult::Incomplete,
                (Some((offset, _)), None) => break CompareResult::Ok(offset),
                (Some((_, l)), Some(r)) => {
                    if !l.eq_ignore_ascii_case(&r) {
                        break CompareResult::Error;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IterOffsets<'a> {
    offset: usize,
    source: &'a [u8],
}

impl<'a> IterOffsets<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { offset: 0, source }
    }
}

impl<'a> Iterator for IterOffsets<'a> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.split_at_checked(self.offset) {
            Some((_, tail)) => {
                let (escape_prefix, tail) = split_fold_prefix(tail);

                match tail {
                    [] => None,
                    [token, ..] => {
                        self.offset += escape_prefix.len();
                        let offset = self.offset;
                        self.offset += 1;
                        Some((offset, *token))
                    }
                }
            }
            None => None,
        }
    }
}

/// Splits the `input` at the longest foldable prefix.
///
/// A fold sequence is either `\r\n` followed by a space or tab (3 bytes),
/// or a bare `\n` followed by a space or tab (2 bytes).
#[inline(always)]
pub(crate) fn split_fold_prefix(input: &[u8]) -> (&[u8], &[u8]) {
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'\r' && i + 2 < input.len() && input[i + 1] == b'\n' && (input[i + 2] == b' ' || input[i + 2] == b'\t') {
            i += 3;
        } else if input[i] == b'\n' && i + 1 < input.len() && (input[i + 1] == b' ' || input[i + 1] == b'\t') {
            i += 2;
        } else {
            break;
        }
    }
    input.split_at(i)
}

#[cfg(test)]
mod tests {
    use winnow::{
        Parser,
        token::{literal, take},
    };

    use super::*;

    #[test]
    fn compare_escaped_caseless() {
        assert_eq!(
            "abc".as_escaped().compare(Caseless("ABC".as_escaped())),
            CompareResult::Ok(3)
        );

        let input = "\r\n\ta\r\n b\r\n\tcd\r\n\te".as_escaped();
        let res: Result<_, ()> = literal(Caseless("ABCD".as_escaped())).parse_peek(input);
        assert!(res.is_ok());
    }

    #[test]
    fn compare_str_caseless() {
        let input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());
        let res: Result<_, ()> =
            (Caseless("A"), Caseless("B"), Caseless("C"), Caseless("D")).parse_peek(input);

        assert!(res.is_ok());

        let res: Result<_, ()> = ("A", "B", "C", "D").parse_peek(input);

        assert!(res.is_err());
    }

    #[test]
    fn compare_char() {
        let input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());

        let res: Result<_, ()> = ('a', 'b', 'c', 'd').parse_peek(input);
        assert_eq!(
            res,
            Ok((Escaped("\r\n\te".as_bytes()), ('a', 'b', 'c', 'd')))
        );

        let res: Result<_, ()> = ('a', 'b', 'c', 'd', 'e').parse_peek(input);
        assert_eq!(res, Ok((Escaped("".as_bytes()), ('a', 'b', 'c', 'd', 'e'))));
    }

    #[test]
    fn take_parser() {
        let input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());

        assert_eq!(
            take::<usize, _, ()>(0).parse_peek(input),
            Ok((input, "".as_escaped())),
        );

        assert_eq!(
            take::<usize, _, ()>(1).parse_peek(input),
            Ok(("\r\n b\r\n\tcd\r\n\te".as_escaped(), "\r\n\ta".as_escaped())),
        );

        assert_eq!(
            take::<usize, _, ()>(2).parse_peek(input),
            Ok(("\r\n\tcd\r\n\te".as_escaped(), "\r\n\ta\r\n b".as_escaped())),
        );

        assert_eq!(
            take::<usize, _, ()>(3).parse_peek(input),
            Ok(("d\r\n\te".as_escaped(), "\r\n\ta\r\n b\r\n\tc".as_escaped())),
        );

        assert_eq!(
            take::<usize, _, ()>(4).parse_peek(input),
            Ok(("\r\n\te".as_escaped(), "\r\n\ta\r\n b\r\n\tcd".as_escaped())),
        );

        assert_eq!(
            take::<usize, _, ()>(5).parse_peek(input),
            Ok(("".as_escaped(), "\r\n\ta\r\n b\r\n\tcd\r\n\te".as_escaped())),
        );
    }

    #[test]
    fn iter_offsets() {
        assert_eq!(
            IterOffsets::new("abc".as_bytes()).collect::<Vec<_>>(),
            vec![(0, b'a'), (1, b'b'), (2, b'c')]
        );

        assert_eq!(
            IterOffsets::new("\r\n\ta\r\n\tbc".as_bytes()).collect::<Vec<_>>(),
            vec![(3, b'a'), (7, b'b'), (8, b'c')]
        );
    }

    #[test]
    fn next_slice() {
        let mut input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());
        assert_eq!(input.next_slice(0), "".as_escaped());
        assert_eq!(input.next_slice(4), "\r\n\ta".as_escaped());
        assert_eq!(input.next_slice(9), "\r\n b\r\n\tcd".as_escaped());
    }

    #[test]
    fn peek_slice() {
        let input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());
        assert_eq!(input.peek_slice(0), "".as_escaped());
        assert_eq!(input.peek_slice(4), "\r\n\ta".as_escaped());
        assert_eq!(input.peek_slice(8), "\r\n\ta\r\n b".as_escaped());
        assert_eq!(input.peek_slice(13), "\r\n\ta\r\n b\r\n\tcd".as_escaped());
    }

    #[test]
    fn offset_at() {
        let input = Escaped("\r\n\ta\r\n b\r\n\tcd\r\n\te".as_bytes());
        assert_eq!(input.offset_at(0), Ok(0));
        assert_eq!(input.offset_at(1), Ok(4));
        assert_eq!(input.offset_at(2), Ok(8));
        assert_eq!(input.offset_at(3), Ok(12));
        assert_eq!(input.offset_at(4), Ok(13));
        assert_eq!(input.offset_at(5), Ok(17));
        assert_eq!(input.offset_at(6), Err(Needed::Unknown));
        assert_eq!(input.offset_at(7), Err(Needed::Unknown));
        assert_eq!(input.offset_at(8), Err(Needed::Unknown));
    }

    #[test]
    fn peek_token() {
        let mut input = Escaped("\r\n\ta\r\n b".as_bytes());
        assert_eq!(input.peek_token(), Some(b'a'));
        assert_eq!(input.0, "\r\n\ta\r\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'a'));
        assert_eq!(input.0, "\r\n b".as_bytes());
        assert_eq!(input.peek_token(), Some(b'b'));
        assert_eq!(input.0, "\r\n b".as_bytes());
    }

    #[test]
    fn next_token() {
        assert_eq!(Escaped("".as_bytes()).next_token(), None);
        assert_eq!(Escaped("a".as_bytes()).next_token(), Some(b'a'));
        assert_eq!(Escaped("\r\n\ta".as_bytes()).next_token(), Some(b'a'));
        assert_eq!(Escaped("a\r\n\t".as_bytes()).next_token(), Some(b'a'));

        let mut input = Escaped("\r\n\ta\r\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'a'));
        assert_eq!(input.0, "\r\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'b'));
        assert_eq!(input.0, "".as_bytes());
    }

    #[test]
    fn checkpoints() {
        let mut input = Escaped("a\r\n\tb\r\n\tc".as_bytes());
        let start = input.checkpoint();
        assert_eq!(input.next_token(), Some(b'a'));
        assert_eq!(input.next_token(), Some(b'b'));
        assert_eq!(input.0, "\r\n\tc".as_bytes());
        input.reset(&start);
        assert_eq!(input.0, "a\r\n\tb\r\n\tc".as_bytes());
    }

    #[test]
    fn escaped_try_into_cow_str() {
        let input = b"hel\r\n lo \r\n\t\r\n world!".as_escaped();
        assert_eq!(input.try_into_cow_str(), Ok("hello world!".into()));

        let input = b"\r\n\t\r\n ".as_escaped();
        assert_eq!(input.try_into_cow_str(), Ok("".into()));

        // here the two bytes of λ are separated by "\r\n ", and we expect
        // that after removing this separator we should recover λ
        let input = [206u8, 187, 13, 10, 32].as_escaped();
        assert_eq!(input.try_into_cow_str(), Ok("λ".into()));
    }

    #[test]
    fn fold_prefix_splitting() {
        assert_eq!(
            split_fold_prefix("".as_bytes()),
            ("".as_bytes(), "".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("a".as_bytes()),
            ("".as_bytes(), "a".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("ab".as_bytes()),
            ("".as_bytes(), "ab".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("abc".as_bytes()),
            ("".as_bytes(), "abc".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n ".as_bytes()),
            ("\r\n ".as_bytes(), "".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n\t".as_bytes()),
            ("\r\n\t".as_bytes(), "".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n\t\r\n".as_bytes()),
            ("\r\n\t".as_bytes(), "\r\n".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n\t\r\n\ta".as_bytes()),
            ("\r\n\t\r\n\t".as_bytes(), "a".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n \r\n\tabcabc".as_bytes()),
            ("\r\n \r\n\t".as_bytes(), "abcabc".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n \r\n\tabcab".as_bytes()),
            ("\r\n \r\n\t".as_bytes(), "abcab".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\r\n ab\r\n\t".as_bytes()),
            ("\r\n ".as_bytes(), "ab\r\n\t".as_bytes()),
        );
    }

    // ==================================================================
    // LF fold tests
    // ==================================================================

    #[test]
    fn fold_prefix_splitting_lf() {
        // Single LF fold with space
        assert_eq!(
            split_fold_prefix("\n ".as_bytes()),
            ("\n ".as_bytes(), "".as_bytes()),
        );

        // Single LF fold with tab
        assert_eq!(
            split_fold_prefix("\n\t".as_bytes()),
            ("\n\t".as_bytes(), "".as_bytes()),
        );

        // Two consecutive LF folds
        assert_eq!(
            split_fold_prefix("\n \n\ta".as_bytes()),
            ("\n \n\t".as_bytes(), "a".as_bytes()),
        );

        // LF not followed by SP/HTAB is not a fold
        assert_eq!(
            split_fold_prefix("\na".as_bytes()),
            ("".as_bytes(), "\na".as_bytes()),
        );

        // Mixed CRLF and LF folds
        assert_eq!(
            split_fold_prefix("\r\n \n\ta".as_bytes()),
            ("\r\n \n\t".as_bytes(), "a".as_bytes()),
        );

        assert_eq!(
            split_fold_prefix("\n \r\n\ta".as_bytes()),
            ("\n \r\n\t".as_bytes(), "a".as_bytes()),
        );
    }

    #[test]
    fn into_escaped_bytes_lf_folds() {
        // LF folds within content
        let input = b"hel\n lo \n\t\n world!".as_escaped();
        assert_eq!(input.into_escaped_bytes().as_ref(), b"hello world!");

        // Mixed CRLF and LF folds
        let input = b"ab\r\n cd\n ef".as_escaped();
        assert_eq!(input.into_escaped_bytes().as_ref(), b"abcdef");
    }

    #[test]
    fn iter_offsets_lf_folds() {
        assert_eq!(
            IterOffsets::new("\n\ta\n\tbc".as_bytes()).collect::<Vec<_>>(),
            vec![(2, b'a'), (5, b'b'), (6, b'c')]
        );

        // Mixed
        assert_eq!(
            IterOffsets::new("\r\n\ta\n bc".as_bytes()).collect::<Vec<_>>(),
            vec![(3, b'a'), (6, b'b'), (7, b'c')]
        );
    }

    #[test]
    fn next_token_lf_folds() {
        assert_eq!(Escaped("\n\ta".as_bytes()).next_token(), Some(b'a'));

        let mut input = Escaped("\n\ta\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'a'));
        assert_eq!(input.0, "\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'b'));
        assert_eq!(input.0, "".as_bytes());
    }

    #[test]
    fn peek_token_lf_folds() {
        let mut input = Escaped("\n\ta\n b".as_bytes());
        assert_eq!(input.peek_token(), Some(b'a'));
        assert_eq!(input.0, "\n\ta\n b".as_bytes());
        assert_eq!(input.next_token(), Some(b'a'));
        assert_eq!(input.0, "\n b".as_bytes());
        assert_eq!(input.peek_token(), Some(b'b'));
    }

    #[test]
    fn try_into_cow_str_lf_folds() {
        let input = b"hel\n lo \n\t\n world!".as_escaped();
        assert_eq!(input.try_into_cow_str(), Ok("hello world!".into()));

        // Bare LF fold only prefix
        let input = b"\n\t\n ".as_escaped();
        assert_eq!(input.try_into_cow_str(), Ok("".into()));
    }
}
