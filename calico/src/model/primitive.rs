//! Primitive types for the object model.
//!
//! # Type Parameters
//!
//! There are two primary groups of type parameters on the types in this module;
//! they are called `S` and `F` by convention.
//!
//! The `S` parameter denotes the _source_ of a type, i.e. the underlying data
//! over which the type is providing a view. Typically this is a slice of the
//! parsed input, e.g. `&str`, `&[u8]`, or
//! [`Escaped`](crate::parser::escaped::Escaped).
//!
//! The `F` parameter is the _time format_ of a type. This is used to distinguish
//! between temporal values which are strictly [`Local`], strictly in reference
//! to [`Utc`], or which may have either a local or absolute [`TimeFormat`].

use std::num::NonZero;

use super::{
    rrule::RRule,
    string::{NameKind, Text, Uri},
};

use functor_derive::Functor;
pub use mitsein::NonEmpty;

// Types re-exported from workspace crates
pub use calendar_types::time::{IsoWeek, Weekday};
pub use rfc5545_types::set::{
    Encoding, EventStatus, Gregorian, JournalStatus, Priority, PriorityClass, ThisAndFuture,
    TimeTransparency, TodoStatus, TriggerRelation, Version,
};
pub use rfc5545_types::value::Geo;

/// The INTEGER type as defined in RFC 5545 §3.3.8.
pub type Integer = i32;

/// The FLOAT type as defined in RFC 5545 §3.3.7.
pub type Float = f64;

/// A strictly positive [`Integer`].
pub type PositiveInteger = NonZero<u32>;

/// A method as defined in RFC 5546 §1.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum Method<S> {
    Publish,
    Request,
    Reply,
    Add,
    Cancel,
    Refresh,
    Counter,
    DeclineCounter,
    Other(S),
}

/// An RFC 5646 language tag.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Language(pub(crate) language_tags::LanguageTag);

/// Date-time or date value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeOrDate<F = TimeFormat> {
    DateTime(DateTime<F>),
    Date(Date),
}

impl<F> From<Date> for DateTimeOrDate<F> {
    fn from(value: Date) -> Self {
        Self::Date(value)
    }
}

impl<F> From<DateTime<F>> for DateTimeOrDate<F> {
    fn from(value: DateTime<F>) -> Self {
        Self::DateTime(value)
    }
}

/// A homogeneous sequence of either datetimes or dates. Used primarily as the
/// value type for the EXDATE property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExDateSeq<F = TimeFormat> {
    DateTime(Vec<DateTime<F>>),
    Date(Vec<Date>),
}

impl<F> DateTimeOrDate<F> {
    /// Returns `true` if the date time or date is [`Date`].
    ///
    /// [`Date`]: DateTimeOrDate::Date
    #[must_use]
    pub fn is_date(&self) -> bool {
        matches!(self, Self::Date(..))
    }

    /// Returns `true` if the date time or date is [`DateTime`].
    ///
    /// [`DateTime`]: DateTimeOrDate::DateTime
    #[must_use]
    pub fn is_date_time(&self) -> bool {
        matches!(self, Self::DateTime(..))
    }
}

/// The product of a [`Date`] and a [`Time`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime<F = TimeFormat> {
    pub date: Date,
    pub time: Time<F>,
}

/// A DATE value (RFC 5545, §3.3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub(crate) year: u16,
    pub(crate) month: NonZero<u8>,
    pub(crate) day: NonZero<u8>,
}

impl Date {
    /// Constructs a [`Date`] from a year, month (1-indexed), and day (1-indexed). The year may not
    /// exceed 9999, the month may not exceed 12, and the day may not exceed 31.
    pub const fn from_ymd_opt(year: u16, month: u8, day: u8) -> Option<Self> {
        if year > 9999 || month > 12 || day > 31 {
            return None;
        }

        let Some(month) = NonZero::new(month) else {
            return None;
        };

        let Some(day) = NonZero::new(day) else {
            return None;
        };

        Some(Self { year, month, day })
    }
}

/// A TIME value (RFC 5545, §3.3.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time<F = TimeFormat> {
    pub raw: RawTime,
    pub format: F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

/// A marker struct for absolute UTC time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Utc;

/// A marker struct for local time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Local;

/// The format of a [`Time`], which may be local or absolute.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Local,
    Utc,
}

impl From<Utc> for TimeFormat {
    fn from(Utc: Utc) -> Self {
        Self::Utc
    }
}

impl From<Local> for TimeFormat {
    fn from(Local: Local) -> Self {
        Self::Local
    }
}

/// RFC 5545 §3.2.8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatType(pub(crate) mime::Mime);

/// DISPLAY parameter values (RFC 7986)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum DisplayType<S> {
    Badge,
    Graphic,
    Fullsize,
    Thumbnail,
    Other(S),
}

/// FEATURE parameter values (RFC 7986)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum FeatureType<S> {
    Audio,
    Chat,
    Feed,
    Moderator,
    Phone,
    Screen,
    Video,
    Other(S),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum CalendarUserType<S> {
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    Other(S),
}

impl<S> Default for CalendarUserType<S> {
    fn default() -> Self {
        Self::Individual
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

/// A value of the RESOURCE-TYPE property (RFC 9073 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ResourceType<S> {
    Room,
    Projector,
    RemoteConferenceAudio,
    RemoteConferenceVideo,
    Other(S),
}

/// A value of the PARTICIPANT-TYPE property (RFC 9073 §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ParticipantType<S> {
    Active,
    Inactive,
    Sponsor,
    Contact,
    BookingContact,
    EmergencyContact,
    PublicityContact,
    PlannerContact,
    Performer,
    Speaker,
    Other(S),
}

/// A value of the ROLE parameter (RFC 5545 §3.2.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ParticipationRole<S> {
    Chair,
    ReqParticipant,
    OptParticipant,
    NonParticipant,
    Other(S),
}

impl<S> Default for ParticipationRole<S> {
    fn default() -> Self {
        Self::ReqParticipant
    }
}

/// A unified status value covering all component types (RFC 5545 §3.8.1.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Tentative,
    Confirmed,
    Cancelled,
    NeedsAction,
    Completed,
    InProcess,
    Draft,
    Final,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ParticipationStatus<S> {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    Completed,
    InProcess,
    Other(S),
}

impl<S> Default for ParticipationStatus<S> {
    fn default() -> Self {
        Self::NeedsAction
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum FreeBusyType<S> {
    Free,
    Busy,
    BusyUnavailable,
    BusyTentative,
    Other(S),
}

/// The [`Audio`] alarm action.
///
/// [`Audio`]: AlarmAction::Audio
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct AudioAction;
/// The [`Display`] alarm action.
///
/// [`Display`]: AlarmAction::Display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DisplayAction;
/// The [`Email`] alarm action.
///
/// [`Email`]: AlarmAction::Email
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct EmailAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownAction<S> {
    Iana(S),
    X(S),
}

impl<S> UnknownAction<S> {
    pub const fn as_ref(&self) -> UnknownAction<&S> {
        match self {
            UnknownAction::Iana(action) => UnknownAction::Iana(action),
            UnknownAction::X(action) => UnknownAction::X(action),
        }
    }

    pub const fn kind(&self) -> NameKind {
        match self {
            UnknownAction::Iana(_) => NameKind::Iana,
            UnknownAction::X(_) => NameKind::X,
        }
    }

    pub fn into_inner(self) -> S {
        match self {
            UnknownAction::Iana(action) | UnknownAction::X(action) => action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum AlarmAction<S> {
    Audio,
    Display,
    Email,
    Other(S),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerValue {
    Duration(Duration),
    DateTime(DateTime<Utc>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum RelationshipType<S> {
    Parent,
    Child,
    Sibling,
    /// RFC 9074 §7.1.
    Snooze,
    Other(S),
}

/// A proximity value (RFC 9074 §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ProximityValue<S> {
    Arrive,
    Depart,
    Connect,
    Disconnect,
    Other(S),
}

/// The type of a [`Value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ValueType<S> {
    Binary,
    Boolean,
    CalAddress,
    Date,
    DateTime,
    Duration,
    Float,
    Integer,
    Period,
    Recur,
    Text,
    Time,
    Uri,
    UtcOffset,
    Other(S),
}

///A runtime-discriminated property value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<S> {
    Binary(Vec<u8>),
    Boolean(bool),
    CalAddress(Box<Uri>),
    Date(Date),
    DateTime(DateTime),
    Duration(Duration),
    Float(Float),
    Integer(i32),
    Period(Period),
    Recur(RRule),
    Text(S),
    Time(Time),
    Uri(Box<Uri>),
    UtcOffset(UtcOffset),
    Other { name: S, value: S },
}

impl<S> Value<S> {
    pub fn as_value_type(&self) -> ValueType<&S> {
        match self {
            Value::Binary(_) => ValueType::Binary,
            Value::Boolean(_) => ValueType::Boolean,
            Value::CalAddress(_) => ValueType::CalAddress,
            Value::Date(_) => ValueType::Date,
            Value::DateTime(_) => ValueType::DateTime,
            Value::Duration(_) => ValueType::Duration,
            Value::Float(_) => ValueType::Float,
            Value::Integer(_) => ValueType::Integer,
            Value::Period(_) => ValueType::Period,
            Value::Recur(_) => ValueType::Recur,
            Value::Text(_) => ValueType::Text,
            Value::Time(_) => ValueType::Time,
            Value::Uri(_) => ValueType::Uri,
            Value::UtcOffset(_) => ValueType::UtcOffset,
            Value::Other { name, .. } => ValueType::Other(name),
        }
    }
}

/// An attached object, either inline as a binary blob or referenced by a URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attachment {
    Uri(Box<Uri>),
    Binary(Vec<u8>),
}

/// The value type of the `CLASS` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Functor)]
pub enum ClassValue<S> {
    Public,
    Private,
    Confidential,
    Other(S),
}

impl<S> Default for ClassValue<S> {
    fn default() -> Self {
        Self::Public
    }
}

/// Period of time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period<F = TimeFormat> {
    Explicit {
        start: DateTime<F>,
        end: DateTime<F>,
    },
    Start {
        start: DateTime<F>,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RDate<F = TimeFormat> {
    DateTime(DateTime<F>),
    Date(Date),
    Period(Period),
}

/// A homogeneous sequence of [`RDate`] values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RDateSeq<F = TimeFormat> {
    DateTime(Vec<DateTime<F>>),
    Date(Vec<Date>),
    Period(Vec<Period>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Positive = 1,
    Negative = -1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub sign: Option<Sign>,
    pub kind: DurationKind,
}

/// The kind of a [`Duration`]. The type parameter `T` is the underlying integer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationKind<T = usize> {
    /// Some number of days with an optional time duration.
    Date {
        days: T,
        time: Option<DurationTime<T>>,
    },
    /// An exact time duration.
    Time { time: DurationTime<T> },
    /// Some number of weeks.
    Week { weeks: T },
}

/// The time portion of a [`Duration`], measured in hours, minutes, and seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationTime<T = usize> {
    HMS { hours: T, minutes: T, seconds: T },
    HM { hours: T, minutes: T },
    MS { minutes: T, seconds: T },
    H { hours: T },
    M { minutes: T },
    S { seconds: T },
}

/// An integer in the range `0..=100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompletionPercentage(pub(crate) u8);

/// A UTC offset (RFC 5545 §3.3.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset {
    pub sign: Sign,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: Option<u8>,
}

/// One of the twelve Gregorian months.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    /// Returns the month number of `self`, which lies in the range `1..=12`.
    pub const fn number(&self) -> NonZero<u8> {
        // SAFETY: the expression `(*self as u8) + 1` is in the range 1..=12.
        unsafe { NonZero::new_unchecked((*self as u8) + 1) }
    }

    pub const fn from_number(number: u8) -> Option<Self> {
        match number {
            1..=12 => {
                // SAFETY: (1..=12) - 1 is effectively 0..=11, which are all
                // valid discriminants of Month
                Some(unsafe { std::mem::transmute::<u8, Self>(number - 1) })
            }
            _ => None,
        }
    }

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Jan,
            Self::Feb,
            Self::Mar,
            Self::Apr,
            Self::May,
            Self::Jun,
            Self::Jul,
            Self::Aug,
            Self::Sep,
            Self::Oct,
            Self::Nov,
            Self::Dec,
        ]
        .iter()
        .copied()
    }
}

/// A value of the REQUEST-STATUS property (RFC 5545 §3.8.8.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestStatus {
    pub code: RequestStatusCode,
    pub description: Box<Text>,
    pub exception_data: Option<Box<Text>>,
}

/// A status code for the REQUEST-STATUS property (RFC 5545 §3.8.8.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestStatusCode<T = u8>(pub(crate) T, pub(crate) T, pub(crate) Option<T>);

impl<T> From<(T, T)> for RequestStatusCode<T> {
    fn from((a, b): (T, T)) -> Self {
        Self(a, b, None)
    }
}

impl<T> From<(T, T, T)> for RequestStatusCode<T> {
    fn from((a, b, c): (T, T, T)) -> Self {
        Self(a, b, Some(c))
    }
}

#[macro_export]
macro_rules! utc_offset {
    (+ $h:expr;$m:expr $(; $s:expr)?) => {
        {
        let s: Option<u8> = None;
        $(let s = Some($s);)?

        $crate::model::primitive::UtcOffset {
            sign: $crate::model::primitive::Sign::Positive,
            hours: $h,
            minutes: $m,
            seconds: s,
        }
        }
    };
    (- $h:expr;$m:expr $(; $s:expr)?) => {
        {
        let _s: Option<u8> = None;
        $(let _s = Some($s);)?

        $crate::model::primitive::UtcOffset {
            sign: $crate::model::primitive::Sign::Negative,
            hours: $h,
            minutes: $m,
            seconds: _s,
        }
        }
    };
}

/// Constructs a [`Date`] from input of the form `yyyy;MM;dd`. Will panic if
/// the given date is invalid according to [`Date::from_ymd_opt`].
#[macro_export]
macro_rules! date {
    ($year:expr ; $month:expr ; $day:expr) => {
        $crate::model::primitive::Date::from_ymd_opt($year, $month, $day).unwrap()
    };
}

/// Constructs a [`Time`] from input of the form `hh;mm;ss, <format>`.
#[macro_export]
macro_rules! time {
    ($hours:expr ; $minutes:expr ; $seconds:expr, $format:ident) => {
        $crate::model::primitive::Time {
            raw: $crate::model::primitive::RawTime {
                hours: $hours,
                minutes: $minutes,
                seconds: $seconds,
            },
            format: $format.into(),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_offset_macro() {
        let pos_0800 = utc_offset!(+8;00);
        assert_eq!(pos_0800.sign, Sign::Positive);
        assert_eq!(pos_0800.hours, 8);
        assert_eq!(pos_0800.minutes, 0);
        assert!(pos_0800.seconds.is_none());

        let neg_160050 = utc_offset!(-16;00;50);
        assert_eq!(neg_160050.sign, Sign::Negative);
        assert_eq!(neg_160050.hours, 16);
        assert_eq!(neg_160050.minutes, 0);
        assert_eq!(neg_160050.seconds, Some(50));

        let neg_1737 = utc_offset!(-17;37);
        assert_eq!(neg_1737.sign, Sign::Negative);
        assert_eq!(neg_1737.hours, 17);
        assert_eq!(neg_1737.minutes, 37);
        assert_eq!(neg_1737.seconds, None);
    }

    #[test]
    fn date_macro() {
        let xmas_2003 = date!(2003;12;25);
        let silvester_1957 = date!(1957;12;31);

        assert_eq!(xmas_2003.month, silvester_1957.month);
    }

    #[test]
    fn time_macro() {
        let noon_utc: Time<Utc> = time!(12;00;00, Utc);
        let noon_utc_tf: Time<TimeFormat> = time!(12;00;00, Utc);
        let noon_local: Time<Local> = time!(12;00;00, Local);
        let noon_local_tf: Time<TimeFormat> = time!(12;00;00, Local);

        let noon_raw = RawTime {
            hours: 12,
            minutes: 0,
            seconds: 0,
        };

        assert_eq!(noon_utc.raw, noon_raw);
        assert_eq!(noon_utc_tf.raw, noon_raw);
        assert_eq!(noon_local.raw, noon_raw);
        assert_eq!(noon_local_tf.raw, noon_raw);

        assert_eq!(noon_utc_tf.format, TimeFormat::Utc);
        assert_eq!(noon_local_tf.format, TimeFormat::Local);
    }

    #[test]
    fn raw_time_ord_impl() {
        assert!(
            RawTime {
                hours: 12,
                minutes: 0,
                seconds: 0
            } < RawTime {
                hours: 12,
                minutes: 30,
                seconds: 0
            }
        );
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn priority_partial_ord() {
        assert_eq!(Priority::Zero.partial_cmp(&Priority::Zero), None);
        assert_eq!(Priority::Zero.partial_cmp(&Priority::A1), None);
        assert_eq!(Priority::A1.partial_cmp(&Priority::Zero), None);

        assert!(Priority::A1 > Priority::A2);
        assert!(Priority::A2 > Priority::A3);
        assert!(Priority::A1 > Priority::A3);

        assert!(Priority::A1 > Priority::B1);
        assert!(Priority::B1 > Priority::C1);
        assert!(Priority::A1 > Priority::C1);

        assert!(!(Priority::Zero > Priority::C2));
        assert!(!(Priority::Zero < Priority::C2));
        assert!(Priority::Zero != Priority::C2);
    }

    #[test]
    fn priority_class_predicates() {
        assert!(!Priority::Zero.is_low());
        assert!(!Priority::Zero.is_medium());
        assert!(!Priority::Zero.is_high());

        assert!(Priority::A1.is_high());
        assert!(Priority::A2.is_high());
        assert!(Priority::A3.is_high());
        assert!(Priority::B1.is_high());

        assert!(!Priority::B2.is_high());
        assert!(Priority::B2.is_medium());
        assert!(!Priority::B2.is_low());

        assert!(Priority::B3.is_low());
        assert!(Priority::C1.is_low());
        assert!(Priority::C2.is_low());
        assert!(Priority::C3.is_low());
    }

    #[test]
    fn priority_class_projection() {
        assert_eq!(Priority::Zero.into_class(), None);

        assert_eq!(Priority::A1.into_class(), Some(PriorityClass::High));
        assert_eq!(Priority::A2.into_class(), Some(PriorityClass::High));
        assert_eq!(Priority::A3.into_class(), Some(PriorityClass::High));
        assert_eq!(Priority::B1.into_class(), Some(PriorityClass::High));

        assert_eq!(Priority::B2.into_class(), Some(PriorityClass::Medium));

        assert_eq!(Priority::B3.into_class(), Some(PriorityClass::Low));
        assert_eq!(Priority::C1.into_class(), Some(PriorityClass::Low));
        assert_eq!(Priority::C2.into_class(), Some(PriorityClass::Low));
        assert_eq!(Priority::C3.into_class(), Some(PriorityClass::Low));
    }

    #[test]
    fn iso_week_from_index() {
        assert_eq!(IsoWeek::from_index(0), None);
        assert_eq!(IsoWeek::from_index(1), Some(IsoWeek::W1));
        assert_eq!(IsoWeek::from_index(2), Some(IsoWeek::W2));
        assert_eq!(IsoWeek::from_index(3), Some(IsoWeek::W3));
        assert_eq!(IsoWeek::from_index(4), Some(IsoWeek::W4));
        assert_eq!(IsoWeek::from_index(5), Some(IsoWeek::W5));
        // ...
        assert_eq!(IsoWeek::from_index(25), Some(IsoWeek::W25));
        assert_eq!(IsoWeek::from_index(26), Some(IsoWeek::W26));
        assert_eq!(IsoWeek::from_index(27), Some(IsoWeek::W27));
        // ...
        assert_eq!(IsoWeek::from_index(51), Some(IsoWeek::W51));
        assert_eq!(IsoWeek::from_index(52), Some(IsoWeek::W52));
        assert_eq!(IsoWeek::from_index(53), Some(IsoWeek::W53));
        assert_eq!(IsoWeek::from_index(54), None);
        assert_eq!(IsoWeek::from_index(55), None);
        //...
        assert_eq!(IsoWeek::from_index(254), None);
        assert_eq!(IsoWeek::from_index(255), None);
    }

    #[test]
    fn sign_ord_impl() {
        assert!(Sign::Negative < Sign::Positive);
    }
}
