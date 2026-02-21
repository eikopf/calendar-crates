//! String data model types for RFC 5545.

use std::num::NonZero;

use dizzy::DstNewtype;

/// An error indicating that a string is not valid `paramtext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid character {c:?} at index {index}")]
pub struct InvalidParamTextError {
    /// The index of the first invalid character.
    pub index: usize,
    /// The invalid character.
    pub c: char,
}

/// A `paramtext` value as defined by RFC 5545 §3.1.
///
/// ```text
/// paramtext = *SAFE-CHAR
/// ```
///
/// This is the unquoted form of a property parameter value. The quoted form (`QSAFE-CHAR`) allows
/// additional characters like `:`, `;`, and `,`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = ParamText::str_is_paramtext, error = InvalidParamTextError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub ParamTextBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct ParamText(str);

impl ParamText {
    fn str_is_paramtext(s: &str) -> Result<(), InvalidParamTextError> {
        for (index, c) in s.chars().enumerate() {
            if !char_is_safe_char(c) {
                return Err(InvalidParamTextError { index, c });
            }
        }
        Ok(())
    }
}

/// Returns `true` iff `c` is a `SAFE-CHAR` as defined by RFC 5545 §3.1.
///
/// ```text
/// SAFE-CHAR = WSP / %x21 / %x23-2B / %x2D-39 / %x3C-7E / NON-US-ASCII
/// ```
///
/// NB: RFC 5545 doesn't define the `WSP` rule in its grammar, as it is defined by RFC 5234 to be
/// either the literal space (U+0020) or the horizontal tab (U+0009).
const fn char_is_safe_char(c: char) -> bool {
    match c {
        '\t' | ' ' | '!' | '#'..='+' | '-'..='9' | '<'..='~' => true,
        _ => !c.is_ascii(),
    }
}

// ============================================================================
// Text / TextBuf
// ============================================================================

/// An error indicating that a string is not a valid TEXT value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid character {c:?} at byte index {index}")]
pub struct InvalidTextError {
    /// The byte index of the first invalid character.
    pub index: usize,
    /// The invalid character.
    pub c: char,
}

/// A textual value (RFC 5545 §3.3.11).
///
/// This is the subset of `str` values that are permissible TEXT property values in iCalendar:
/// all strings that do not contain ASCII control characters other than HTAB (U+0009) and
/// LF (U+000A).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = Text::str_is_text, error = InvalidTextError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub TextBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct Text(str);

impl Text {
    /// Returns `true` iff `c` is valid in a TEXT value.
    pub const fn char_is_valid(c: char) -> bool {
        char_is_text(c)
    }

    pub(crate) fn str_is_text(s: &str) -> Result<(), InvalidTextError> {
        for (index, c) in s.char_indices() {
            if !char_is_text(c) {
                return Err(InvalidTextError { index, c });
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TextBuf {
    /// Consumes the `TextBuf` and returns the inner `String`.
    pub fn into_string(self) -> String {
        self.__dizzy_owned_inner
    }

    /// Creates a `TextBuf` from a `String`, validating the text invariant.
    pub fn from_string(s: String) -> Result<Self, InvalidTextError> {
        Text::str_is_text(&s)?;
        // SAFETY: validated above; the inner field is correct
        Ok(Self {
            __data: std::marker::PhantomData,
            __dizzy_owned_inner: s,
        })
    }

    /// Creates a `TextBuf` from a `String` without checking the text invariant.
    ///
    /// # Safety
    /// The string must not contain ASCII control characters other than HTAB or LF.
    pub unsafe fn from_string_unchecked(s: String) -> Self {
        debug_assert!(Text::str_is_text(&s).is_ok());
        Self {
            __data: std::marker::PhantomData,
            __dizzy_owned_inner: s,
        }
    }
}

impl std::fmt::Display for TextBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returns `true` iff `c` is valid in a TEXT value (RFC 5545 §3.3.11).
///
/// Allowed: all characters except ASCII control characters, with the exceptions of
/// HTAB (U+0009) and LF (U+000A) which are permitted.
const fn char_is_text(c: char) -> bool {
    if c.is_ascii_control() {
        // Allow HTAB and LF
        c == '\t' || c == '\n'
    } else {
        true
    }
}

// ============================================================================
// CaselessStr
// ============================================================================

/// A case-insensitive `str` with `repr(transparent)` layout.
///
/// Equality and hashing are performed case-insensitively (ASCII case folding).
/// This is used for iCalendar property and parameter names (RFC 5545 §3.1).
#[derive(Debug, Eq)]
#[repr(transparent)]
pub struct CaselessStr(str);

impl CaselessStr {
    /// Wraps a `&str` as a `&CaselessStr`.
    #[inline(always)]
    pub fn new(s: &str) -> &Self {
        // SAFETY: CaselessStr is repr(transparent) over str
        unsafe { &*(s as *const str as *const CaselessStr) }
    }

    /// Returns the underlying string slice.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts a `Box<str>` into a `Box<CaselessStr>`.
    #[inline(always)]
    pub fn from_box_str(value: Box<str>) -> Box<CaselessStr> {
        // SAFETY: CaselessStr is repr(transparent) over str
        unsafe { Box::from_raw(Box::into_raw(value) as *mut CaselessStr) }
    }

    /// Converts a `Box<CaselessStr>` into a `Box<str>`.
    #[inline(always)]
    pub fn into_box_str(self: Box<Self>) -> Box<str> {
        // SAFETY: CaselessStr is repr(transparent) over str
        unsafe { Box::from_raw(Box::into_raw(self) as *mut str) }
    }
}

impl Clone for Box<CaselessStr> {
    fn clone(&self) -> Self {
        CaselessStr::from_box_str(Box::<str>::from(&self.0))
    }
}

impl<'a> From<&'a str> for &'a CaselessStr {
    fn from(value: &'a str) -> Self {
        CaselessStr::new(value)
    }
}

impl From<&str> for Box<CaselessStr> {
    fn from(value: &str) -> Self {
        CaselessStr::from_box_str(Box::<str>::from(value))
    }
}

impl From<Box<str>> for Box<CaselessStr> {
    fn from(value: Box<str>) -> Self {
        CaselessStr::from_box_str(value)
    }
}

impl From<String> for Box<CaselessStr> {
    fn from(value: String) -> Self {
        CaselessStr::from_box_str(value.into_boxed_str())
    }
}

impl PartialEq for CaselessStr {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialEq<str> for CaselessStr {
    fn eq(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl std::hash::Hash for CaselessStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for byte in self.0.as_bytes() {
            byte.to_ascii_lowercase().hash(state);
        }
    }
}

impl std::fmt::Display for CaselessStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ============================================================================
// Name / NameKind
// ============================================================================

/// An error indicating that a string is not a valid iCalendar name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidNameError {
    /// The string was empty.
    #[error("name must not be empty")]
    EmptyString,
    /// A character that is not alphanumeric or hyphen was found.
    #[error("invalid character {c:?} at byte index {index}")]
    InvalidChar {
        /// The byte index of the invalid character.
        index: usize,
        /// The invalid character.
        c: char,
    },
}

/// A name in a content line (RFC 5545 §3.1), or any value satisfying the `iana-token` grammar.
///
/// The values of this type are non-empty strings whose characters are ASCII alphanumeric or
/// hyphen (U+002D).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = Name::str_is_name, error = InvalidNameError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct Name(str);

impl Name {
    fn str_is_name(s: &str) -> Result<(), InvalidNameError> {
        if s.is_empty() {
            return Err(InvalidNameError::EmptyString);
        }
        for (index, c) in s.char_indices() {
            if !c.is_ascii_alphanumeric() && c != '-' {
                return Err(InvalidNameError::InvalidChar { index, c });
            }
        }
        Ok(())
    }

    /// Returns the length of this name.
    pub fn len(&self) -> NonZero<usize> {
        // SAFETY: Name invariant guarantees non-empty
        unsafe { NonZero::new_unchecked(self.0.len()) }
    }

    /// Returns the kind of this name (IANA or X-).
    pub fn kind(&self) -> NameKind {
        NameKind::of(self.as_str()).expect("Name is non-empty")
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The kind of an iCalendar name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameKind {
    /// An IANA-registered name.
    Iana,
    /// An experimental (X-) name.
    X,
}

impl NameKind {
    /// Determines the kind of the given string, or `None` if it is empty.
    pub fn of(s: &str) -> Option<Self> {
        match s.as_bytes().first_chunk::<2>() {
            Some(b"x-" | b"X-") => Some(Self::X),
            _ if !s.is_empty() => Some(Self::Iana),
            _ => None,
        }
    }
}

/// An error indicating that a string is not a valid iCalendar `paramtext` or TEXT value,
/// because it contains an invalid character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCharError {
    /// The byte index of the invalid character.
    pub byte_index: usize,
    /// The invalid character.
    pub invalid_char: char,
}

impl InvalidCharError {
    /// Constructs an `InvalidCharError` from a `(byte_index, char)` pair
    /// as returned by `str::char_indices`.
    pub const fn from_char_index((byte_index, invalid_char): (usize, char)) -> Self {
        Self {
            byte_index,
            invalid_char,
        }
    }
}
