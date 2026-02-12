//! String data model types for RFC 5545.

use dizzy::DstNewtype;

/// An error indicating that a string is not valid `paramtext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

