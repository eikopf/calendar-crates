//! String data model types.

use std::{borrow::Cow, fmt::Debug, num::NonZero};

pub use calendar_types::string::{
    InvalidUidError, InvalidUriError, LanguageTag, LanguageTagParseError, Uid, UidBuf, Uri, UriBuf,
};
use dizzy::DstNewtype;
use rfc5545_types::string::ParamText;
use thiserror::Error;

use crate::json::{DestructibleJsonValue, TryFromJson, TypeErrorOr};

impl<V: DestructibleJsonValue> TryFromJson<V> for LanguageTag {
    type Error = TypeErrorOr<StringError<LanguageTagParseError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;
        LanguageTag::parse(input.as_ref())
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

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

/// A string validation error, pairing the rejected input with the underlying error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringError<E> {
    pub(crate) input: Box<str>,
    pub(crate) error: E,
}

impl<E: std::fmt::Display> std::fmt::Display for StringError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid value {:?}: {}", self.input, self.error)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for StringError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
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

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

    /// Returns the underlying bytes of this `Id`.
    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: two slices have the same layout iff their parameter types have the same layout.
        // IdChar has repr(u8), so this is satisfied, and moreover every value of IdChar is valid
        // as a byte
        unsafe { std::mem::transmute::<&[IdChar], &[u8]>(self.as_slice()) }
    }

    /// Returns this `Id` as a string slice.
    #[inline(always)]
    pub const fn as_str(&self) -> &str {
        let bytes: &[u8] = self.as_bytes();
        debug_assert!(bytes.is_ascii());

        // SAFETY: the bytes of `self` must have the values of the variants of IdChar, which are
        // all valid ASCII bytes
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    /// Returns the length of this `Id` as a non-zero `u8`.
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
    /// Converts this `IdChar` into a `char`.
    #[inline(always)]
    pub const fn into_char(self) -> char {
        (self as u8) as char
    }

    /// Returns `true` if `value` is a valid [`IdChar`] (ASCII alphanumeric, hyphen, or underscore).
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

impl std::fmt::Display for CustomTimeZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum InvalidImplicitJsonPointerError {
    /// A tilde (`~`) occurred without being immediately followed by `0` or `1` at this index.
    #[error("a tilde ocurred without being immediately followed by `0` or `1` at index {index}")]
    BareTilde { index: usize },
    #[error("a forward slash occurred at the start of the pointer")]
    Explicit,
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

impl std::fmt::Display for ImplicitJsonPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl ImplicitJsonPointer {
    fn str_is_implicit_json_pointer(s: &str) -> Result<(), InvalidImplicitJsonPointerError> {
        let mut iter = s.char_indices().peekable();

        // if the string starts with a forward slash, it's invalid
        if let Some(&(_, '/')) = iter.peek() {
            return Err(InvalidImplicitJsonPointerError::Explicit);
        }

        while let Some((index, c)) = iter.next() {
            if c == '~' && iter.peek().is_none_or(|(_, c)| c != &'0' && c != &'1') {
                return Err(InvalidImplicitJsonPointerError::BareTilde { index });
            }
        }

        Ok(())
    }

    /// Returns an iterator over the segments of this pointer, split on `/` and unescaping `~0`/`~1`.
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

impl std::fmt::Display for VendorStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

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

    /// Returns the length of this `VendorStr` as a non-zero `usize`.
    #[inline(always)]
    pub const fn len(&self) -> NonZero<usize> {
        debug_assert!(!self.as_str().is_empty());

        // SAFETY: it is an invariant of VendorStr that the underlying string slice is not empty
        unsafe { NonZero::new_unchecked(self.as_str().len()) }
    }

    /// Splits this string at the first colon, returning `(vendor_domain, suffix)`.
    #[inline(always)]
    pub fn split_at_colon(&self) -> (&str, &str) {
        self.as_str()
            .split_once(':')
            .expect("a VendorStr must contain at least one colon")
    }

    /// Returns the vendor domain portion (before the first colon).
    #[inline(always)]
    pub fn vendor_domain(&self) -> &str {
        self.split_at_colon().0
    }

    /// Returns the suffix portion (after the first colon).
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

impl std::fmt::Display for CalAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

/// An error indicating that a string is not a valid email address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidEmailAddrError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("expected exactly one '@' character")]
    InvalidAtSign,
    #[error("empty local part before '@'")]
    EmptyLocalPart,
    #[error("empty domain part after '@'")]
    EmptyDomainPart,
}

/// An email address (RFC 5322 §3.4.1).
///
/// This performs minimal validation: the string must be non-empty,
/// contain exactly one `@` character, and have non-empty local and domain parts.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = EmailAddr::str_is_email_addr, error = InvalidEmailAddrError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct EmailAddr(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<EmailAddr> {
    type Error = TypeErrorOr<StringError<InvalidEmailAddrError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        EmailAddr::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl std::fmt::Display for EmailAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl EmailAddr {
    fn str_is_email_addr(s: &str) -> Result<(), InvalidEmailAddrError> {
        if s.is_empty() {
            return Err(InvalidEmailAddrError::EmptyString);
        }

        let (local, domain) = s
            .split_once('@')
            .ok_or(InvalidEmailAddrError::InvalidAtSign)?;

        if domain.contains('@') {
            return Err(InvalidEmailAddrError::InvalidAtSign);
        }
        if local.is_empty() {
            return Err(InvalidEmailAddrError::EmptyLocalPart);
        }
        if domain.is_empty() {
            return Err(InvalidEmailAddrError::EmptyDomainPart);
        }

        Ok(())
    }

    /// Returns the local part (before `@`).
    #[inline(always)]
    pub fn local_part(&self) -> &str {
        self.as_str()
            .split_once('@')
            .expect("an EmailAddr must contain @")
            .0
    }

    /// Returns the domain part (after `@`).
    #[inline(always)]
    pub fn domain(&self) -> &str {
        self.as_str()
            .split_once('@')
            .expect("an EmailAddr must contain @")
            .1
    }
}

/// An error indicating that a string is not a valid geo URI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidGeoUriError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("expected 'geo:' scheme")]
    NotGeoScheme,
    #[error("missing latitude value")]
    MissingLatitude,
    #[error("missing longitude value")]
    MissingLongitude,
    #[error("invalid latitude value")]
    InvalidLatitude,
    #[error("invalid longitude value")]
    InvalidLongitude,
}

/// A geographic URI (RFC 5870).
///
/// Format: `geo:latitude,longitude[,altitude][;parameters]`
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = GeoUri::str_is_geo_uri, error = InvalidGeoUriError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct GeoUri(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<GeoUri> {
    type Error = TypeErrorOr<StringError<InvalidGeoUriError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        GeoUri::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl std::fmt::Display for GeoUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl GeoUri {
    fn str_is_geo_uri(s: &str) -> Result<(), InvalidGeoUriError> {
        if s.is_empty() {
            return Err(InvalidGeoUriError::EmptyString);
        }

        let body = s
            .strip_prefix("geo:")
            .ok_or(InvalidGeoUriError::NotGeoScheme)?;

        // Split off parameters (after `;`)
        let coords = body.split(';').next().unwrap_or(body);

        let mut parts = coords.split(',');

        let lat_str = parts.next().ok_or(InvalidGeoUriError::MissingLatitude)?;
        if lat_str.is_empty() {
            return Err(InvalidGeoUriError::MissingLatitude);
        }
        lat_str
            .parse::<f64>()
            .map_err(|_| InvalidGeoUriError::InvalidLatitude)?;

        let lon_str = parts.next().ok_or(InvalidGeoUriError::MissingLongitude)?;
        if lon_str.is_empty() {
            return Err(InvalidGeoUriError::MissingLongitude);
        }
        lon_str
            .parse::<f64>()
            .map_err(|_| InvalidGeoUriError::InvalidLongitude)?;

        // Altitude is optional, we don't validate it strictly
        Ok(())
    }
}

/// An error indicating that a string is not a valid Content-ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidContentIdError {
    #[error("expected at least one character")]
    EmptyString,
}

/// A Content-ID reference (RFC 2392 §2).
///
/// This is a minimal validation: the string must be non-empty.
/// Full Content-ID validation is complex; we defer to usage context.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = ContentId::str_is_content_id, error = InvalidContentIdError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct ContentId(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<ContentId> {
    type Error = TypeErrorOr<StringError<InvalidContentIdError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        ContentId::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ContentId {
    fn str_is_content_id(s: &str) -> Result<(), InvalidContentIdError> {
        if s.is_empty() {
            return Err(InvalidContentIdError::EmptyString);
        }
        Ok(())
    }
}

/// An error indicating that a string is not a valid media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidMediaTypeError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("expected '/' separator between type and subtype")]
    MissingSlash,
    #[error("empty type part before '/'")]
    EmptyType,
    #[error("empty subtype part after '/'")]
    EmptySubtype,
}

/// A MIME media type (RFC 6838).
///
/// Format: `type/subtype[;parameters]`
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = MediaType::str_is_media_type, error = InvalidMediaTypeError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct MediaType(str);

impl<V: DestructibleJsonValue> TryFromJson<V> for Box<MediaType> {
    type Error = TypeErrorOr<StringError<InvalidMediaTypeError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let input = value.try_into_string()?;

        MediaType::new(input.as_ref())
            .map(Into::into)
            .map_err(|error| StringError {
                input: String::from(input.as_ref()).into(),
                error,
            })
            .map_err(TypeErrorOr::Other)
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl MediaType {
    fn str_is_media_type(s: &str) -> Result<(), InvalidMediaTypeError> {
        if s.is_empty() {
            return Err(InvalidMediaTypeError::EmptyString);
        }

        // Split off parameters (after `;`)
        let type_subtype = s.split(';').next().unwrap_or(s);

        let (type_part, subtype) = type_subtype
            .split_once('/')
            .ok_or(InvalidMediaTypeError::MissingSlash)?;

        if type_part.is_empty() {
            return Err(InvalidMediaTypeError::EmptyType);
        }
        if subtype.is_empty() {
            return Err(InvalidMediaTypeError::EmptySubtype);
        }

        Ok(())
    }

    /// Returns the type part (before `/`).
    #[inline(always)]
    pub fn type_part(&self) -> &str {
        self.as_str()
            .split(';')
            .next()
            .unwrap()
            .split_once('/')
            .expect("a MediaType must contain /")
            .0
    }

    /// Returns the subtype part (after `/`, before parameters).
    #[inline(always)]
    pub fn subtype(&self) -> &str {
        self.as_str()
            .split(';')
            .next()
            .unwrap()
            .split_once('/')
            .expect("a MediaType must contain /")
            .1
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

impl std::fmt::Display for AlphaNumeric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AlphaNumeric {
    /// Returns `Ok` if every character in `s` is ASCII alphanumeric.
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

// ============================================================================
// IntoJson impls for string newtypes
// ============================================================================

use crate::json::{ConstructibleJsonValue, IntoJson};

impl<V: ConstructibleJsonValue> IntoJson<V> for LanguageTag {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<Id> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<Uid> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<Uri> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<VendorStr> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<CalAddress> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<EmailAddr> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<GeoUri> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<ContentId> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<MediaType> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<AlphaNumeric> {
    fn into_json(self) -> V {
        V::string(self.as_str().to_owned())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<CustomTimeZoneId> {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Box<ImplicitJsonPointer> {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
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
