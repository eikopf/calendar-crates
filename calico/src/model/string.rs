//! String newtypes in the iCalendar data model.

use std::num::NonZero;

use mitsein::str1::Str1;

/// An empty type that implements [`AsRef<str>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeverStr {}

impl AsRef<str> for NeverStr {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        unreachable!()
    }
}

pub use rfc5545_types::string::{CaselessStr, InvalidCharError};

/// Helper trait for converting a reference to an unsized type into a boxed value of the type.
trait CloneUnsized {
    fn clone_to_boxed(&self) -> Box<Self>;
}

impl CloneUnsized for str {
    fn clone_to_boxed(&self) -> Box<Self> {
        Box::<str>::from(self)
    }
}

impl CloneUnsized for Text {
    fn clone_to_boxed(&self) -> Box<Self> {
        let s = Box::<str>::from(&self.0);
        // SAFETY: `s` is definitely a Text value because it was cloned from a Text value
        unsafe { Text::from_boxed_unchecked(s) }
    }
}

impl CloneUnsized for Str1 {
    fn clone_to_boxed(&self) -> Box<Self> {
        self.to_string1().into_boxed_str1()
    }
}

macro_rules! unsized_newtype {
    ($(#[$m:meta])* $v:vis struct $name:ident ($t:ty)) => {
        $(#[$m])*
        #[repr(transparent)]
        $v struct $name ($t);

        impl $name {
            /// Converts the given `value` into a boxed instance of this type without checking any
            /// preconditions of the type. This will never cause memory unsafety, but may produce
            /// an invalid instance of the type.
            ///
            /// # Safety
            /// The `value` parameter must correspond to a valid instance of this type.
            #[inline(always)]
            #[allow(dead_code)]
            pub(crate) unsafe fn from_boxed_unchecked(value: Box<$t>) -> Box<$name> {
                // SAFETY: `value` is a valid Box, so has been allocated by the global allocator
                // and this provenance is transferred to the raw pointer. moreover `Self` is a
                // transparent newtype of `$t`, so the pointer cast is sound
                unsafe { Box::from_raw(Box::into_raw(value) as *mut $name) }
            }

            /// Converts the given `value` into a reference to an instance of this type without
            /// checking any preconditions of the type. This will never cause memory unsafety, but
            /// may produce an invalid instance of the type.
            ///
            /// # Safety
            /// The `value` parameter must correspond to a valid instance of this type.
            #[inline(always)]
            #[allow(dead_code)]
            pub(crate) const unsafe fn from_ref_unchecked(value: &$t) -> &$name {
                // SAFETY: `Self` is a transparent newtype of `$t`, so this transmute is sound
                unsafe { std::mem::transmute::<&$t, &$name>(value) }
            }
        }

        impl std::convert::AsRef<$t> for $name {
            fn as_ref(&self) -> &$t {
                &self.0
            }
        }

        impl Clone for Box<$name> {
            fn clone(&self) -> Self {
                let s: Box<$t> = self.0.clone_to_boxed();
                // SAFETY: `self` is already a valid instance of $name
                unsafe { $name::from_boxed_unchecked(s) }
            }
        }

        impl<'a> From<&'a $name> for Box<$name> {
            fn from(value: &'a $name) -> Box<$name> {
                let s: Box<$t> = value.0.clone_to_boxed();
                // SAFETY: `self` is already a valid instance of $name
                unsafe { $name::from_boxed_unchecked(s) }
            }
        }
    };
}

unsized_newtype! {
    /// A timezone identifier.
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct TzId(Text)
}

impl TzId {
    pub fn from_boxed_text(value: Box<Text>) -> Box<Self> {
        // SAFETY: all Text values are valid TzId values
        unsafe { Self::from_boxed_unchecked(value) }
    }
}

unsized_newtype! {
    /// A URI (RFC 3986 §3.3.13).
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Uri(str)
}

unsized_newtype! {
    /// A unique identifier (RFC 5545 §3.8.4.7).
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Uid(Text)
}

impl Uid {
    pub fn from_boxed_text(value: Box<Text>) -> Box<Self> {
        // SAFETY: all Text values are valid Uid values
        unsafe { Self::from_boxed_unchecked(value) }
    }
}

unsized_newtype! {
    /// A name in a content line (RFC 5545 §3.1), or any value satisfying the `iana-token` grammar
    /// rule.
    ///
    /// The values of this type are non-empty strings whose characters may be the alphanumeric
    /// ASCII characters or the hyphen (U+002D) character. This conveys all the usual properties of
    /// an ASCII string, but also guarantees a non-zero length that corresponds exactly to the
    /// number of characters in the string.
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Name(Str1)
}

impl Name {
    /// Returns `true` iff the given `char` is valid in a [`Name`].
    pub const fn char_is_valid(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-'
    }

    pub const fn len(&self) -> NonZero<usize> {
        self.0.len()
    }

    pub fn kind(&self) -> NameKind {
        NameKind::of(self).expect("self is non-empty")
    }

    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn new(value: &str) -> Result<&Self, InvalidNameError> {
        let value = Str1::try_from_str(value).map_err(|_| InvalidNameError::EmptyStr)?;

        match value.char_indices().find(|(_, c)| !Name::char_is_valid(*c)) {
            Some(c) => Err(InvalidCharError::from_char_index(c).into()),
            None => Ok({
                // SAFETY: in this branch we found no invalid characters
                unsafe { Self::from_ref_unchecked(value) }
            }),
        }
    }
}

impl std::convert::AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// The kind of a [`Name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameKind {
    Iana,
    X,
}

impl NameKind {
    /// Returns the kind of the given string, or [`None`] if it's the empty string.
    pub fn of(s: impl AsRef<str>) -> Option<Self> {
        let str = s.as_ref();

        match str.as_bytes().first_chunk::<2>() {
            Some(b"x-" | b"X-") => Some(Self::X),
            _ if !str.is_empty() => Some(Self::Iana),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidNameError {
    InvalidChar(InvalidCharError),
    EmptyStr,
}

impl From<InvalidCharError> for InvalidNameError {
    fn from(v: InvalidCharError) -> Self {
        Self::InvalidChar(v)
    }
}

unsized_newtype! {
    /// A parameter value string (RFC 5545 §3.1). In practice this is a [`Text`] value that cannot
    /// contain double quotes (U+0022) and newlines (U+000A).
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct ParamValue(Text)
}

impl ParamValue {
    /// Returns `true` iff the given `char` is valid in a [`ParamValue`]. If this method returns
    /// `true`, then [`Text::char_is_valid`] is guaranteed to return `true` for the same input.
    #[inline(always)]
    pub const fn char_is_valid(c: char) -> bool {
        !((c.is_ascii_control() && c != '\t') || c == '"')
    }

    pub fn new(value: &str) -> Result<&Self, InvalidCharError> {
        match value.char_indices().find(|(_, c)| !Self::char_is_valid(*c)) {
            Some(c) => Err(InvalidCharError::from_char_index(c)),
            None => Ok({
                // SAFETY: every character satisfies Self::char_is_valid, which entails that every
                // character also satisfies Text::char_is_valid. hence this is a valid Text value
                let text = unsafe { Text::from_ref_unchecked(value) };
                // SAFETY: every character satisfies Self::char_is_valid
                unsafe { Self::from_ref_unchecked(text) }
            }),
        }
    }
}

impl TryFrom<String> for Box<ParamValue> {
    type Error = InvalidCharError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value
            .char_indices()
            .find(|(_, c)| !ParamValue::char_is_valid(*c))
        {
            Some(c) => Err(InvalidCharError::from_char_index(c)),
            None => Ok({
                // SAFETY: every character satisfies Self::char_is_valid, which entails that every
                // character also satisfies Text::char_is_valid. hence this is a valid Text value
                let text = unsafe { Text::from_boxed_unchecked(value.into_boxed_str()) };
                // SAFETY: every character satisfies Self::char_is_valid
                unsafe { ParamValue::from_boxed_unchecked(text) }
            }),
        }
    }
}

unsized_newtype! {
    /// A textual value (RFC 5545 §3.3.11).
    ///
    /// This type represents the subset of `str` values that are permissible TEXT property values
    /// in iCalendar, which are all the strings that do not contain ASCII control characters other
    /// than HTAB (U+0009) and LF/NL (U+000A) [^unicode-ranges].
    ///
    /// [^unicode-ranges]: To be explicit, this is any character in the range of U+0000 through
    /// U+0008, or U+000B through U+001F, or the single character U+007F.
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Text(str)
}

impl Text {
    /// Returns `true` iff the given `char` is valid in a [`Text`] value.
    pub const fn char_is_valid(c: char) -> bool {
        !(c.is_ascii_control() || c == '\t' || c == '\n')
    }

    pub fn new(value: &str) -> Result<&Self, InvalidCharError> {
        match value.char_indices().find(|(_, c)| !Text::char_is_valid(*c)) {
            Some(c) => Err(InvalidCharError::from_char_index(c)),
            None => Ok({
                // SAFETY: since every character is valid, this is a valid Text value
                unsafe { Self::from_ref_unchecked(value) }
            }),
        }
    }
}

impl std::ops::Deref for Text {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for Box<Text> {
    type Error = InvalidCharError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Text::new(value).map(Into::into)
    }
}

impl TryFrom<&str> for TextBuf {
    type Error = InvalidCharError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Text::new(value).map(Into::into)
    }
}

impl<'a> TryFrom<&'a str> for &'a Text {
    type Error = InvalidCharError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Text::new(value)
    }
}

/// An owned, growable textual value (RFC 5545 §3.3.11). See [`Text`] for details about invariants
/// and usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBuf(String);

impl TextBuf {
    pub fn into_boxed_text(self) -> Box<Text> {
        // SAFETY: `self` definitely represents a valid Text value
        unsafe { Text::from_boxed_unchecked(self.0.into_boxed_str()) }
    }

    pub const fn as_text(&self) -> &Text {
        // SAFETY: `self` definitely represents a valid Text value
        unsafe { Text::from_ref_unchecked(self.0.as_str()) }
    }

    /// Converts the given `value` into a `TextBuf` without checking the preconditions of [`Text`].
    /// This will never cause memory unsafety, but may result in invalid instances of `TextBuf`.
    ///
    /// # Safety
    /// The `value` parameter must represent a valid [`Text`] value.
    pub const unsafe fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    pub fn new(value: String) -> Result<Self, InvalidCharError> {
        match value.char_indices().find(|(_, c)| !Text::char_is_valid(*c)) {
            Some(c) => Err(InvalidCharError::from_char_index(c)),
            None => Ok(Self(value)),
        }
    }
}

impl std::ops::Deref for TextBuf {
    type Target = Text;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `self` definitely represents a valid Text value
        unsafe { Text::from_ref_unchecked(self.0.as_str()) }
    }
}

impl std::convert::AsRef<Text> for TextBuf {
    fn as_ref(&self) -> &Text {
        self
    }
}

impl std::convert::AsRef<str> for TextBuf {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&Text> for TextBuf {
    fn from(value: &Text) -> Self {
        let buf = String::from(&value.0);
        Self(buf)
    }
}

impl From<TextBuf> for Box<str> {
    fn from(value: TextBuf) -> Self {
        value.0.into_boxed_str()
    }
}

#[macro_export]
macro_rules! text {
    ($lit:literal $(, $t:ty)?) => {
        {
            let value $(: $t)? = $lit .try_into().ok().expect("valid literal");
            value
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_macro() {
        let _: Box<Text> = text!("America/New_York");
        let _: &Text = text!("America/New_York");
        let _: TextBuf = text!("America/New_York");
        let _ = text!("America/New_York", Box<Text>);
    }
}
