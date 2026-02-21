//! Compound property value types.

use calendar_types::string::Uri;

/// A latitude-longitude pair of geographic coordinates (RFC 5545 §3.8.1.6).
///
/// Both latitude and longitude are stored as `f64`, which provides sufficient precision
/// for the six decimal places specified by RFC 5545 (accuracy to within one meter).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Geo {
    pub lat: f64,
    pub lon: f64,
}

/// An attached object (RFC 5545 §3.8.1.1).
///
/// Either a URI reference or inline binary data (base64-encoded on the wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attachment {
    Uri(Box<Uri>),
    Binary(Vec<u8>),
}

/// An error indicating that a string is not a valid FMTTYPE value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidFormatTypeError {
    #[error("expected at least one character")]
    EmptyString,
    #[error("missing '/' separator between type and subtype")]
    MissingSlash,
    #[error("empty type part before '/'")]
    EmptyType,
    #[error("empty subtype part after '/'")]
    EmptySubtype,
}

/// A media type/subtype pair (RFC 5545 §3.2.8, FMTTYPE parameter).
///
/// Format: `type/subtype` (e.g. `text/plain`, `image/png`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, dizzy::DstNewtype)]
#[dizzy(invariant = FormatType::str_is_format_type, error = InvalidFormatTypeError)]
#[dizzy(constructor = pub new)]
#[dizzy(getter = pub const as_str)]
#[dizzy(derive(Debug, CloneBoxed, IntoBoxed))]
#[dizzy(owned = pub FormatTypeBuf(String))]
#[dizzy(derive_owned(Debug, IntoBoxed))]
#[repr(transparent)]
pub struct FormatType(str);

impl FormatType {
    fn str_is_format_type(s: &str) -> Result<(), InvalidFormatTypeError> {
        if s.is_empty() {
            return Err(InvalidFormatTypeError::EmptyString);
        }

        let (type_part, subtype) = s
            .split_once('/')
            .ok_or(InvalidFormatTypeError::MissingSlash)?;

        if type_part.is_empty() {
            return Err(InvalidFormatTypeError::EmptyType);
        }
        if subtype.is_empty() {
            return Err(InvalidFormatTypeError::EmptySubtype);
        }

        Ok(())
    }

    /// Returns the type part (before `/`).
    #[inline(always)]
    pub fn type_part(&self) -> &str {
        self.as_str()
            .split_once('/')
            .expect("FormatType must contain /")
            .0
    }

    /// Returns the subtype part (after `/`).
    #[inline(always)]
    pub fn subtype(&self) -> &str {
        self.as_str()
            .split_once('/')
            .expect("FormatType must contain /")
            .1
    }
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for FormatTypeBuf {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for FormatTypeBuf {}

impl std::fmt::Display for FormatTypeBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A value of the STRUCTURED-DATA property (RFC 9073 §6.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredDataValue {
    Text(String),
    Binary(Vec<u8>),
    Uri(Box<Uri>),
}

/// A value of the STYLED-DESCRIPTION property (RFC 9073 §6.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyledDescriptionValue {
    Text(String),
    Uri(Box<Uri>),
    Iana { value_type: String, value: String },
}
