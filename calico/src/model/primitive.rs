//! Primitive types for the object model.
//!
//! Most types in this module are re-exported from the `calendar-types` and `rfc5545-types`
//! workspace crates. Extensible enum types use `Token<ClosedEnum, S>` from `calendar-types`
//! where `S` is the type of the unknown variant.
//!
//! The only type defined locally is [`Value`], which is a runtime-discriminated property value
//! tightly coupled to text-format parsing.

pub use mitsein::NonEmpty;

// ============================================================================
// Re-exports from calendar-types
// ============================================================================

pub use calendar_types::primitive::Sign;
pub use calendar_types::set::Token;
pub use calendar_types::string::LanguageTag as Language;
pub use calendar_types::time::{
    Date, DateTime, Day, Hour, IsoWeek, Local, Minute, Month, NonLeapSecond, Second, Time,
    TimeFormat, Utc, Weekday, Year,
};
pub use calendar_types::duration::{
    Duration, ExactDuration, NominalDuration, SignedDuration,
};

// ============================================================================
// Re-exports from rfc5545-types
// ============================================================================

pub use rfc5545_types::primitive::{Float, Integer, PositiveInteger};

pub use rfc5545_types::request_status::{Class, RequestStatus, StatusCode as RequestStatusCode};

pub use rfc5545_types::set::{
    AlarmAction, AudioAction, CalendarUserType, ClassValue, DisplayAction, DisplayType,
    EmailAction, Encoding, EventStatus, FeatureType, FreeBusyType, Gregorian, JournalStatus,
    Method, ParticipantType, ParticipationRole, ParticipationStatus, Priority, PriorityClass,
    ProximityValue, RelationshipType, ResourceType, Status, ThisAndFuture, TimeTransparency,
    TodoStatus, TriggerRelation, UnknownAction, ValueType, Version,
};

pub use rfc5545_types::set::Percent as CompletionPercentage;

pub use rfc5545_types::time::{
    DateTimeOrDate, ExDateSeq, Period, RDate, RDateSeq, TriggerValue, UtcOffset,
};

pub use rfc5545_types::value::{
    Attachment, FormatType, FormatTypeBuf, Geo, StructuredDataValue, StyledDescriptionValue,
};

// ============================================================================
// Value<S>
// ============================================================================

use super::{rrule::RRule, string::Uri};

/// A runtime-discriminated property value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<S> {
    Binary(Vec<u8>),
    Boolean(bool),
    CalAddress(Box<Uri>),
    Date(Date),
    DateTime(DateTime<TimeFormat>),
    Duration(SignedDuration),
    Float(Float),
    Integer(i32),
    Period(Period<TimeFormat>),
    Recur(RRule),
    Text(S),
    Time(Time, TimeFormat),
    Uri(Box<Uri>),
    UtcOffset(UtcOffset),
    Other { name: S, value: S },
}

impl<S> Value<S> {
    pub fn as_value_type(&self) -> Token<ValueType, &S> {
        match self {
            Value::Binary(_) => Token::Known(ValueType::Binary),
            Value::Boolean(_) => Token::Known(ValueType::Boolean),
            Value::CalAddress(_) => Token::Known(ValueType::CalAddress),
            Value::Date(_) => Token::Known(ValueType::Date),
            Value::DateTime(_) => Token::Known(ValueType::DateTime),
            Value::Duration(_) => Token::Known(ValueType::Duration),
            Value::Float(_) => Token::Known(ValueType::Float),
            Value::Integer(_) => Token::Known(ValueType::Integer),
            Value::Period(_) => Token::Known(ValueType::Period),
            Value::Recur(_) => Token::Known(ValueType::Recur),
            Value::Text(_) => Token::Known(ValueType::Text),
            Value::Time(..) => Token::Known(ValueType::Time),
            Value::Uri(_) => Token::Known(ValueType::Uri),
            Value::UtcOffset(_) => Token::Known(ValueType::UtcOffset),
            Value::Other { name, .. } => Token::Unknown(name),
        }
    }
}

// ============================================================================
// Macros
// ============================================================================

/// Constructs a [`UtcOffset`] from input of the form `+/-h;m(;s)?`.
#[macro_export]
macro_rules! utc_offset {
    (+ $h:expr;$m:expr $(; $s:expr)?) => {{
        let s: u8 = 0;
        $(let s: u8 = $s;)?

        $crate::model::primitive::UtcOffset {
            sign: $crate::model::primitive::Sign::Pos,
            hour: ::calendar_types::time::Hour::new($h).unwrap(),
            minute: ::calendar_types::time::Minute::new($m).unwrap(),
            second: ::calendar_types::time::NonLeapSecond::new(s).unwrap(),
        }
    }};
    (- $h:expr;$m:expr $(; $s:expr)?) => {{
        let s: u8 = 0;
        $(let s: u8 = $s;)?

        $crate::model::primitive::UtcOffset {
            sign: $crate::model::primitive::Sign::Neg,
            hour: ::calendar_types::time::Hour::new($h).unwrap(),
            minute: ::calendar_types::time::Minute::new($m).unwrap(),
            second: ::calendar_types::time::NonLeapSecond::new(s).unwrap(),
        }
    }};
}

/// Constructs a [`Date`] from input of the form `yyyy;MM;dd`. Panics if the date is invalid.
#[macro_export]
macro_rules! date {
    ($year:expr ; $month:expr ; $day:expr) => {
        ::calendar_types::time::Date::new(
            ::calendar_types::time::Year::new($year).unwrap(),
            ::calendar_types::time::Month::new($month).unwrap(),
            ::calendar_types::time::Day::new($day).unwrap(),
        )
        .unwrap()
    };
}

/// Constructs a [`Time`] from input of the form `hh;mm;ss`. Panics if the time is invalid.
#[macro_export]
macro_rules! time {
    ($hours:expr ; $minutes:expr ; $seconds:expr) => {
        ::calendar_types::time::Time::new(
            ::calendar_types::time::Hour::new($hours).unwrap(),
            ::calendar_types::time::Minute::new($minutes).unwrap(),
            ::calendar_types::time::Second::new($seconds).unwrap(),
            None,
        )
        .unwrap()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_offset_macro() {
        let pos_0800 = utc_offset!(+8;00);
        assert_eq!(pos_0800.sign, Sign::Pos);
        assert_eq!(pos_0800.hour, Hour::new(8).unwrap());
        assert_eq!(pos_0800.minute, Minute::new(0).unwrap());
        assert_eq!(pos_0800.second, NonLeapSecond::new(0).unwrap());

        let neg_160050 = utc_offset!(-16;00;50);
        assert_eq!(neg_160050.sign, Sign::Neg);
        assert_eq!(neg_160050.hour, Hour::new(16).unwrap());
        assert_eq!(neg_160050.minute, Minute::new(0).unwrap());
        assert_eq!(neg_160050.second, NonLeapSecond::new(50).unwrap());

        let neg_1737 = utc_offset!(-17;37);
        assert_eq!(neg_1737.sign, Sign::Neg);
        assert_eq!(neg_1737.hour, Hour::new(17).unwrap());
        assert_eq!(neg_1737.minute, Minute::new(37).unwrap());
        assert_eq!(neg_1737.second, NonLeapSecond::new(0).unwrap());
    }

    #[test]
    fn date_macro() {
        let xmas_2003 = date!(2003;12;25);
        let silvester_1957 = date!(1957;12;31);

        assert_eq!(xmas_2003.month(), silvester_1957.month());
    }

    #[test]
    fn time_macro() {
        let noon = time!(12;00;00);
        assert_eq!(noon.hour(), Hour::new(12).unwrap());
        assert_eq!(noon.minute(), Minute::new(0).unwrap());
        assert_eq!(noon.second(), Second::new(0).unwrap());
    }
}
