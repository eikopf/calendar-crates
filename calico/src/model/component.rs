//! Model types for iCalendar components.

use structible::structible;

use super::{
    css::Css3Color,
    parameter::Params,
    primitive::{
        Attachment, ClassValue, CompletionPercentage, DateTime, DateTimeOrDate, Geo, Gregorian,
        Integer, Method, ParticipantType, Period, Priority, RDateSeq,
        RequestStatus, ResourceType, SignedDuration, Status, StyledDescriptionValue,
        TimeTransparency, Token, TriggerValue, Utc, UtcOffset, Value, Version,
    },
    property::{Prop, StructuredDataProp},
    rrule::RRule,
    string::{CaselessStr, TzId, Uid, Uri},
};

// ============================================================================
// Calendar (RFC 5545 §3.4)
// ============================================================================

/// An iCalendar object (RFC 5545 §3.4).
#[structible]
pub struct Calendar {
    // Required
    pub version: Prop<Token<Version, String>, Params>,

    // Required
    pub prod_id: Prop<String, Params>,

    // Optional (at most once)
    pub cal_scale: Option<Prop<Token<Gregorian, String>, Params>>,
    pub method: Option<Prop<Token<Method, String>, Params>>,

    // RFC 7986 optional
    pub uid: Option<Prop<Box<Uid>, Params>>,
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,
    pub refresh_interval: Option<Prop<SignedDuration, Params>>,
    pub source: Option<Prop<Box<Uri>, Params>>,
    pub color: Option<Prop<Css3Color, Params>>,

    // RFC 7986 multi-valued
    pub name: Option<Vec<Prop<String, Params>>>,
    pub description: Option<Vec<Prop<String, Params>>>,
    pub categories: Option<Vec<Prop<Vec<String>, Params>>>,
    pub image: Option<Vec<Prop<Attachment, Params>>>,

    // Subcomponents
    pub components: Vec<CalendarComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

impl Calendar {
    /// Parses an iCalendar stream from a string, returning zero or more [`Calendar`] objects.
    pub fn parse(s: &str) -> Result<Vec<Calendar>, crate::parser::error::ParseError> {
        use crate::parser::{
            component::icalendar_stream, error::ParseError, escaped::AsEscaped,
        };
        let total = s.len();
        let mut input = s.as_escaped();
        icalendar_stream::<_, ParseError>(&mut input).map_err(|e| e.with_total_len(total))
    }

    /// Parses an iCalendar stream from a byte slice, returning zero or more [`Calendar`] objects.
    pub fn parse_bytes(b: &[u8]) -> Result<Vec<Calendar>, crate::parser::error::ParseError> {
        use crate::parser::{
            component::icalendar_stream, error::ParseError, escaped::AsEscaped,
        };
        let total = b.len();
        let mut input = b.as_escaped();
        icalendar_stream::<_, ParseError>(&mut input).map_err(|e| e.with_total_len(total))
    }
}

// ============================================================================
// CalendarComponent enum
// ============================================================================

/// An immediate subcomponent of a [`Calendar`].
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarComponent {
    Event(Event),
    Todo(Todo),
    Journal(Journal),
    FreeBusy(FreeBusy),
    TimeZone(TimeZone),
    Other(OtherComponent),
}

// ============================================================================
// Event (RFC 5545 §3.6.1)
// ============================================================================

/// A VEVENT component (RFC 5545 §3.6.1).
#[structible]
pub struct Event {
    // Required by RFC 5545, but omitted by many producers
    pub dtstamp: Option<Prop<DateTime<Utc>, Params>>,
    pub uid: Option<Prop<Box<Uid>, Params>>,

    // Optional (at most once)
    pub dtstart: Option<Prop<DateTimeOrDate, Params>>,
    pub class: Option<Prop<Token<ClassValue, String>, Params>>,
    pub created: Option<Prop<DateTime<Utc>, Params>>,
    pub description: Option<Prop<String, Params>>,
    pub geo: Option<Prop<Geo, Params>>,
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub location: Option<Prop<String, Params>>,
    pub organizer: Option<Prop<Box<Uri>, Params>>,
    pub priority: Option<Prop<Priority, Params>>,
    pub sequence: Option<Prop<Integer, Params>>,
    pub status: Option<Prop<Status, Params>>,
    pub summary: Option<Prop<String, Params>>,
    pub transp: Option<Prop<TimeTransparency, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,
    pub recurrence_id: Option<Prop<DateTimeOrDate, Params>>,
    pub dtend: Option<Prop<DateTimeOrDate, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub color: Option<Prop<Css3Color, Params>>,

    // Multi-valued
    pub attach: Option<Vec<Prop<Attachment, Params>>>,
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub categories: Option<Vec<Prop<Vec<String>, Params>>>,
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub contact: Option<Vec<Prop<String, Params>>>,
    pub exdate: Option<Vec<Prop<DateTimeOrDate, Params>>>,
    pub request_status: Option<Vec<Prop<RequestStatus, Params>>>,
    pub related_to: Option<Vec<Prop<Box<Uid>, Params>>>,
    pub resources: Option<Vec<Prop<Vec<String>, Params>>>,
    pub rdate: Option<Vec<Prop<RDateSeq, Params>>>,
    pub rrule: Option<Vec<Prop<RRule, Params>>>,
    pub image: Option<Vec<Prop<Attachment, Params>>>,
    pub conference: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub styled_description: Option<Vec<Prop<StyledDescriptionValue, Params>>>,
    pub structured_data: Option<Vec<StructuredDataProp>>,

    // Subcomponents
    pub alarms: Vec<Alarm>,
    pub participants: Vec<Participant>,
    pub locations: Vec<LocationComponent>,
    pub resource_components: Vec<ResourceComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// Todo (RFC 5545 §3.6.2)
// ============================================================================

/// A VTODO component (RFC 5545 §3.6.2).
#[structible]
pub struct Todo {
    // Required by RFC 5545, but omitted by many producers
    pub dtstamp: Option<Prop<DateTime<Utc>, Params>>,
    pub uid: Option<Prop<Box<Uid>, Params>>,

    // Optional (at most once)
    pub dtstart: Option<Prop<DateTimeOrDate, Params>>,
    pub class: Option<Prop<Token<ClassValue, String>, Params>>,
    pub completed: Option<Prop<DateTime<Utc>, Params>>,
    pub created: Option<Prop<DateTime<Utc>, Params>>,
    pub description: Option<Prop<String, Params>>,
    pub geo: Option<Prop<Geo, Params>>,
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub location: Option<Prop<String, Params>>,
    pub organizer: Option<Prop<Box<Uri>, Params>>,
    pub percent_complete: Option<Prop<CompletionPercentage, Params>>,
    pub priority: Option<Prop<Priority, Params>>,
    pub recurrence_id: Option<Prop<DateTimeOrDate, Params>>,
    pub sequence: Option<Prop<Integer, Params>>,
    pub status: Option<Prop<Status, Params>>,
    pub summary: Option<Prop<String, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,
    pub due: Option<Prop<DateTimeOrDate, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub color: Option<Prop<Css3Color, Params>>,

    // Multi-valued
    pub attach: Option<Vec<Prop<Attachment, Params>>>,
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub categories: Option<Vec<Prop<Vec<String>, Params>>>,
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub contact: Option<Vec<Prop<String, Params>>>,
    pub exdate: Option<Vec<Prop<DateTimeOrDate, Params>>>,
    pub request_status: Option<Vec<Prop<RequestStatus, Params>>>,
    pub related_to: Option<Vec<Prop<Box<Uid>, Params>>>,
    pub resources: Option<Vec<Prop<Vec<String>, Params>>>,
    pub rdate: Option<Vec<Prop<RDateSeq, Params>>>,
    pub rrule: Option<Vec<Prop<RRule, Params>>>,
    pub image: Option<Vec<Prop<Attachment, Params>>>,
    pub conference: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub styled_description: Option<Vec<Prop<StyledDescriptionValue, Params>>>,
    pub structured_data: Option<Vec<StructuredDataProp>>,

    // Subcomponents
    pub alarms: Vec<Alarm>,
    pub participants: Vec<Participant>,
    pub locations: Vec<LocationComponent>,
    pub resource_components: Vec<ResourceComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// Journal (RFC 5545 §3.6.3)
// ============================================================================

/// A VJOURNAL component (RFC 5545 §3.6.3).
#[structible]
pub struct Journal {
    // Required
    pub dtstamp: Prop<DateTime<Utc>, Params>,
    pub uid: Prop<Box<Uid>, Params>,

    // Optional (at most once)
    pub dtstart: Option<Prop<DateTimeOrDate, Params>>,
    pub class: Option<Prop<Token<ClassValue, String>, Params>>,
    pub created: Option<Prop<DateTime<Utc>, Params>>,
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub organizer: Option<Prop<Box<Uri>, Params>>,
    pub recurrence_id: Option<Prop<DateTimeOrDate, Params>>,
    pub sequence: Option<Prop<Integer, Params>>,
    pub status: Option<Prop<Status, Params>>,
    pub summary: Option<Prop<String, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,

    // Multi-valued
    pub attach: Option<Vec<Prop<Attachment, Params>>>,
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub categories: Option<Vec<Prop<Vec<String>, Params>>>,
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub contact: Option<Vec<Prop<String, Params>>>,
    pub description: Option<Vec<Prop<String, Params>>>,
    pub exdate: Option<Vec<Prop<DateTimeOrDate, Params>>>,
    pub related_to: Option<Vec<Prop<Box<Uid>, Params>>>,
    pub rdate: Option<Vec<Prop<RDateSeq, Params>>>,
    pub rrule: Option<Vec<Prop<RRule, Params>>>,
    pub request_status: Option<Vec<Prop<RequestStatus, Params>>>,

    // Subcomponents
    pub participants: Vec<Participant>,
    pub locations: Vec<LocationComponent>,
    pub resource_components: Vec<ResourceComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// FreeBusy (RFC 5545 §3.6.4)
// ============================================================================

/// A VFREEBUSY component (RFC 5545 §3.6.4).
#[structible]
pub struct FreeBusy {
    // Required
    pub dtstamp: Prop<DateTime<Utc>, Params>,
    pub uid: Prop<Box<Uid>, Params>,

    // Optional (at most once)
    pub contact: Option<Prop<String, Params>>,
    pub dtstart: Option<Prop<DateTimeOrDate, Params>>,
    pub dtend: Option<Prop<DateTimeOrDate, Params>>,
    pub organizer: Option<Prop<Box<Uri>, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,

    // Multi-valued
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub freebusy: Option<Vec<Prop<Vec<Period>, Params>>>,
    pub request_status: Option<Vec<Prop<RequestStatus, Params>>>,

    // Subcomponents
    pub participants: Vec<Participant>,
    pub locations: Vec<LocationComponent>,
    pub resource_components: Vec<ResourceComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// TimeZone (RFC 5545 §3.6.5)
// ============================================================================

/// A VTIMEZONE component (RFC 5545 §3.6.5).
#[structible]
pub struct TimeZone {
    // Required
    pub tz_id: Prop<Box<TzId>, Params>,

    // Optional (at most once)
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub tz_url: Option<Prop<Box<Uri>, Params>>,

    // Subcomponents
    pub rules: Vec<TzRule>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// TzRule (STANDARD / DAYLIGHT)
// ============================================================================

/// A STANDARD or DAYLIGHT subcomponent of a [`TimeZone`].
#[structible]
pub struct TzRule {
    // Required
    pub kind: TzRuleKind,
    pub dtstart: Prop<DateTimeOrDate, Params>,
    pub tz_offset_to: Prop<UtcOffset, Params>,
    pub tz_offset_from: Prop<UtcOffset, Params>,

    // Multi-valued
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub rdate: Option<Vec<Prop<RDateSeq, Params>>>,
    pub rrule: Option<Vec<Prop<RRule, Params>>>,
    pub tz_name: Option<Vec<Prop<String, Params>>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// Alarm (RFC 5545 §3.6.6)
// ============================================================================

/// A VALARM component (RFC 5545 §3.6.6).
#[derive(Debug, Clone, PartialEq)]
pub enum Alarm {
    Audio(AudioAlarm),
    Display(DisplayAlarm),
    Email(EmailAlarm),
    Other(OtherAlarm),
}

/// A VALARM with the AUDIO action.
#[structible]
pub struct AudioAlarm {
    // Required
    pub trigger: Prop<TriggerValue, Params>,

    // Optional (at most once)
    pub attach: Option<Prop<Attachment, Params>>,
    pub uid: Option<Prop<Box<Uid>, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub repeat: Option<Prop<Integer, Params>>,
    pub acknowledged: Option<Prop<DateTime<Utc>, Params>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

/// A VALARM with the DISPLAY action.
#[structible]
pub struct DisplayAlarm {
    // Required
    pub trigger: Prop<TriggerValue, Params>,

    // Required
    pub description: Prop<String, Params>,

    // Optional (at most once)
    pub uid: Option<Prop<Box<Uid>, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub repeat: Option<Prop<Integer, Params>>,
    pub acknowledged: Option<Prop<DateTime<Utc>, Params>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

/// A VALARM with the EMAIL action.
#[structible]
pub struct EmailAlarm {
    // Required
    pub trigger: Prop<TriggerValue, Params>,
    pub description: Prop<String, Params>,
    pub summary: Prop<String, Params>,

    // Optional (at most once)
    pub uid: Option<Prop<Box<Uid>, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub repeat: Option<Prop<Integer, Params>>,
    pub acknowledged: Option<Prop<DateTime<Utc>, Params>>,

    // Multi-valued
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub attach: Option<Vec<Prop<Attachment, Params>>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

/// A VALARM with an action other than AUDIO, DISPLAY, or EMAIL.
#[structible]
pub struct OtherAlarm {
    // Required
    pub trigger: Prop<TriggerValue, Params>,
    pub action: Prop<String, Params>,

    // Optional
    pub description: Option<Prop<String, Params>>,
    pub summary: Option<Prop<String, Params>>,
    pub uid: Option<Prop<Box<Uid>, Params>>,
    pub duration: Option<Prop<SignedDuration, Params>>,
    pub repeat: Option<Prop<Integer, Params>>,
    pub acknowledged: Option<Prop<DateTime<Utc>, Params>>,

    // Multi-valued
    pub attendee: Option<Vec<Prop<Box<Uri>, Params>>>,
    pub attach: Option<Vec<Prop<Attachment, Params>>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// RFC 9073 Components
// ============================================================================

/// A VLOCATION component (RFC 9073 §7.2).
#[structible]
pub struct LocationComponent {
    // Required
    pub uid: Prop<Box<Uid>, Params>,

    // Optional (at most once)
    pub description: Option<Prop<String, Params>>,
    pub geo: Option<Prop<Geo, Params>>,
    pub name: Option<Prop<String, Params>>,
    pub location_type: Option<Prop<String, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,

    // Multi-valued
    pub structured_data: Option<Vec<StructuredDataProp>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

/// A VRESOURCE component (RFC 9073 §7.3).
#[structible]
pub struct ResourceComponent {
    // Required
    pub uid: Prop<Box<Uid>, Params>,

    // Optional (at most once)
    pub description: Option<Prop<String, Params>>,
    pub geo: Option<Prop<Geo, Params>>,
    pub name: Option<Prop<String, Params>>,
    pub resource_type: Option<Prop<Token<ResourceType, String>, Params>>,

    // Multi-valued
    pub structured_data: Option<Vec<StructuredDataProp>>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

/// A PARTICIPANT component (RFC 9073 §7.1).
#[structible]
pub struct Participant {
    // Required
    pub uid: Prop<Box<Uid>, Params>,
    pub participant_type: Prop<Token<ParticipantType, String>, Params>,

    // Optional (at most once)
    pub calendar_address: Option<Prop<Box<Uri>, Params>>,
    pub created: Option<Prop<DateTime<Utc>, Params>>,
    pub description: Option<Prop<String, Params>>,
    pub dtstamp: Option<Prop<DateTime<Utc>, Params>>,
    pub geo: Option<Prop<Geo, Params>>,
    pub last_modified: Option<Prop<DateTime<Utc>, Params>>,
    pub priority: Option<Prop<Priority, Params>>,
    pub sequence: Option<Prop<Integer, Params>>,
    pub status: Option<Prop<Status, Params>>,
    pub summary: Option<Prop<String, Params>>,
    pub url: Option<Prop<Box<Uri>, Params>>,

    // Multi-valued
    pub attach: Option<Vec<Prop<Attachment, Params>>>,
    pub categories: Option<Vec<Prop<Vec<String>, Params>>>,
    pub comment: Option<Vec<Prop<String, Params>>>,
    pub contact: Option<Vec<Prop<String, Params>>>,
    pub location_prop: Option<Vec<Prop<String, Params>>>,
    pub request_status: Option<Vec<Prop<RequestStatus, Params>>>,
    pub related_to: Option<Vec<Prop<Box<Uid>, Params>>>,
    pub resources: Option<Vec<Prop<Vec<String>, Params>>>,
    pub styled_description: Option<Vec<Prop<StyledDescriptionValue, Params>>>,
    pub structured_data: Option<Vec<StructuredDataProp>>,

    // Subcomponents
    pub locations: Vec<LocationComponent>,
    pub resource_components: Vec<ResourceComponent>,

    // Unknown properties
    #[structible(key = Box<CaselessStr>)]
    pub x_property: Option<Vec<Prop<Value<String>, Params>>>,
}

// ============================================================================
// OtherComponent
// ============================================================================

/// An arbitrary component which may have any properties and subcomponents.
#[derive(Debug, Clone, PartialEq)]
pub struct OtherComponent {
    pub name: Box<str>,
    pub subcomponents: Vec<OtherComponent>,
}

// ============================================================================
// Enums kept from original
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentName<S> {
    Calendar,
    Event,
    Todo,
    Journal,
    FreeBusy,
    TimeZone,
    Alarm,
    Standard,
    Daylight,
    Participant,
    Location,
    Resource,
    Unknown(S),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmKind {
    Audio,
    Display,
    Email,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TzRuleKind {
    Standard,
    Daylight,
}

