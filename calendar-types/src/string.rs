//! String data model types.

use dizzy::DstNewtype;
use thiserror::Error;

/// An error indicating that a string is not a valid UID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidUidError {
    #[error("expected at least one character")]
    EmptyString,
}

/// A globally unique identifier.
///
/// This type is used by both JSCalendar (RFC 8984 §4.1.1) and iCalendar (RFC 5545 §3.8.4.7).
/// The value is an arbitrary non-empty string with no particular format required.
/// Uniqueness is a semantic property and is not enforced by this type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = Uid::str_is_uid, error = InvalidUidError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub UidBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct Uid(str);

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Uid {
    fn str_is_uid(s: &str) -> Result<(), InvalidUidError> {
        if s.is_empty() {
            return Err(InvalidUidError::EmptyString);
        }
        Ok(())
    }
}

/// An error indicating that a string is not a valid URI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidUriError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("missing colon after scheme")]
    MissingColon,
    #[error("scheme must start with a letter")]
    SchemeStartsWithNonLetter,
    #[error("invalid character in scheme: {c}")]
    InvalidSchemeChar { index: usize, c: char },
}

/// A URI string (RFC 3986).
///
/// # Invariants
/// 1. The underlying string is not empty.
/// 2. The string contains a colon separating the scheme from the rest.
/// 3. The scheme starts with a letter and contains only letters, digits, `+`, `-`, or `.`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, DstNewtype)]
#[dizzy(invariant = Uri::str_is_uri, error = InvalidUriError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub UriBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct Uri(str);

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Uri {
    fn str_is_uri(s: &str) -> Result<(), InvalidUriError> {
        let (scheme, _rest) = s.split_once(':').ok_or(if s.is_empty() {
            InvalidUriError::EmptyString
        } else {
            InvalidUriError::MissingColon
        })?;

        let mut chars = scheme.chars().enumerate();

        match chars.next() {
            None => return Err(InvalidUriError::MissingColon),
            Some((_, c)) if !c.is_ascii_alphabetic() => {
                return Err(InvalidUriError::SchemeStartsWithNonLetter);
            }
            Some(_) => {}
        }

        for (index, c) in chars {
            if !c.is_ascii_alphanumeric() && c != '+' && c != '-' && c != '.' {
                return Err(InvalidUriError::InvalidSchemeChar { index, c });
            }
        }

        Ok(())
    }

    /// Returns the scheme portion of the URI (before the first colon).
    #[inline(always)]
    pub fn scheme(&self) -> &str {
        self.as_str()
            .split_once(':')
            .expect("a Uri must contain a colon")
            .0
    }
}