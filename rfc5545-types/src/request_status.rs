//! Types for describing request statuses.

/// A value of the REQUEST-STATUS property (RFC 5545 §3.8.8.3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestStatus {
    /// The hierarchical status code.
    pub code: StatusCode,
    /// A human-readable description of the status.
    pub description: Box<str>,
    /// Optional additional data about the status.
    pub exception_data: Option<Box<str>>,
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.code, self.description)?;
        if let Some(data) = &self.exception_data {
            write!(f, ";{data}")?;
        }
        Ok(())
    }
}

/// A hierarchical status code (e.g. `2.0`, `3.1.2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode {
    /// The status class (1–5).
    pub class: Class,
    /// The major status number within the class.
    pub major: u8,
    /// The optional minor status number.
    pub minor: Option<u8>,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.class.as_u8(), self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
        Ok(())
    }
}

/// The class of a [`StatusCode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    /// Preliminary success.
    ///
    /// The request has been processed but completion is pending.
    C1,
    /// Complete success.
    ///
    /// The request has been completed successfully, although based on the specific status code a
    /// fallback may have been taken.
    C2,
    /// Client error.
    ///
    /// The request was not successful, due to a syntactic or semantic error in the
    /// client-formatted request. The request should not be retried until the error condition in
    /// the request has been corrected.
    C3,
    /// Scheduling error.
    ///
    /// The request was not successful due to a semantic issue with the calendaring and scheduling
    /// service not directly related to the request itself.
    C4,
    /// Service error.
    ///
    /// The request was not successful because a service either did not exist, was not available,
    /// or was unsupported.
    C5,
}

impl Class {
    /// Returns the numeric value of this class (1–5).
    pub const fn as_u8(self) -> u8 {
        match self {
            Class::C1 => 1,
            Class::C2 => 2,
            Class::C3 => 3,
            Class::C4 => 4,
            Class::C5 => 5,
        }
    }

    /// Creates a `Class` from a numeric value (1–5), returning `None` if out of range.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Class::C1),
            2 => Some(Class::C2),
            3 => Some(Class::C3),
            4 => Some(Class::C4),
            5 => Some(Class::C5),
            _ => None,
        }
    }
}
