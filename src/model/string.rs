//! String data model types.

use std::{borrow::Cow, num::NonZero};

use dizzy::DstNewtype;

/// A string slice satisfying the regex `/[A-Za-z0-9\-\_]{1, 255}/` (RFC 8984 §1.4.1).
///
/// # Invariants
/// 1. The underlying string has at least 1 and at most 255 characters.
/// 2. All the characters of the string correspond to the variants of [`IdChar`].
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = Id::check_slice, error = InvalidIdError)]
#[dizzy(constructor = pub const try_from_slice)]
#[dizzy(unsafe_constructor = const from_slice_unchecked)]
#[dizzy(getter = pub const as_slice)]
#[dizzy(derive(CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct Id([IdChar]);

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <str as std::fmt::Debug>::fmt(self.as_str(), f)
    }
}

impl Id {
    const fn check_slice(value: &[IdChar]) -> Result<(), InvalidIdError> {
        match value.len() {
            0 => Err(InvalidIdError::EmptyString),
            1..256 => Ok(()),
            _ => Err(InvalidIdError::TooLong),
        }
    }

    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: two slices have the same layout iff their parameter types have the same layout.
        // IdChar has repr(u8), so this is satisfied, and moreover every value of IdChar is valid
        // as a byte
        unsafe { std::mem::transmute::<&[IdChar], &[u8]>(self.as_slice()) }
    }

    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        let bytes: &[u8] = self.as_bytes();
        debug_assert!(bytes.is_ascii());

        // SAFETY: the bytes of `self` must have the values of the variants of IdChar, which are
        // all valid ASCII bytes
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    #[inline(always)]
    pub const fn len(&self) -> NonZero<u8> {
        let length = self.0.len();
        debug_assert!(length != 0 && length <= 255);

        // SAFETY: the length must be nonzero because the return type of NonEmpty::len is nonzero,
        // and `length as u8` can never overflow because it is an invariant of Id that its length
        // must be less than 256
        unsafe { NonZero::new_unchecked(length as u8) }
    }

    /// Tries to construct an [`Id`] reference from the given `value`, failing if it does not
    /// satisfy the [invariants](Self#Invariants).
    #[inline(always)]
    pub const fn new(value: &str) -> Result<&Self, InvalidIdError> {
        match value.len() {
            0 => Err(InvalidIdError::EmptyString),
            256.. => Err(InvalidIdError::TooLong),
            _ => match Id::invalid_char_index(value) {
                Some(index) => Err(InvalidIdError::InvalidChar { index }),
                None => {
                    // SAFETY: we know that the length of `value` is more than 0 and less than 256
                    // from the outer match statement, and in this branch we know that there is no
                    // index in the first 255 bytes of `value` where the corresponding character is
                    // not valid as an IdChar
                    unsafe { Ok(Id::new_unchecked(value)) }
                }
            },
        }
    }

    /// Converts the given `value` reference into an [`Id`] reference without checking invariants.
    ///
    /// # Safety
    /// `value` must have a length of at least 1 and at most 255, and the bytes of `value` must all
    /// be valid when interpreted as [`IdChar`] values.
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: &str) -> &Self {
        debug_assert!(!value.is_empty() && value.len() <= 255);
        debug_assert!(Id::invalid_char_index(value).is_none());

        let bytes = value.as_bytes();

        // SAFETY: consider each of the lines individually:
        // 1. the first line is sound because &[u8] and &[IdChar] have the same layout,
        //    and we have as an invariant that the bytes of the string are all valid
        //    when interpreted as IdChar values
        // 3. the second line is sound because Id is a transparent newtype of [IdChar]
        unsafe {
            let chars = std::mem::transmute::<&[u8], &[IdChar]>(bytes);
            Id::from_slice_unchecked(chars)
        }
    }

    /// Returns the first index in the first 255 bytes of `value` for which a character which is
    /// not an [`IdChar`] occurs; if no such index exists then `None` is returned. If this method
    /// returns `None` then all the characters of the given `value` satisfy [`IdChar::contains`].
    #[inline(always)]
    const fn invalid_char_index(value: &str) -> Option<u8> {
        if value.is_empty() {
            return None;
        }

        // we have to use a while loop here because for loops cannot occur in const contexts, and
        // similarly we cannot call usize::min to compute `end` here because it is a trait method

        let mut i: usize = 0;
        let end = if value.len() < 255 { value.len() } else { 255 };

        while i < end {
            let c = value.as_bytes()[i] as char;

            if IdChar::contains(c) {
                i += 1;
                continue;
            } else {
                debug_assert!(i <= 255);
                return Some(i as u8);
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum InvalidIdError {
    InvalidChar { index: u8 },
    EmptyString,
    TooLong,
}

/// A character which may occur in an [`Id`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum IdChar {
    Hyphen = 0x2C,
    Zero = 0x30,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    UpperA = 0x41,
    UpperB,
    UpperC,
    UpperD,
    UpperE,
    UpperF,
    UpperG,
    UpperH,
    UpperI,
    UpperJ,
    UpperK,
    UpperL,
    UpperM,
    UpperN,
    UpperO,
    UpperP,
    UpperQ,
    UpperR,
    UpperS,
    UpperT,
    UpperU,
    UpperV,
    UpperW,
    UpperX,
    UpperY,
    UpperZ,
    Underscore = 0x5F,
    LowerA = 0x61,
    LowerB,
    LowerC,
    LowerD,
    LowerE,
    LowerF,
    LowerG,
    LowerH,
    LowerI,
    LowerJ,
    LowerK,
    LowerL,
    LowerM,
    LowerN,
    LowerO,
    LowerP,
    LowerQ,
    LowerR,
    LowerS,
    LowerT,
    LowerU,
    LowerV,
    LowerW,
    LowerX,
    LowerY,
    LowerZ,
}

impl IdChar {
    #[inline(always)]
    pub const fn into_char(self) -> char {
        (self as u8) as char
    }

    #[inline(always)]
    pub const fn contains(value: char) -> bool {
        value == '-' || value == '_' || value.is_ascii_alphanumeric()
    }
}

impl std::fmt::Debug for IdChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <char as std::fmt::Debug>::fmt(&self.into_char(), f)
    }
}

// TODO: define (at a minimum) the following string types:
// 1. TimeZoneId (RFC 8984 §1.4.8, §4.7)
// 2. JsonPointer (RFC 8984 §1.4.9, RFC 6901 §3)
// 3. a URI type (maybe use iri-string?)

#[derive(Debug, Clone, Copy)]
pub enum InvalidImplicitJsonPointerError {
    /// A tilde (`~`) occurred without being immediately followed by `0` or `1` at this index.
    BareTilde { index: usize },
}

/// An implicit unevaluated JSON pointer (RFC 8984 §1.4.9).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = ImplicitJsonPointer::str_is_implicit_json_pointer)]
#[dizzy(error = InvalidImplicitJsonPointerError)]
#[dizzy(constructor = pub new)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub ImplicitJsonPointerBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct ImplicitJsonPointer(str);

impl ImplicitJsonPointer {
    fn str_is_implicit_json_pointer(s: &str) -> Result<(), InvalidImplicitJsonPointerError> {
        let mut iter = s.char_indices().peekable();
        while let Some((index, c)) = iter.next() {
            if c == '~' && iter.peek().is_none_or(|(_, c)| c != &'0' && c != &'1') {
                return Err(InvalidImplicitJsonPointerError::BareTilde { index });
            }
        }

        Ok(())
    }

    pub fn segments(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.0.split('/').map(|s| {
            let mut buf = Cow::Borrowed("");
            let mut tail = s;

            while !tail.is_empty() {
                match tail.split_once('~') {
                    Some((head, new_tail)) => {
                        buf += head;
                        let mut tail_chars = new_tail.chars();
                        let digit = tail_chars.next().expect("~ must be followed by a char");
                        let new_tail = tail_chars.as_str();
                        tail = new_tail;

                        buf += match digit {
                            '0' => "~",
                            '1' => "/",
                            _ => unreachable!(),
                        };
                    }
                    None => {
                        buf += tail;
                        tail = "";
                    }
                }
            }

            buf
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_json_pointer_segmentation() {
        let ptr = ImplicitJsonPointer::new("foo/0/~0/a~1b").unwrap();
        let mut iter = ptr.segments();

        assert_eq!(iter.next(), Some(Cow::Borrowed("foo")));
        assert_eq!(iter.next(), Some(Cow::Borrowed("0")));
        assert_eq!(iter.next(), Some(Cow::Borrowed("~")));
        assert_eq!(iter.next(), Some(Cow::Owned(String::from("a/b"))));
        assert!(iter.next().is_none());
    }
}
