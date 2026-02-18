//! String data model types.
//!
//! # TODO
//!
//! - `MediaType`: a MIME media type string (RFC 8984 §4.2.3). Used by `Property::DescriptionContentType`.

use std::{borrow::Cow, fmt::Debug, num::NonZero};

use std::str::FromStr;

pub use calendar_types::string::{InvalidUidError, InvalidUriError, Uid, UidBuf, Uri, UriBuf};
use dizzy::DstNewtype;
use rfc5545_types::string::ParamText;
use thiserror::Error;

/// A BCP 47 language tag (RFC 5646).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageTag(language_tags::LanguageTag);

impl LanguageTag {
    /// Parses a language tag from a string.
    pub fn parse(s: &str) -> Result<Self, language_tags::ParseError> {
        language_tags::LanguageTag::parse(s).map(LanguageTag)
    }

    /// Returns the language tag as a string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the primary language subtag.
    #[inline]
    pub fn primary_language(&self) -> &str {
        self.0.primary_language()
    }
}

impl FromStr for LanguageTag {
    type Err = language_tags::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

use crate::json::{DestructibleJsonValue, TryFromJson, TypeErrorOr};

// TryFromJson impls for reexported string types

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<Uid> {
    type Error = TypeErrorOr<StringError<InvalidUidError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        Uid::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<Uri> {
    type Error = TypeErrorOr<StringError<InvalidUriError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        Uri::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringError<E> {
    input: Box<str>,
    error: E,
}

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

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<Id> {
    type Error = TypeErrorOr<StringError<InvalidIdError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        // NOTE: since the given `value` might be an owned string, it might be better to call
        // `.into()` to get a String without copying and then try to convert that into a Box<Id>

        let input = value.try_into_string()?;

        Id::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into_boxed_str(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

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
                Some((index, c)) => Err(InvalidIdError::InvalidChar { index, c }),
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
    const fn invalid_char_index(value: &str) -> Option<(u8, char)> {
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
                return Some((i as u8, c));
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidIdError {
    #[error("expected an ASCII alphanumeric character, hyphen, or underscore, but got {c} instead")]
    InvalidChar { index: u8, c: char },
    #[error("empty string")]
    EmptyString,
    #[error("string exceeds 255 bytes in length")]
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

/// A custom time zone identifier (RFC 8984 §4.7.2).
///
/// By *custom* we mean that the identifier does not occur in the [IANA Time Zone Database], and
/// this property is guaranteed by requiring that the identifier starts with a forward slash. In
/// addition, we require that the identifier is a valid `paramtext` value ([RFC 5545 §3.1]).
///
/// [IANA Time Zone Database]: https://www.iana.org/time-zones
/// [RFC 5545 §3.1]: https://www.rfc-editor.org/rfc/rfc5545#section-3.1
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = CustomTimeZoneId::str_is_custom_time_zone_id)]
#[dizzy(error = InvalidCustomTimeZoneIdError)]
#[dizzy(constructor = pub new)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct CustomTimeZoneId(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<CustomTimeZoneId> {
    type Error = TypeErrorOr<StringError<InvalidCustomTimeZoneIdError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        CustomTimeZoneId::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl CustomTimeZoneId {
    fn str_is_custom_time_zone_id(s: &str) -> Result<(), InvalidCustomTimeZoneIdError> {
        let body = s.strip_prefix('/').ok_or(if s.is_empty() {
            InvalidCustomTimeZoneIdError::EmptyString
        } else {
            InvalidCustomTimeZoneIdError::MissingSlash
        })?;

        ParamText::new(body).map_err(|e| InvalidCustomTimeZoneIdError::InvalidBodyChar {
            // Adjust index to account for the leading '/'
            index: e.index + 1,
            c: e.c,
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidCustomTimeZoneIdError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("expected a forward slash")]
    MissingSlash,
    #[error("{c} is invalid in a TimeZoneId")]
    InvalidBodyChar { index: usize, c: char },
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidVendorStrError {
    /// The string was empty.
    EmptyString,
    /// A colon occurred at the beginning of the string.
    EmptyPrefix,
    /// The only colon occurred at the end of the string.
    EmptySuffix,
    /// No colon occurred in the string.
    MissingColon,
}

/// A string slice prefixed by a vendor domain name (RFC 8984 §3.3).
///
/// # Invariants
/// 1. The underlying string is not empty.
/// 2. The underlying string contains at least one colon (U+003A) character.
/// 3. After splitting on the first colon, both resulting substrings will not be empty.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = VendorStr::is_vendor_str, error = InvalidVendorStrError)]
#[dizzy(constructor = pub const new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub VendorString(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct VendorStr(str);

impl VendorStr {
    const fn is_vendor_str(s: &str) -> Result<(), InvalidVendorStrError> {
        match s.as_bytes().split_first() {
            None => Err(InvalidVendorStrError::EmptyString),
            Some((b':', _)) => Err(InvalidVendorStrError::EmptyPrefix),
            Some((_, tail)) => match tail.split_last() {
                None => Err(InvalidVendorStrError::MissingColon),
                Some((b':', _)) => Err(InvalidVendorStrError::EmptySuffix),
                Some((_, body)) => {
                    let mut i = 0;

                    while i < body.len() {
                        if body[i] == b':' {
                            return Ok(());
                        }

                        i += 1;
                    }

                    Err(InvalidVendorStrError::MissingColon)
                }
            },
        }
    }

    #[inline(always)]
    pub const fn len(&self) -> NonZero<usize> {
        debug_assert!(!self.as_str().is_empty());

        // SAFETY: it is an invariant of VendorStr that the underlying string slice is not empty
        unsafe { NonZero::new_unchecked(self.as_str().len()) }
    }

    #[inline(always)]
    pub fn split_at_colon(&self) -> (&str, &str) {
        self.as_str()
            .split_once(':')
            .expect("a VendorStr must contain at least one colon")
    }

    #[inline(always)]
    pub fn vendor_domain(&self) -> &str {
        self.split_at_colon().0
    }

    #[inline(always)]
    pub fn suffix(&self) -> &str {
        self.split_at_colon().1
    }
}

/// An error indicating that a string is not a valid calendar address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidCalAddressError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("expected mailto: scheme")]
    NotMailto,
}

/// A calendar user address (RFC 8984 §4.4.5).
///
/// This must be a `mailto:` URI.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = CalAddress::str_is_cal_address, error = InvalidCalAddressError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub CalAddressBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct CalAddress(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<CalAddress> {
    type Error = TypeErrorOr<StringError<InvalidCalAddressError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        CalAddress::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl CalAddress {
    fn str_is_cal_address(s: &str) -> Result<(), InvalidCalAddressError> {
        if s.is_empty() {
            return Err(InvalidCalAddressError::EmptyString);
        }
        if !s.starts_with("mailto:") {
            return Err(InvalidCalAddressError::NotMailto);
        }
        Ok(())
    }

    /// Returns the email address portion (after `mailto:`).
    #[inline(always)]
    pub fn email(&self) -> &str {
        self.as_str()
            .strip_prefix("mailto:")
            .expect("a CalAddress must start with mailto:")
    }
}

/// A string slice where every character is ASCII alphanumeric.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = AlphaNumeric::str_is_alphanumeric, error = InvalidAlphaNumericError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct AlphaNumeric(str);

impl AlphaNumeric {
    pub fn str_is_alphanumeric(s: &str) -> Result<(), InvalidAlphaNumericError> {
        match s.char_indices().find(|(_, c)| !c.is_ascii_alphanumeric()) {
            Some((index, c)) => Err(InvalidAlphaNumericError { c, index }),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Error)]
#[error("encountered the non-alphanumeric character {c} at index {index}")]
pub struct InvalidAlphaNumericError {
    c: char,
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde_json")]
    #[test]
    fn id_from_serde_json() {
        use serde_json::Value;

        let parse = |s| Box::<Id>::try_from_json(serde_json::from_str::<'_, Value>(s).unwrap());

        let too_long = {
            let mut buf = String::from('"');
            buf.extend(['a'; 256]);
            buf.push('"');
            buf
        };

        assert!(parse("\"\"").is_err());
        assert!(parse(too_long.as_str()).is_err());

        assert!(parse("\"Event\"").is_ok());
        assert!(parse("\"Group\"").is_ok());
        assert!(parse("\"3213521675673128567312\"").is_ok());

        assert!(parse("\"λ\"").is_err());
        assert!(parse("true").is_err());
        assert!(parse("17").is_err());
    }

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

    #[test]
    fn vendor_str_predicate() {
        let p = VendorStr::is_vendor_str;

        assert_eq!(p(""), Err(InvalidVendorStrError::EmptyString));
        assert_eq!(p(":"), Err(InvalidVendorStrError::EmptyPrefix));
        assert_eq!(p("a:"), Err(InvalidVendorStrError::EmptySuffix));
        assert_eq!(p("a"), Err(InvalidVendorStrError::MissingColon));

        assert!(p("a:b").is_ok());
        assert!(p("foo:bar").is_ok());
        assert!(p("example.com:foo:bar:baz").is_ok());
    }
}
