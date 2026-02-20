//! Types for describing request statuses.

/// A value of the REQUEST-STATUS property (RFC 5545 §3.8.8.3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestStatus {
    pub code: StatusCode,
    pub description: Box<str>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode {
    pub class: Class,
    pub major: u8,
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
    pub const fn as_u8(self) -> u8 {
        match self {
            Class::C1 => 1,
            Class::C2 => 2,
            Class::C3 => 3,
            Class::C4 => 4,
            Class::C5 => 5,
        }
    }
}
