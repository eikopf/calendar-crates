//! Parser configurations

use crate::{
    model::string::ParamValue,
    parser::error::{CalendarParseError, ParseFloatError},
};

/// A trait providing customizable behaviour for a parser.
pub trait Config {
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

/// A unit struct that implements [`Config`].
#[derive(Debug, Clone, Copy)]
pub struct DefaultConfig;

impl Config for DefaultConfig {}
