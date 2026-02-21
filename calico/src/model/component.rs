//! Model types for iCalendar components.

use enumflags2::{BitFlags, bitflags};
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
    pub prod_id: Prop<String, Params>,
    pub version: Prop<Version, Params>,

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
    // Required
    pub dtstamp: Prop<DateTime<Utc>, Params>,
    pub uid: Prop<Box<Uid>, Params>,

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
    // Required
    pub dtstamp: Prop<DateTime<Utc>, Params>,
    pub uid: Prop<Box<Uid>, Params>,

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

// ============================================================================
// Malformed component error types
// ============================================================================

// TODO: these error types currently don't have variants associated with the ordering of time (e.g.
// errors when DTSTART occurs after DTEND); these should be added where appropriate

/// The set of ways in which a `VCALENDAR` component may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedCalendarError {
    /// The `PRODID` property is absent.
    MissingProdId,
    /// The `VERSION` property is absent.
    MissingVersion,
    /// The `PRODID` property occurred more than once.
    DuplicateProdId,
    /// The `VERSION` property occurred more than once.
    DuplicateVersion,
    /// The `CALSCALE` property occurred more than once.
    DuplicateCalScale,
    /// The `METHOD` property occurred more than once.
    DuplicateMethod,
    /// The component has no subcomponents.
    ZeroSubcomponents,
}

// NOTE: "date events" (i.e. VEVENT components whose DTSTART properties have the DATE value type)
// have additional invariants outlined in the second paragraph of RFC 5545 §3.6.1. this should
// probably be reified with a `DateEvent` component newtype

/// The set of ways in which a `VEVENT` component may be invalid.
///
/// # Interpretation of RFC 5545
///
/// Although RFC 5545 never explicitly states that the `DTEND` and `DURATION` properties must occur
/// at most once, we consider this to be the case both because it would otherwise be semantically
/// incoherent to have duplicate ends or durations and because the prose in RFC 5545 §3.6.1 refers
/// to `DTEND` and `DURATION` properties on `VEVENT` in the singular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedEventError {
    /// The `DTSTAMP` property is absent.
    MissingDtStamp,
    /// The `UID` property is absent.
    MissingUid,
    /// The `DTSTAMP` property occurred more than once.
    DuplicateDtStamp,
    /// The `UID` property occurred more than once.
    DuplicateUid,
    /// The `DTSTART` property occurred more than once.
    DuplicateDtStart,
    /// The `CLASS` property occurred more than once.
    DuplicateClass,
    /// The `CREATED` property occurred more than once.
    DuplicateCreated,
    /// The `DESCRIPTION` property occurred more than once.
    DuplicateDescription,
    /// The `GEO` property occurred more than once.
    DuplicateGeo,
    /// The `LAST-MODIFIED` property occurred more than once.
    DuplicateLastModified,
    /// The `LOCATION` property occurred more than once.
    DuplicateLocation,
    /// The `ORGANIZER` property occurred more than once.
    DuplicateOrganizer,
    /// The `PRIORITY` property occurred more than once.
    DuplicatePriority,
    /// The `SEQUENCE` property occurred more than once.
    DuplicateSequence,
    /// The `STATUS` property occurred more than once.
    DuplicateStatus,
    /// The `SUMMARY` property occurred more than once.
    DuplicateSummary,
    /// The `TRANSP` property occurred more than once.
    DuplicateTransp,
    /// The `URL` property occurred more than once.
    DuplicateUrl,
    /// The `RECUR-ID` property occurred more than once.
    DuplicateRecurId,
    /// The `DTEND` property occurred more than once.
    DuplicateDtEnd,
    /// The `DURATION` property occurred more than once.
    DuplicateDuration,
    /// The `DTEND` and `DURATION` properties occurred simultaneously.
    DtEndAndDuration,
    /// The `DTEND` property occurs with a different value type from `DTSTART`.
    DtEndValueTypeMismatch,
    /// The `DTSTART` property has the `DATE` value type and the `DURATION` property occurs with a
    /// value not of the "dur-day" or "dur-week" forms.
    DateEventWithInvalidDuration,
    /// The `STATUS` component occurred with a value which is invalid for `VEVENT` components.
    InvalidStatusValue,
    /// The subcomponents do not occur in the expected order.
    BadlyOrderedSubcomponents,
}

/// The set of ways in which a `VTODO` component may be invalid.
///
/// # Interpretation of RFC 5545
///
/// We require the `DUE` and `DURATION` properties to occur at most once for similar reasons to
/// those outlined for [`VEVENT` components](MalformedEventError#interpretation-of-rfc-5545).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedTodoError {
    /// The `DTSTAMP` property is absent.
    MissingDtStamp,
    /// The `UID` property is absent.
    MissingUid,
    /// The `DTSTAMP` property occurred more than once.
    DuplicateDtStamp,
    /// The `UID` property occurred more than once.
    DuplicateUid,
    /// The `CLASS` property occurred more than once.
    DuplicateClass,
    /// The `COMPLETED` property occurred more than once.
    DuplicateCompleted,
    /// The `CREATED` property occurred more than once.
    DuplicateCreated,
    /// The `DESCRIPTION` property occurred more than once.
    DuplicateDescription,
    /// The `DTSTART` property occurred more than once.
    DuplicateDtStart,
    /// The `GEO` property occurred more than once.
    DuplicateGeo,
    /// The `LAST-MODIFIED` property occurred more than once.
    DuplicateLastModified,
    /// The `LOCATION` property occurred more than once.
    DuplicateLocation,
    /// The `ORGANIZER` property occurred more than once.
    DuplicateOrganizer,
    /// The `PERCENT` property occurred more than once.
    DuplicatePercent,
    /// The `PRIORITY` property occurred more than once.
    DuplicatePriority,
    /// The `RECUR-ID` property occurred more than once.
    DuplicateRecurId,
    /// The `SEQUENCE` property occurred more than once.
    DuplicateSequence,
    /// The `STATUS` property occurred more than once.
    DuplicateStatus,
    /// The `SUMMARY` property occurred more than once.
    DuplicateSummary,
    /// The `URL` property occurred more than once.
    DuplicateUrl,
    /// The `DUE` property occurred more than once.
    DuplicateDue,
    /// The `DURATION` property occurred more than once.
    DuplicateDuration,
    /// The `DUE` and `DURATION` properties occurred simultaneously.
    DueAndDuration,
    /// The `DUE` property occurs with a different value type from `DTSTART` (and implicitly
    /// DTSTART occurs at least once).
    DueValueTypeMismatch,
    /// The `STATUS` component occurred with a value which is invalid for `VTODO` components.
    InvalidStatusValue,
    /// The subcomponents do not occur in the expected order.
    BadlyOrderedSubcomponents,
}

/// The set of ways in which a `VJOURNAL` component may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedJournalError {
    /// The `DTSTAMP` property is absent.
    MissingDtStamp,
    /// The `UID` property is absent.
    MissingUid,
    /// The `DTSTAMP` property occurred more than once.
    DuplicateDtStamp,
    /// The `UID` property occurred more than once.
    DuplicateUid,
    /// The `CLASS` property occurred more than once.
    DuplicateClass,
    /// The `CREATED` property occurred more than once.
    DuplicateCreated,
    /// The `DTSTART` property occurred more than once.
    DuplicateDtStart,
    /// The `LAST-MODIFIED` property occurred more than once.
    DuplicateLastModified,
    /// The `ORGANIZER` property occurred more than once.
    DuplicateOrganizer,
    /// The `RECUR-ID` property occurred more than once.
    DuplicateRecurId,
    /// The `SEQUENCE` property occurred more than once.
    DuplicateSequence,
    /// The `STATUS` property occurred more than once.
    DuplicateStatus,
    /// The `SUMMARY` property occurred more than once.
    DuplicateSummary,
    /// The `URL` property occurred more than once.
    DuplicateUrl,
    /// The `STATUS` component occurred with a value which is invalid for `VJOURNAL` components.
    InvalidStatusValue,
    /// The subcomponents do not occur in the expected order.
    BadlyOrderedSubcomponents,
}

/// The set of ways in which a `VFREEBUSY` component may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedFreeBusyError {
    /// The `DTSTAMP` property is absent.
    MissingDtStamp,
    /// The `UID` property is absent.
    MissingUid,
    /// The `DTSTAMP` property occurred more than once.
    DuplicateDtStamp,
    /// The `UID` property occurred more than once.
    DuplicateUid,
    /// The `CONTACT` property occurred more than once.
    DuplicateContact,
    /// The `DTSTART` property occurred more than once.
    DuplicateDtStart,
    /// The `DTEND` property occurred more than once.
    DuplicateDtEnd,
    /// The `ORGANIZER` property occurred more than once.
    DuplicateOrganizer,
    /// The `URL` property occurred more than once.
    DuplicateUrl,
    /// The subcomponents do not occur in the expected order.
    BadlyOrderedSubcomponents,
}

/// The set of ways in which a `VTIMEZONE` component may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedTimeZoneError {
    /// The `TZID` property is absent.
    MissingTzId,
    /// The `TZID` property occurred more than once.
    DuplicateTzId,
    /// The `LAST-MODIFIED` property occurred more than once.
    DuplicateLastModified,
    /// The `TZURL` property occurred more than once.
    DuplicateTzUrl,
    /// There are zero `STANDARD` or `DAYLIGHT` subcomponents.
    ZeroTzRules,
    /// The subcomponents do not occur in the expected order.
    BadlyOrderedSubcomponents,
}

/// The set of ways in which a `STANDARD` or `DAYLIGHT` component may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedTzRuleError {
    /// The `DTSTART` property is absent.
    MissingDtStart,
    /// The `TZOFFSETTO` property is absent.
    MissingTzOffsetTo,
    /// The `TZOFFSETFROM` property is absent.
    MissingTzOffsetFrom,
    /// The `DTSTART` property occurred more than once.
    DuplicateDtStart,
    /// The `TZOFFSETTO` property occurred more than once.
    DuplicateTzOffsetTo,
    /// The `TZOFFSETFROM` property occurred more than once.
    DuplicateTzOffsetFrom,
    /// The `DTSTART` property occurs with a non-local time value.
    NonLocalDtStart,
}

/// The set of ways in which a `VALARM` component with any action may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedAlarmError {
    /// The `ACTION` property is absent.
    MissingAction,
    /// The `TRIGGER` property is absent.
    MissingTrigger,
    /// The `ACTION` property occurred more than once.
    DuplicateAction,
    /// The `TRIGGER` property occurred more than once.
    DuplicateTrigger,
    /// The `DURATION` property occurred more than once.
    DuplicateDuration,
    /// The `REPEAT` property occurred more than once.
    DuplicateRepeat,
    /// The `DURATION` property occurred without the `REPEAT` property.
    DurationWithoutRepeat,
    /// The `REPEAT` property occurred without the `DURATION` property.
    RepeatWithoutDuration,
}

/// The set of ways in which a `VALARM` component can be invalid according to its action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlarmActionErrors {
    Audio(BitFlags<MalformedAudioAlarmError>),
    Display(BitFlags<MalformedDisplayAlarmError>),
    Email(BitFlags<MalformedEmailAlarmError>),
    Unknown,
}

/// The set of ways in which a `VALARM` component with the `AUDIO` action may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedAudioAlarmError {
    /// The `ATTACH` property occurred more than once.
    DuplicateAttach,
}

/// The set of ways in which a `VALARM` component with the `DISPLAY` action may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedDisplayAlarmError {
    /// The `DESCRIPTION` property is absent.
    MissingDescription,
    /// The `DESCRIPTION` property occurred more than once.
    DuplicateDescription,
}

/// The set of ways in which a `VALARM` component with the `EMAIL` action may be invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[bitflags]
#[repr(u32)]
enum MalformedEmailAlarmError {
    /// The `DESCRIPTION` property is absent.
    MissingDescription,
    /// The `SUMMARY` property is absent.
    MissingSummary,
    /// The `ATTENDEE` property is absent.
    MissingAttendee,
    /// The `DESCRIPTION` property occurred more than once.
    DuplicateDescription,
    /// The `SUMMARY` property occurred more than once.
    DuplicateSummary,
}

// TODO: write Malformed*Error types and add corresponding variants for:
// - Participant (RFC 9073)
// - Location (RFC 9073)
// - Resource (RFC 9073)
