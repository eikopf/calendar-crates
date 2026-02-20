//! Error types for parsing iCalendar.

use std::convert::Infallible;

// TODO: replace this reexport with a custom error enum that provides a subset of the variants
// (since some error variants are impossible).

pub use lexical_parse_float::Error as ParseFloatError;

use crate::{
    model::{
        component::TzRuleKind,
        parameter::{KnownParam, SDParamsFromParamsError, StaticParam},
        primitive::{Integer, Sign, Status, ValueType},
        rrule,
    },
    parser::property::PropName,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CalendarParseError<S> {
    // errors from dependencies
    Utf8Error(std::str::Utf8Error),
    Base64DecodeError(base64::DecodeError),
    LanguageParseError(language_tags::ParseError),
    FloatToF64Failure(ParseFloatError),
    // string newtype errors
    InvalidCharInParamValue(char),
    // primitive parser errors
    InvalidFormatType(S),
    InvalidPositiveInteger(Integer),
    InvalidRawTime(InvalidRawTimeError),
    InvalidUtcOffset(InvalidUtcOffsetError),
    InvalidDate(InvalidDateError),
    InvalidInteger(InvalidIntegerError),
    InvalidGeo(InvalidGeoError),
    InvalidCompletionPercentage(InvalidCompletionPercentageError),
    InvalidPriority(InvalidPriorityError),
    InvalidDurationTime(InvalidDurationTimeError),
    /// Received the interval 0 in a recurrence rule, which must be a
    /// positive integer.
    ZeroInterval,
    /// Expected an ISO week index, got a value outside the range `1..=53`.
    InvalidIsoWeekIndex(u8),
    /// Expected a month day index, got a value outside the range `1..=31`.
    InvalidMonthDayIndex(u8),
    /// Expected a month number, got a value outside the range `1..=12`.
    InvalidMonthNumber(u8),
    /// Expected an hour index, got a value outside the range `0..=23`.
    InvalidHourIndex(u8),
    /// Expected a minute index, got a value outside the range `0..=59`.
    InvalidMinuteIndex(u8),
    /// Expected a second index, got a value outside the range `0..=60`.
    InvalidSecondIndex(u8),
    /// Received a part in a recurrence rule more than once.
    DuplicateRRulePart(rrule::PartName),
    /// Both the COUNT and UNTIL parts occurred in the same RRULE.
    CountAndUntilInRRule,
    /// The FREQ part did not occur in an RRULE.
    MissingFreqPart,
    /// A BYxxx rule occurred that was inadmissible for the current FREQ value.
    UnexpectedByRule {
        freq: rrule::Freq,
        by_rule: rrule::ByRuleName,
    },
    /// A parameter with a multiplicity less than 2 occurred more than once.
    DuplicateParam(StaticParam),
    /// A parameter was expected but not present.
    MissingParam(StaticParam),
    /// Two parameters were expected but not present.
    MissingParam2(StaticParam, StaticParam),
    /// A VALUE parameter was expected but did not occur.
    MissingValueType,
    /// A VALUE parameter was present but had an invalid value for the property.
    InvalidValueType(ValueType<S>),
    /// A property with the BINARY value did not also have the ENCODING parameter.
    MissingEncodingOnBinaryValue,
    /// A property with the BINARY value had ENCODING=8bit as a parameter.
    Bit8EncodingOnBinaryValue,
    UnexpectedProp {
        prop: PropName<S>,
        component: ComponentKind<S>,
    },
    MissingProp {
        prop: PropName<S>,
        component: ComponentKind<S>,
    },
    DurationWithoutRepeat,
    RepeatWithoutDuration,
    TooManyAttachmentsOnAudioAlarm,
    InvalidEventStatus(Status),
    InvalidTodoStatus(Status),
    InvalidJournalStatus(Status),
    MoreThanOneProp {
        prop: PropName<S>,
        component: ComponentKind<S>,
    },
    /// Both DTEND and DURATION occurred in the same VEVENT.
    EventTerminationCollision,
    /// Both DTDUE and DURATION occurred in the same VTODO.
    TodoTerminationCollision,
    /// The ORDER parameter occurred on a property that cannot occur more than once.
    OrderOnNonRepeatableProp,
}

impl<S> From<language_tags::ParseError> for CalendarParseError<S> {
    fn from(v: language_tags::ParseError) -> Self {
        Self::LanguageParseError(v)
    }
}

impl<S> From<base64::DecodeError> for CalendarParseError<S> {
    fn from(v: base64::DecodeError) -> Self {
        Self::Base64DecodeError(v)
    }
}

impl<S> From<Infallible> for CalendarParseError<S> {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl<S> From<std::str::Utf8Error> for CalendarParseError<S> {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl<S> From<SDParamsFromParamsError> for CalendarParseError<S> {
    fn from(value: SDParamsFromParamsError) -> Self {
        match value {
            SDParamsFromParamsError::MissingFormatType => {
                Self::MissingParam(StaticParam::FormatType)
            }
            SDParamsFromParamsError::MissingSchema => Self::MissingParam(StaticParam::Schema),
            SDParamsFromParamsError::MissingFormatTypeAndSchema => {
                Self::MissingParam2(StaticParam::FormatType, StaticParam::Schema)
            }
        }
    }
}

/// A component kind, including the static subcomponents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind<S> {
    Calendar,
    Event,
    Todo,
    Journal,
    FreeBusy,
    TimeZone,
    Alarm,
    AudioAlarm,
    DisplayAlarm,
    EmailAlarm,
    Standard,
    Daylight,
    StandardOrDaylight,
    Iana(S),
    X(S),
    /// Iana or X without a specific name.
    Unknown,
}

impl<S> From<TzRuleKind> for ComponentKind<S> {
    fn from(value: TzRuleKind) -> Self {
        match value {
            TzRuleKind::Standard => Self::Standard,
            TzRuleKind::Daylight => Self::Daylight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRawTimeError {
    pub(crate) hours: u8,
    pub(crate) minutes: u8,
    pub(crate) seconds: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidUtcOffsetError {
    NegativeZero,
    BadHours(u8),
    BadMinutes(u8),
    BadSeconds(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDateError {
    pub(crate) year: u16,
    pub(crate) month: u8,
    pub(crate) day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIntegerError {
    pub(crate) sign: Option<Sign>,
    pub(crate) digits: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidGeoError {
    LatOutOfBounds(f64),
    LonOutOfBounds(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCompletionPercentageError(pub(crate) Integer);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPriorityError(pub(crate) Integer);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDurationTimeError<T = usize> {
    pub(crate) hours: Option<T>,
    pub(crate) seconds: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedKnownParamError<S> {
    pub(crate) current_property: PropName<S>,
    pub(crate) unexpected_param: KnownParam,
}
