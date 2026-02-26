//! Parser configurations

use crate::{
    model::string::ParamValue,
    parser::error::{CalendarParseError, ParseFloatError},
};

/// The line ending convention used in an iCalendar document.
///
/// RFC 5545 mandates CRLF (`\r\n`), but many real-world `.ics` files use bare LF (`\n`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnding {
    /// `\r\n`
    Crlf,
    /// `\n`
    Lf,
}

impl LineEnding {
    /// Auto-detect the line ending convention from the input by finding the first `\n` and
    /// checking whether it is preceded by `\r`.
    pub fn detect(input: &[u8]) -> LineEnding {
        for (i, &b) in input.iter().enumerate() {
            if b == b'\n' {
                if i > 0 && input[i - 1] == b'\r' {
                    return LineEnding::Crlf;
                } else {
                    return LineEnding::Lf;
                }
            }
        }
        // No newline found; default to CRLF.
        LineEnding::Crlf
    }
}

/// A trait providing customizable behaviour for a parser.
pub trait Config {
    /// Returns the line ending convention to use when parsing line terminators between properties.
    fn line_ending(&self) -> LineEnding {
        LineEnding::Crlf
    }

    /// Sets the line ending convention. The default implementation is a no-op.
    fn set_line_ending(&mut self, _le: LineEnding) {}

    /// Updates the value of an existing parameter when a duplicate parameter is encountered. The
    /// default behaviour simply appends the `new_value` to the `previous_value` and returns
    /// `Ok(())`, but (for example) a simpler implementation might just replace the `previous_value`
    /// with the `new_value` (this is the behaviour of `ical.js`).
    fn handle_duplicate_param<S>(
        &mut self,
        previous_value: &mut Vec<Box<ParamValue>>,
        mut new_value: Vec<Box<ParamValue>>,
    ) -> Result<(), CalendarParseError<S>> {
        previous_value.append(&mut new_value);
        Ok(())
    }

    /// Called by [`float_with_config`] if [`lexical_parse_float`] fails to convert the parsed
    /// float string into an [`f64`]. The parsed string and error are passed as parameters to this
    /// function, and it may either return an error or produce a substitute `f64` value. The
    /// default behaviour is to simply return the passed error.
    ///
    /// The string parameter is guaranteed to a well-formed float as described by RFC 5545 § 3.3.7.
    ///
    /// [`float_with_config`]: crate::parser::primitive::float_with_config
    fn handle_float_parse_failure<S>(
        &mut self,
        _slice: &str,
        error: ParseFloatError,
    ) -> Result<f64, CalendarParseError<S>> {
        Err(CalendarParseError::FloatToF64Failure(error))
    }
}

/// A struct that implements [`Config`] with configurable line ending.
#[derive(Debug, Clone, Copy)]
pub struct DefaultConfig {
    line_ending: LineEnding,
}

impl DefaultConfig {
    /// Creates a new `DefaultConfig` with the given line ending.
    pub fn new(line_ending: LineEnding) -> Self {
        Self { line_ending }
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            line_ending: LineEnding::Crlf,
        }
    }
}

impl Config for DefaultConfig {
    fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    fn set_line_ending(&mut self, le: LineEnding) {
        self.line_ending = le;
    }
}
