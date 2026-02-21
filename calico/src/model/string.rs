//! String newtypes in the iCalendar data model.

use dizzy::DstNewtype;

pub use rfc5545_types::string::{CaselessStr, InvalidCharError, InvalidNameError, InvalidTextError};
pub use rfc5545_types::string::{Name, NameKind, Text, TextBuf};

// ============================================================================
// TzId
// ============================================================================

/// A timezone identifier.
#[derive(PartialEq, Eq, Hash, DstNewtype)]
#[dizzy(invariant = dizzy::trivial, error = std::convert::Infallible)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct TzId(str);

impl TzId {
    /// Creates a boxed `TzId` from a `TextBuf`.
    pub fn from_text_buf(value: TextBuf) -> Box<Self> {
        let s = value.into_string();
        // unwrap is infallible: trivial invariant never fails
        Self::new(&s).unwrap().into()
    }
}

// ============================================================================
// Uri
// ============================================================================

/// A URI (RFC 3986 §3.3.13).
///
/// This is a permissive wrapper around `str` used for URI values in iCalendar content lines.
/// No scheme validation is performed — use [`calendar_types::string::Uri`] for stricter parsing.
#[derive(PartialEq, Eq, Hash, DstNewtype)]
#[dizzy(invariant = dizzy::trivial, error = std::convert::Infallible)]
#[dizzy(constructor = pub(crate) new)]
#[dizzy(unsafe_constructor = pub(crate) const from_str_unchecked)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct Uri(str);

// ============================================================================
// Uid
// ============================================================================

/// A unique identifier (RFC 5545 §3.8.4.7).
#[derive(PartialEq, Eq, Hash, DstNewtype)]
#[dizzy(invariant = dizzy::trivial, error = std::convert::Infallible)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct Uid(str);

impl Uid {
    /// Creates a boxed `Uid` from a `TextBuf`.
    pub fn from_text_buf(value: TextBuf) -> Box<Self> {
        let s = value.into_string();
        // unwrap is infallible: trivial invariant never fails
        Self::new(&s).unwrap().into()
    }
}

// ============================================================================
// ParamValue
// ============================================================================

/// A parameter value string (RFC 5545 §3.1). In practice this is a value that cannot contain
/// ASCII control characters (other than HTAB), double quotes (U+0022), or newlines (U+000A).
#[derive(PartialEq, Eq, Hash, DstNewtype)]
#[dizzy(invariant = ParamValue::str_is_param_value, error = InvalidCharError)]
#[dizzy(constructor = pub new)]
#[dizzy(unsafe_constructor = pub(crate) from_str_unchecked)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[repr(transparent)]
pub struct ParamValue(str);

impl ParamValue {
    /// Returns `true` iff the given `char` is valid in a [`ParamValue`].
    #[inline(always)]
    pub const fn char_is_valid(c: char) -> bool {
        !((c.is_ascii_control() && c != '\t') || c == '"')
    }

    fn str_is_param_value(s: &str) -> Result<(), InvalidCharError> {
        match s.char_indices().find(|(_, c)| !Self::char_is_valid(*c)) {
            Some(c) => Err(InvalidCharError::from_char_index(c)),
            None => Ok(()),
        }
    }
}

impl TryFrom<String> for Box<ParamValue> {
    type Error = InvalidCharError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(ParamValue::new(&value)?.into())
    }
}

// ============================================================================
// text! macro
// ============================================================================

/// Constructs a [`Text`] value from a string literal.
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
    fn text_from_workspace() {
        let t = Text::new("America/New_York").unwrap();
        let b: Box<Text> = t.into();
        let tb = TextBuf::from(&*b);
        assert_eq!(tb.as_str(), "America/New_York");
    }

    #[test]
    fn tz_id_from_text_buf() {
        let tb = TextBuf::from(Text::new("America/New_York").unwrap());
        let tz = TzId::from_text_buf(tb);
        assert_eq!(tz.as_str(), "America/New_York");
    }

    #[test]
    fn param_value_validation() {
        assert!(ParamValue::new("hello world").is_ok());
        assert!(ParamValue::new("has\ttab").is_ok());
        assert!(ParamValue::new("has\"quote").is_err());
        assert!(ParamValue::new("has\x00null").is_err());
    }
}
