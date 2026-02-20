//! Distinguished object types.

use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    hash::Hash,
    num::NonZero,
};

use structible::structible;
use thiserror::Error;

use crate::parser::{local_date_time, parse_full};
use crate::{
    json::{
        ConstructibleJsonValue, DestructibleJsonValue, DocumentError, IntoJson, Int,
        IntoDocumentError, JsonArray, JsonObject, JsonValue, PathSegment, TryFromJson, TypeError,
        TypeErrorOr, UnsignedInt,
    },
    model::{
        request_status::{RequestStatus, StatusCode},
        rrule::RRule,
        set::{
            AlertAction, AlertRelativeTo, Color, DisplayPurpose, EventStatus, FreeBusyStatus,
            LinkRelation, LocationType, Method, ParticipantKind, ParticipantRole,
            ParticipationStatus, Percent, Priority, Privacy, RelationValue, ScheduleAgent,
            TaskProgress, VirtualLocationFeature,
        },
        string::{
            AlphaNumeric, CalAddress, ContentId, CustomTimeZoneId, EmailAddr, GeoUri, Id,
            ImplicitJsonPointer, InvalidImplicitJsonPointerError, LanguageTag, MediaType, Uid, Uri,
        },
        time::{
            Date, DateTime, Day, Duration, Hour, IsoWeek, Local, Minute, Month, NonLeapSecond,
            Sign, SignedDuration, Utc, UtcOffset, Weekday, Year,
        },
    },
};
use rfc5545_types::rrule::weekday_num_set::WeekdayNumSet;
use rfc5545_types::time::DateTimeOrDate;

type Token<T> = super::set::Token<T, Box<str>>;

/// A JSCalendar group opject (RFC 8984 §2.3).
///
/// A group is a collection of [`Event`] and [`Task`] objects. Typically, objects are grouped by
/// topic (e.g. by keywords) or calendar membership.
#[structible]
pub struct Group<V: JsonValue> {
    // Group Properties (RFC 8984 §5.3)
    pub entries: Vec<TaskOrEvent<V>>,
    pub source: Option<Box<Uri>>,

    // Common Properties (RFC 8984 §4)
    pub uid: Box<Uid>,
    pub prod_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,
    pub locale: Option<LanguageTag>,
    pub keywords: Option<HashSet<String>>,
    pub categories: Option<HashSet<String>>,
    pub color: Option<Color>,
    pub time_zones: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>>,

    // Custom vendor properties (RFC 8984 §3.3)
    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A [`Task`] or an [`Event`].
#[non_exhaustive]
pub enum TaskOrEvent<V: JsonValue> {
    Task(Task<V>),
    Event(Event<V>),
}

impl<V> PartialEq for TaskOrEvent<V>
where
    V: JsonValue + PartialEq,
    V::Object: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Task(l0), Self::Task(r0)) => l0 == r0,
            (Self::Event(l0), Self::Event(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<V> Clone for TaskOrEvent<V>
where
    V: JsonValue + Clone,
    V::Object: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Task(arg0) => Self::Task(arg0.clone()),
            Self::Event(arg0) => Self::Event(arg0.clone()),
        }
    }
}

impl<V> std::fmt::Debug for TaskOrEvent<V>
where
    V: JsonValue + std::fmt::Debug,
    V::Object: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Task(arg0) => f.debug_tuple("Task").field(arg0).finish(),
            Self::Event(arg0) => f.debug_tuple("Event").field(arg0).finish(),
        }
    }
}

impl<V: JsonValue> TaskOrEvent<V> {
    pub const fn as_task(&self) -> Option<&Task<V>> {
        if let Self::Task(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn as_event(&self) -> Option<&Event<V>> {
        if let Self::Event(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// A JSCalendar event object (RFC 8984 §2.1).
///
/// An event represents a scheduled amount of time on a calendar, typically a meeting, appointment,
/// reminder, or anniversary. It is required to start at a certain point in time and typically has
/// a non-zero duration. Multiple participants may partake in the event at multiple locations.
#[structible]
pub struct Event<V: JsonValue> {
    // Event Properties (RFC 8984 §5.1)
    pub start: DateTime<Local>,
    pub duration: Option<Duration>,
    pub status: Option<Token<EventStatus>>,

    // Metadata Properties (RFC 8984 §4.1)
    pub uid: Box<Uid>,
    pub related_to: Option<HashMap<Box<Uid>, Relation<V>>>,
    pub prod_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub sequence: Option<UnsignedInt>,
    pub method: Option<Token<Method>>,

    // What and Where Properties (RFC 8984 §4.2)
    pub title: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub show_without_time: Option<bool>,
    pub locations: Option<HashMap<Box<Id>, Location<V>>>,
    pub virtual_locations: Option<HashMap<Box<Id>, VirtualLocation<V>>>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,
    pub locale: Option<LanguageTag>,
    pub keywords: Option<HashSet<String>>,
    pub categories: Option<HashSet<String>>,
    pub color: Option<Color>,

    // Recurrence Properties (RFC 8984 §4.3)
    pub recurrence_id: Option<DateTime<Local>>,
    pub recurrence_id_time_zone: Option<String>,
    pub recurrence_rules: Option<Vec<RRule>>,
    pub excluded_recurrence_rules: Option<Vec<RRule>>,
    pub recurrence_overrides: Option<HashMap<DateTime<Local>, PatchObject<V>>>,
    pub excluded: Option<bool>,

    // Sharing and Scheduling Properties (RFC 8984 §4.4)
    pub priority: Option<Priority>,
    pub free_busy_status: Option<Token<FreeBusyStatus>>,
    pub privacy: Option<Token<Privacy>>,
    pub reply_to: Option<ReplyTo>,
    pub sent_by: Option<Box<CalAddress>>,
    pub participants: Option<HashMap<Box<Id>, Participant<V>>>,
    pub request_status: Option<RequestStatus>,

    // Alerts Properties (RFC 8984 §4.5)
    pub use_default_alerts: Option<bool>,
    pub alerts: Option<HashMap<Box<Id>, Alert<V>>>,

    // Multilingual Properties (RFC 8984 §4.6)
    pub localizations: Option<HashMap<LanguageTag, PatchObject<V>>>,

    // Time Zone Properties (RFC 8984 §4.7)
    pub time_zone: Option<String>,
    pub time_zones: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>>,

    // Custom vendor properties (RFC 8984 §3.3)
    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A JSCalendar task object (RFC 8984 §2.2).
///
/// A task represents an action item, assignment, to-do item, or work item. It may start and be due
/// at certain points in time, take some estimated time to complete, and recur, none of which is
/// required.
#[structible]
pub struct Task<V: JsonValue> {
    // Task Properties (RFC 8984 §5.2)
    pub due: Option<DateTime<Local>>,
    pub start: Option<DateTime<Local>>,
    pub estimated_duration: Option<Duration>,
    pub percent_complete: Option<Percent>,
    pub progress: Option<Token<TaskProgress>>,
    pub progress_updated: Option<DateTime<Utc>>,

    // Metadata Properties (RFC 8984 §4.1)
    pub uid: Box<Uid>,
    pub related_to: Option<HashMap<Box<Uid>, Relation<V>>>,
    pub prod_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub sequence: Option<UnsignedInt>,
    pub method: Option<Token<Method>>,

    // What and Where Properties (RFC 8984 §4.2)
    pub title: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub show_without_time: Option<bool>,
    pub locations: Option<HashMap<Box<Id>, Location<V>>>,
    pub virtual_locations: Option<HashMap<Box<Id>, VirtualLocation<V>>>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,
    pub locale: Option<LanguageTag>,
    pub keywords: Option<HashSet<String>>,
    pub categories: Option<HashSet<String>>,
    pub color: Option<Color>,

    // Recurrence Properties (RFC 8984 §4.3)
    pub recurrence_id: Option<DateTime<Local>>,
    pub recurrence_id_time_zone: Option<String>,
    pub recurrence_rules: Option<Vec<RRule>>,
    pub excluded_recurrence_rules: Option<Vec<RRule>>,
    pub recurrence_overrides: Option<HashMap<DateTime<Local>, PatchObject<V>>>,
    pub excluded: Option<bool>,

    // Sharing and Scheduling Properties (RFC 8984 §4.4)
    pub priority: Option<Priority>,
    pub free_busy_status: Option<Token<FreeBusyStatus>>,
    pub privacy: Option<Token<Privacy>>,
    pub reply_to: Option<ReplyTo>,
    pub sent_by: Option<Box<CalAddress>>,
    pub participants: Option<HashMap<Box<Id>, TaskParticipant<V>>>,
    pub request_status: Option<RequestStatus>,

    // Alerts Properties (RFC 8984 §4.5)
    pub use_default_alerts: Option<bool>,
    pub alerts: Option<HashMap<Box<Id>, Alert<V>>>,

    // Multilingual Properties (RFC 8984 §4.6)
    pub localizations: Option<HashMap<LanguageTag, PatchObject<V>>>,

    // Time Zone Properties (RFC 8984 §4.7)
    pub time_zone: Option<String>,
    pub time_zones: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>>,

    // Custom vendor properties (RFC 8984 §3.3)
    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A description of a physical location (RFC 8984 §4.2.5).
#[structible]
pub struct Location<V> {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_types: Option<HashSet<LocationType>>,
    pub relative_to: Option<Token<RelationValue>>,
    pub time_zone: Option<String>,
    pub coordinates: Option<Box<GeoUri>>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A description of a virtual location (RFC 8984 §4.2.6).
#[structible]
pub struct VirtualLocation<V> {
    pub name: Option<String>,
    pub description: Option<String>,
    pub uri: Box<Uri>,
    pub features: Option<HashSet<Token<VirtualLocationFeature>>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A link to an external resource (RFC 8984 §1.4.11).
#[structible]
pub struct Link<V> {
    pub href: Box<Uri>,
    pub content_id: Option<Box<ContentId>>,
    pub media_type: Option<Box<MediaType>>,
    pub size: Option<UnsignedInt>,
    pub relation: Option<LinkRelation>,
    pub display: Option<Token<DisplayPurpose>>,
    pub title: Option<String>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A description of a time zone (RFC 8984 §4.7.2).
#[structible]
pub struct TimeZone<V> {
    pub tz_id: String,
    pub updated: Option<DateTime<Utc>>,
    pub url: Option<Box<Uri>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub aliases: Option<HashSet<Box<str>>>,
    pub standard: Option<Vec<TimeZoneRule<V>>>,
    pub daylight: Option<Vec<TimeZoneRule<V>>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A rule belonging to a [`TimeZone`], which may describe a period of either standard or daylight
/// savings time (RFC 8984 §4.7.2).
#[structible]
pub struct TimeZoneRule<V> {
    pub start: DateTime<Local>,
    pub offset_from: UtcOffset,
    pub offset_to: UtcOffset,
    pub recurrence_rules: Option<Vec<RRule>>,
    pub recurrence_overrides: Option<HashMap<DateTime<Local>, PatchObject<V>>>,
    pub names: Option<HashSet<String>>,
    pub comments: Option<Vec<String>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A description of a participant (RFC 8984 §4.4.6).
#[structible]
pub struct Participant<V> {
    pub name: Option<String>,
    pub email: Option<Box<EmailAddr>>,
    pub description: Option<String>,
    pub send_to: Option<SendToParticipant>,
    pub kind: Option<Token<ParticipantKind>>,
    pub roles: Option<HashSet<Token<ParticipantRole>>>, // this could be a bitset
    pub location_id: Option<Box<Id>>,
    pub language: Option<LanguageTag>,
    pub participation_status: Option<Token<ParticipationStatus>>,
    pub participation_comment: Option<String>,
    pub expect_reply: Option<bool>,
    pub schedule_agent: Option<Token<ScheduleAgent>>,
    pub schedule_force_send: Option<bool>,
    pub schedule_sequence: Option<UnsignedInt>,
    pub schedule_status: Option<Vec<StatusCode>>,
    pub schedule_updated: Option<DateTime<Utc>>,
    pub sent_by: Option<Box<EmailAddr>>,
    pub invited_by: Option<Box<Id>>,
    pub delegated_to: Option<HashSet<Box<Id>>>,
    pub delegated_from: Option<HashSet<Box<Id>>>,
    pub member_of: Option<HashSet<Box<Id>>>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A description of a participant which may occur in a [`Task`] (RFC 8984 §4.4.6).
#[structible]
pub struct TaskParticipant<V> {
    // general participant fields
    pub name: Option<String>,
    pub email: Option<Box<EmailAddr>>,
    pub description: Option<String>,
    pub send_to: Option<SendToParticipant>,
    pub kind: Option<Token<ParticipantKind>>,
    pub roles: Option<HashSet<Token<ParticipantRole>>>, // this could be a bitset
    pub location_id: Option<Box<Id>>,
    pub language: Option<LanguageTag>,
    pub participation_status: Option<Token<ParticipationStatus>>,
    pub participation_comment: Option<String>,
    pub expect_reply: Option<bool>,
    pub schedule_agent: Option<Token<ScheduleAgent>>,
    pub schedule_force_send: Option<bool>,
    pub schedule_sequence: Option<UnsignedInt>,
    pub schedule_status: Option<Vec<StatusCode>>,
    pub schedule_updated: Option<DateTime<Utc>>,
    pub sent_by: Option<Box<EmailAddr>>,
    pub invited_by: Option<Box<Id>>,
    pub delegated_to: Option<HashSet<Box<Id>>>,
    pub delegated_from: Option<HashSet<Box<Id>>>,
    pub member_of: Option<HashSet<Box<Id>>>,
    pub links: Option<HashMap<Box<Id>, Link<V>>>,

    // task-specific fields
    pub progress: Option<Token<TaskProgress>>,
    pub progress_updated: Option<DateTime<Utc>>,
    pub percent_complete: Option<Percent>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

// TODO: define an HttpsUrl newtype for URIs that are statically known to start with the https:
// scheme, which should then be used for the type of ReplyTo::web

/// The type of the `replyTo` property (RFC 8984 §4.4.4).
#[structible]
pub struct ReplyTo {
    /// If the `imip` field is defined, then the organizer accepts an iMIP (RFC 6047) response at
    /// the corresponding email address.
    pub imip: Option<Box<CalAddress>>,
    /// If the `web` field is defined, then opening the corresponding [`Uri`] in a web browser will
    /// provide the user with a page where they can submit a reply to the organizer.
    pub web: Option<Box<Uri>>,
    /// If any other `replyTo` method is present, the organizer is considered to be identified by
    /// the corresponding [`Uri`], but the method for submitting the response is undefined. This
    /// includes vendor-prefixed method names.
    #[structible(key = Box<AlphaNumeric>)]
    pub other: Option<Box<Uri>>,
}

/// The type of the `sendTo` property on [`Participant`] (RFC 8984 §4.4.6).
#[structible]
pub struct SendToParticipant {
    /// If the `imip` field is defined, then the participant accepts an iMIP (RFC 6047) request at
    /// the corresponding email address. The email address may be different from the [`email`]
    /// property on the [`Participant`].
    ///
    /// [`email`]: Participant::email
    pub imip: Option<Box<CalAddress>>,
    /// If any other `sendTo` method is present, the participant is considered to be identified by
    /// the corresponding [`Uri`], but the method for submitting invitations and updates is
    /// undefined. This includes vendor-prefixed method names.
    #[structible(key = Box<AlphaNumeric>)]
    pub other: Option<Box<Uri>>,
}

/// A representation of an alert or a reminder (RFC 8984 §4.5.2).
#[structible]
pub struct Alert<V: JsonValue> {
    pub trigger: Trigger<V>,
    pub acknowledged: Option<DateTime<Utc>>,
    pub related_to: Option<HashMap<Box<str>, Relation<V>>>,
    pub action: Option<Token<AlertAction>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// The trigger of an [`Alert`].
#[derive(PartialEq)]
#[non_exhaustive]
pub enum Trigger<V: JsonValue> {
    Offset(OffsetTrigger<V>),
    Absolute(AbsoluteTrigger<V>),
    Unknown(V::Object),
}

impl<V> Clone for Trigger<V>
where
    V: JsonValue + Clone,
    V::Object: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Offset(arg0) => Self::Offset(arg0.clone()),
            Self::Absolute(arg0) => Self::Absolute(arg0.clone()),
            Self::Unknown(arg0) => Self::Unknown(arg0.clone()),
        }
    }
}

impl<V> std::fmt::Debug for Trigger<V>
where
    V: JsonValue + std::fmt::Debug,
    V::Object: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offset(arg0) => f.debug_tuple("Offset").field(arg0).finish(),
            Self::Absolute(arg0) => f.debug_tuple("Absolute").field(arg0).finish(),
            Self::Unknown(arg0) => f.debug_tuple("Unknown").field(arg0).finish(),
        }
    }
}

/// A trigger defined relative to a time property (RFC 8984 §4.5.2).
#[structible]
pub struct OffsetTrigger<V> {
    pub offset: SignedDuration,
    pub relative_to: Option<Token<AlertRelativeTo>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A trigger defined at an absolute time (RFC 8984 §4.5.2).
#[structible]
pub struct AbsoluteTrigger<V> {
    pub when: DateTime<Utc>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A set of relationship types (RFC 8984 §1.4.10).
#[structible]
pub struct Relation<V> {
    pub relations: HashSet<Token<RelationValue>>,

    #[structible(key = Box<str>)]
    pub vendor_property: Option<V>,
}

/// A set of patches to be applied to a JSON object (RFC 8984 §1.4.9).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchObject<V>(HashMap<Box<ImplicitJsonPointer>, V>);

impl<V> PatchObject<V> {
    /// Returns a reference to the value for the given pointer, if present.
    pub fn get(&self, key: &ImplicitJsonPointer) -> Option<&V> {
        self.0.get(key)
    }

    /// Returns the number of patches.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no patches.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over all (pointer, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ImplicitJsonPointer, &V)> {
        self.0.iter().map(|(k, v)| (&**k, v))
    }

    /// Consumes the `PatchObject` and returns the underlying map.
    pub fn into_inner(self) -> HashMap<Box<ImplicitJsonPointer>, V> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("the key {key} is not an implicit JSON pointer")]
pub struct InvalidPatchObjectError {
    key: Box<str>,
    error: InvalidImplicitJsonPointerError,
}

impl IntoDocumentError for InvalidPatchObjectError {
    type Residual = InvalidImplicitJsonPointerError;

    fn into_document_error(self) -> DocumentError<Self::Residual> {
        let mut path = VecDeque::with_capacity(1);
        path.push_front(PathSegment::String(self.key));

        DocumentError {
            path,
            error: self.error,
        }
    }
}

impl<V: DestructibleJsonValue> TryFromJson<V> for PatchObject<V> {
    type Error = TypeErrorOr<InvalidPatchObjectError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        value
            .try_into_object()?
            .into_iter()
            .map(|(key, value)| {
                let k = <V as JsonValue>::Object::key_into_string(key);

                match ImplicitJsonPointer::new(&k) {
                    Ok(ptr) => Ok((ptr.into(), value)),
                    Err(error) => Err(InvalidPatchObjectError {
                        key: k.into_boxed_str(),
                        error,
                    }),
                }
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .map(PatchObject)
            .map_err(TypeErrorOr::Other)
    }
}

// ============================================================================
// Error type and helpers for object parsing
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ObjectFromJsonError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("{0}")]
    InvalidFieldValue(Box<str>),
}

type ObjErr = DocumentError<TypeErrorOr<ObjectFromJsonError>>;

fn field_err<E: std::fmt::Display>(field: &'static str, e: TypeErrorOr<E>) -> ObjErr {
    let err = match e {
        TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
        TypeErrorOr::Other(e) => TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
            e.to_string().into_boxed_str(),
        )),
    };
    DocumentError {
        path: [PathSegment::Static(field)].into(),
        error: err,
    }
}

fn type_field_err(field: &'static str, e: TypeError) -> ObjErr {
    DocumentError {
        path: [PathSegment::Static(field)].into(),
        error: TypeErrorOr::TypeError(e),
    }
}

fn doc_field_err<E: std::fmt::Display>(
    field: &'static str,
    mut e: DocumentError<TypeErrorOr<E>>,
) -> ObjErr {
    let err = match e.error {
        TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
        TypeErrorOr::Other(e) => TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
            e.to_string().into_boxed_str(),
        )),
    };
    e.path.push_front(PathSegment::Static(field));
    DocumentError {
        path: e.path,
        error: err,
    }
}

fn prepend(field: &'static str, mut e: ObjErr) -> ObjErr {
    e.path.push_front(PathSegment::Static(field));
    e
}

fn missing(field: &'static str) -> ObjErr {
    DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::MissingField(field)))
}

// ============================================================================
// UtcOffset TryFromJson
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid UTC offset string: {0:?}")]
pub struct InvalidUtcOffsetError(pub Box<str>);

impl<V: DestructibleJsonValue> TryFromJson<V> for UtcOffset {
    type Error = TypeErrorOr<InvalidUtcOffsetError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let s = value.try_into_string()?;
        parse_utc_offset(s.as_ref()).ok_or_else(|| {
            TypeErrorOr::Other(InvalidUtcOffsetError(
                String::from(s.as_ref()).into_boxed_str(),
            ))
        })
    }
}

fn parse_utc_offset(s: &str) -> Option<UtcOffset> {
    let (sign, rest) = match s.as_bytes().first() {
        Some(b'+') => (Sign::Pos, &s[1..]),
        Some(b'-') => (Sign::Neg, &s[1..]),
        _ => return None,
    };
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let hh: u8 = parts[0].parse().ok()?;
    let mm: u8 = parts[1].parse().ok()?;
    let ss: u8 = if parts.len() == 3 {
        parts[2].parse().ok()?
    } else {
        0
    };
    Some(UtcOffset {
        sign,
        hour: Hour::new(hh).ok()?,
        minute: Minute::new(mm).ok()?,
        second: NonLeapSecond::new(ss).ok()?,
    })
}

// ============================================================================
// StatusCode TryFromJson
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid status code string: {0:?}")]
pub struct InvalidStatusCodeError(pub Box<str>);

impl<V: DestructibleJsonValue> TryFromJson<V> for StatusCode {
    type Error = TypeErrorOr<InvalidStatusCodeError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let s = value.try_into_string()?;
        parse_status_code(s.as_ref()).ok_or_else(|| {
            TypeErrorOr::Other(InvalidStatusCodeError(
                String::from(s.as_ref()).into_boxed_str(),
            ))
        })
    }
}

fn parse_status_code(s: &str) -> Option<StatusCode> {
    use crate::model::request_status::Class;
    let mut parts = s.splitn(3, '.');
    let class_n: u8 = parts.next()?.parse().ok()?;
    let class = match class_n {
        1 => Class::C1,
        2 => Class::C2,
        3 => Class::C3,
        4 => Class::C4,
        5 => Class::C5,
        _ => return None,
    };
    let major: u8 = parts.next()?.parse().ok()?;
    let minor: Option<u8> = match parts.next() {
        Some(s) => Some(s.parse().ok()?),
        None => None,
    };
    Some(StatusCode {
        class,
        major,
        minor,
    })
}

// ============================================================================
// RequestStatus TryFromJson
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid request status string: {0:?}")]
pub struct InvalidRequestStatusError(pub Box<str>);

impl<V: DestructibleJsonValue> TryFromJson<V> for RequestStatus {
    type Error = TypeErrorOr<InvalidRequestStatusError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let s = value.try_into_string()?;
        parse_request_status(s.as_ref()).ok_or_else(|| {
            TypeErrorOr::Other(InvalidRequestStatusError(
                String::from(s.as_ref()).into_boxed_str(),
            ))
        })
    }
}

fn parse_request_status(s: &str) -> Option<RequestStatus> {
    let mut parts = s.splitn(3, ';');
    let code_str = parts.next()?;
    let code = parse_status_code(code_str)?;
    let description: Box<str> = parts.next()?.into();
    let exception_data: Option<Box<str>> = parts.next().map(Into::into);
    Some(RequestStatus {
        code,
        description,
        exception_data,
    })
}

// ============================================================================
// RRule TryFromJson
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum RRuleFromJsonError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid field value: {0}")]
    InvalidValue(Box<str>),
}

impl<V: DestructibleJsonValue> TryFromJson<V> for RRule {
    type Error = DocumentError<TypeErrorOr<RRuleFromJsonError>>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        fn rrule_field_err<E: std::fmt::Display>(
            field: &'static str,
            e: TypeErrorOr<E>,
        ) -> DocumentError<TypeErrorOr<RRuleFromJsonError>> {
            let err = match e {
                TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
                TypeErrorOr::Other(e) => TypeErrorOr::Other(RRuleFromJsonError::InvalidValue(
                    e.to_string().into_boxed_str(),
                )),
            };
            DocumentError {
                path: [PathSegment::Static(field)].into(),
                error: err,
            }
        }
        fn rrule_invalid(
            field: &'static str,
            msg: &str,
        ) -> DocumentError<TypeErrorOr<RRuleFromJsonError>> {
            DocumentError {
                path: [PathSegment::Static(field)].into(),
                error: TypeErrorOr::Other(RRuleFromJsonError::InvalidValue(msg.into())),
            }
        }
        fn rrule_missing(field: &'static str) -> DocumentError<TypeErrorOr<RRuleFromJsonError>> {
            DocumentError::root(TypeErrorOr::Other(RRuleFromJsonError::MissingField(field)))
        }

        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        // Collect raw JSON values for each field
        let mut frequency_val: Option<V> = None;
        let mut interval_val: Option<V> = None;
        let mut count_val: Option<V> = None;
        let mut until_val: Option<V> = None;
        let mut week_start_val: Option<V> = None;
        let mut by_day_val: Option<V> = None;
        let mut by_hour_val: Option<V> = None;
        let mut by_minute_val: Option<V> = None;
        let mut by_second_val: Option<V> = None;
        let mut by_month_val: Option<V> = None;
        let mut by_set_pos_val: Option<V> = None;
        let mut by_month_day_val: Option<V> = None;
        let mut by_year_day_val: Option<V> = None;
        let mut by_week_no_val: Option<V> = None;

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" | "rscale" | "skip" => {}
                "frequency" => frequency_val = Some(val),
                "interval" => interval_val = Some(val),
                "count" => count_val = Some(val),
                "until" => until_val = Some(val),
                "firstDayOfWeek" => week_start_val = Some(val),
                "byDay" => by_day_val = Some(val),
                "byHour" => by_hour_val = Some(val),
                "byMinute" => by_minute_val = Some(val),
                "bySecond" => by_second_val = Some(val),
                "byMonth" => by_month_val = Some(val),
                "bySetPosition" => by_set_pos_val = Some(val),
                "byMonthDay" => by_month_day_val = Some(val),
                "byYearDay" => by_year_day_val = Some(val),
                "byWeekNo" => by_week_no_val = Some(val),
                _ => {}
            }
        }

        // Parse frequency (required)
        let freq_str = frequency_val
            .ok_or_else(|| rrule_missing("frequency"))?
            .try_into_string()
            .map_err(|e| {
                rrule_field_err::<std::convert::Infallible>("frequency", TypeErrorOr::TypeError(e))
            })?;

        // Parse interval
        let interval = match interval_val {
            None => None,
            Some(v) => {
                let n =
                    UnsignedInt::try_from_json(v).map_err(|e| rrule_field_err("interval", e))?;
                let nz = NonZero::new(n.get())
                    .ok_or_else(|| rrule_invalid("interval", "interval must be >= 1"))?;
                Some(crate::model::rrule::Interval::new(nz))
            }
        };

        // Parse termination (count or until, mutually exclusive)
        let termination = match (count_val, until_val) {
            (Some(c), None) => {
                let n = UnsignedInt::try_from_json(c).map_err(|e| rrule_field_err("count", e))?;
                Some(crate::model::rrule::Termination::Count(n.get()))
            }
            (None, Some(u)) => {
                let s = u.try_into_string().map_err(|e| {
                    rrule_field_err::<std::convert::Infallible>("until", TypeErrorOr::TypeError(e))
                })?;
                let until = parse_date_or_datetime(s.as_ref())
                    .ok_or_else(|| rrule_invalid("until", s.as_ref()))?;
                Some(crate::model::rrule::Termination::Until(until))
            }
            (None, None) => None,
            (Some(_), Some(_)) => {
                return Err(rrule_invalid(
                    "count",
                    "count and until are mutually exclusive",
                ));
            }
        };

        // Parse firstDayOfWeek
        let week_start = match week_start_val {
            None => None,
            Some(v) => {
                let s = v.try_into_string().map_err(|e| {
                    rrule_field_err::<std::convert::Infallible>(
                        "firstDayOfWeek",
                        TypeErrorOr::TypeError(e),
                    )
                })?;
                let wd = parse_weekday_code(s.as_ref())
                    .ok_or_else(|| rrule_invalid("firstDayOfWeek", s.as_ref()))?;
                Some(wd)
            }
        };

        // Parse byDay → WeekdayNumSet
        let by_day = match by_day_val {
            None => None,
            Some(v) => Some(parse_by_day::<V>(v).map_err(|e| {
                let error = match e.error {
                    TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
                    TypeErrorOr::Other(br) => TypeErrorOr::Other(RRuleFromJsonError::InvalidValue(
                        br.to_string().into_boxed_str(),
                    )),
                };
                let mut path = e.path;
                path.push_front(PathSegment::Static("byDay"));
                DocumentError { path, error }
            })?),
        };

        // Parse byHour → HourSet
        let by_hour = match by_hour_val {
            None => None,
            Some(v) => Some(parse_by_hour::<V>(v).map_err(|e| rrule_field_err("byHour", e))?),
        };

        // Parse byMinute → MinuteSet
        let by_minute = match by_minute_val {
            None => None,
            Some(v) => Some(parse_by_minute::<V>(v).map_err(|e| rrule_field_err("byMinute", e))?),
        };

        // Parse bySecond → SecondSet
        let by_second = match by_second_val {
            None => None,
            Some(v) => Some(parse_by_second::<V>(v).map_err(|e| rrule_field_err("bySecond", e))?),
        };

        // Parse byMonth → MonthSet
        let by_month = match by_month_val {
            None => None,
            Some(v) => Some(parse_by_month::<V>(v).map_err(|e| rrule_field_err("byMonth", e))?),
        };

        // Parse bySetPosition → BTreeSet<YearDayNum>
        let by_set_pos = match by_set_pos_val {
            None => None,
            Some(v) => {
                Some(parse_year_day_nums::<V>(v).map_err(|e| rrule_field_err("bySetPosition", e))?)
            }
        };

        // Parse byMonthDay → MonthDaySet
        let by_month_day = match by_month_day_val {
            None => None,
            Some(v) => {
                Some(parse_by_month_day::<V>(v).map_err(|e| rrule_field_err("byMonthDay", e))?)
            }
        };

        // Parse byYearDay → BTreeSet<YearDayNum>
        let by_year_day = match by_year_day_val {
            None => None,
            Some(v) => {
                Some(parse_year_day_nums::<V>(v).map_err(|e| rrule_field_err("byYearDay", e))?)
            }
        };

        // Parse byWeekNo → WeekNoSet
        let by_week_no = match by_week_no_val {
            None => None,
            Some(v) => Some(parse_by_week_no::<V>(v).map_err(|e| rrule_field_err("byWeekNo", e))?),
        };

        // Build CoreByRules
        let core_by_rules = crate::model::rrule::CoreByRules {
            by_second,
            by_minute,
            by_hour,
            by_month,
            by_day,
            by_set_pos,
        };

        // Build FreqByRules based on frequency string
        let freq = match freq_str.as_ref().to_lowercase().as_str() {
            "secondly" => {
                crate::model::rrule::FreqByRules::Secondly(crate::model::rrule::ByPeriodDayRules {
                    by_month_day,
                    by_year_day,
                })
            }
            "minutely" => {
                crate::model::rrule::FreqByRules::Minutely(crate::model::rrule::ByPeriodDayRules {
                    by_month_day,
                    by_year_day,
                })
            }
            "hourly" => {
                crate::model::rrule::FreqByRules::Hourly(crate::model::rrule::ByPeriodDayRules {
                    by_month_day,
                    by_year_day,
                })
            }
            "daily" => {
                crate::model::rrule::FreqByRules::Daily(crate::model::rrule::ByMonthDayRule {
                    by_month_day,
                })
            }
            "weekly" => crate::model::rrule::FreqByRules::Weekly,
            "monthly" => {
                crate::model::rrule::FreqByRules::Monthly(crate::model::rrule::ByMonthDayRule {
                    by_month_day,
                })
            }
            "yearly" => {
                crate::model::rrule::FreqByRules::Yearly(crate::model::rrule::YearlyByRules {
                    by_month_day,
                    by_year_day,
                    by_week_no,
                })
            }
            _ => {
                return Err(rrule_invalid("frequency", freq_str.as_ref()));
            }
        };

        Ok(RRule {
            freq,
            core_by_rules,
            interval,
            termination,
            week_start,
        })
    }
}

fn parse_weekday_code(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "mo" => Some(Weekday::Monday),
        "tu" => Some(Weekday::Tuesday),
        "we" => Some(Weekday::Wednesday),
        "th" => Some(Weekday::Thursday),
        "fr" => Some(Weekday::Friday),
        "sa" => Some(Weekday::Saturday),
        "su" => Some(Weekday::Sunday),
        _ => None,
    }
}

fn parse_date_or_datetime(s: &str) -> Option<DateTimeOrDate<crate::model::time::Local>> {
    if let Ok(dt) = parse_full(local_date_time)(s) {
        return Some(DateTimeOrDate::DateTime(dt));
    }
    // Try date-only: YYYY-MM-DD
    if s.len() == 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-') {
        let year: u16 = s[0..4].parse().ok()?;
        let month: u8 = s[5..7].parse().ok()?;
        let day: u8 = s[8..10].parse().ok()?;
        let date = Date::new(
            Year::new(year).ok()?,
            Month::new(month).ok()?,
            Day::new(day).ok()?,
        )
        .ok()?;
        return Some(DateTimeOrDate::Date(date));
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ByRuleParseError {
    #[error("invalid value in by-rule array")]
    InvalidValue,
}

fn parse_by_day<V: DestructibleJsonValue>(
    val: V,
) -> Result<WeekdayNumSet, DocumentError<TypeErrorOr<ByRuleParseError>>> {
    let arr = val
        .try_into_array()
        .map_err(TypeErrorOr::from)
        .map_err(DocumentError::root)?;
    let mut set = WeekdayNumSet::with_capacity(0);
    for (i, elem) in arr.into_iter().enumerate() {
        let obj = elem.try_into_object().map_err(|e| DocumentError {
            path: [PathSegment::Index(i)].into(),
            error: TypeErrorOr::TypeError(e),
        })?;
        let mut day_val: Option<Weekday> = None;
        let mut nth_val: Option<i64> = None;
        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "day" => {
                    let s = val.try_into_string().map_err(|e| DocumentError {
                        path: [PathSegment::Index(i), PathSegment::Static("day")].into(),
                        error: TypeErrorOr::TypeError(e),
                    })?;
                    day_val =
                        Some(parse_weekday_code(s.as_ref()).ok_or_else(|| DocumentError {
                            path: [PathSegment::Index(i), PathSegment::Static("day")].into(),
                            error: TypeErrorOr::Other(ByRuleParseError::InvalidValue),
                        })?);
                }
                "nthOfPeriod" => {
                    let n = Int::try_from_json(val).map_err(|e| DocumentError {
                        path: [PathSegment::Index(i), PathSegment::Static("nthOfPeriod")].into(),
                        error: match e {
                            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
                            TypeErrorOr::Other(_) => {
                                TypeErrorOr::Other(ByRuleParseError::InvalidValue)
                            }
                        },
                    })?;
                    nth_val = Some(n.get());
                }
                _ => {}
            }
        }
        let weekday = day_val.ok_or_else(|| DocumentError {
            path: [PathSegment::Index(i)].into(),
            error: TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let ordinal = match nth_val {
            None => None,
            Some(0) => {
                return Err(DocumentError {
                    path: [PathSegment::Index(i)].into(),
                    error: TypeErrorOr::Other(ByRuleParseError::InvalidValue),
                });
            }
            Some(n) => {
                let sign = if n > 0 { Sign::Pos } else { Sign::Neg };
                let abs = u8::try_from(n.unsigned_abs()).map_err(|_| DocumentError {
                    path: [PathSegment::Index(i)].into(),
                    error: TypeErrorOr::Other(ByRuleParseError::InvalidValue),
                })?;
                let week = IsoWeek::from_index(abs).ok_or_else(|| DocumentError {
                    path: [PathSegment::Index(i)].into(),
                    error: TypeErrorOr::Other(ByRuleParseError::InvalidValue),
                })?;
                Some((sign, week))
            }
        };
        set.insert(crate::model::rrule::WeekdayNum { ordinal, weekday });
    }
    Ok(set)
}

fn parse_by_hour<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::HourSet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::HourSet::default();
    for elem in arr.into_iter() {
        let n = elem.try_as_unsigned_int().map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let h = crate::model::rrule::Hour::from_repr(
            u8::try_from(n.get()).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        set.set(h);
    }
    Ok(set)
}

fn parse_by_minute<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::MinuteSet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::MinuteSet::default();
    for elem in arr.into_iter() {
        let n = elem.try_as_unsigned_int().map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let m = crate::model::rrule::Minute::from_repr(
            u8::try_from(n.get()).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        set.set(m);
    }
    Ok(set)
}

fn parse_by_second<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::SecondSet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::SecondSet::default();
    for elem in arr.into_iter() {
        let n = elem.try_as_unsigned_int().map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let s = crate::model::rrule::Second::from_repr(
            u8::try_from(n.get()).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        set.set(s);
    }
    Ok(set)
}

fn parse_by_month<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::MonthSet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::MonthSet::default();
    for elem in arr.into_iter() {
        let n = elem.try_as_unsigned_int().map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let m = Month::new(
            u8::try_from(n.get()).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        set.set(m);
    }
    Ok(set)
}

fn parse_year_day_nums<V: DestructibleJsonValue>(
    val: V,
) -> Result<BTreeSet<crate::model::rrule::YearDayNum>, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = BTreeSet::new();
    for elem in arr.into_iter() {
        let n = Int::try_from_json(elem).map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let raw = n.get();
        let (sign, abs) = if raw >= 0 {
            (Sign::Pos, raw as u64)
        } else {
            (Sign::Neg, raw.unsigned_abs())
        };
        let abs_u16 = u16::try_from(abs)
            .map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        let ydn = crate::model::rrule::YearDayNum::from_signed_index(sign, abs_u16)
            .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        set.insert(ydn);
    }
    Ok(set)
}

fn parse_by_month_day<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::MonthDaySet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::MonthDaySet::default();
    for elem in arr.into_iter() {
        let n = Int::try_from_json(elem).map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let raw = n.get();
        let (sign, abs) = if raw >= 0 {
            (Sign::Pos, raw as u64)
        } else {
            (Sign::Neg, raw.unsigned_abs())
        };
        let md = crate::model::rrule::MonthDay::from_repr(
            u8::try_from(abs).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        let idx = crate::model::rrule::MonthDaySetIndex::from_signed_month_day(sign, md);
        set.set(idx);
    }
    Ok(set)
}

fn parse_by_week_no<V: DestructibleJsonValue>(
    val: V,
) -> Result<crate::model::rrule::WeekNoSet, TypeErrorOr<ByRuleParseError>> {
    let arr = val.try_into_array().map_err(TypeErrorOr::from)?;
    let mut set = crate::model::rrule::WeekNoSet::default();
    for elem in arr.into_iter() {
        let n = Int::try_from_json(elem).map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(ByRuleParseError::InvalidValue),
        })?;
        let raw = n.get();
        let (sign, abs) = if raw >= 0 {
            (Sign::Pos, raw as u64)
        } else {
            (Sign::Neg, raw.unsigned_abs())
        };
        let week = IsoWeek::from_index(
            u8::try_from(abs).map_err(|_| TypeErrorOr::Other(ByRuleParseError::InvalidValue))?,
        )
        .ok_or(TypeErrorOr::Other(ByRuleParseError::InvalidValue))?;
        let idx = crate::model::rrule::WeekNoSetIndex::from_signed_week(sign, week);
        set.set(idx);
    }
    Ok(set)
}

// ============================================================================
// Relation TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Relation<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut relations: Option<HashSet<Token<RelationValue>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "relation" => {
                    relations = Some(
                        HashSet::<Token<RelationValue>>::try_from_json(val)
                            .map_err(|e| doc_field_err("relation", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let relations = relations.unwrap_or_default();
        let mut result = Relation::new(relations);
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// OffsetTrigger TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for OffsetTrigger<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut offset_val: Option<SignedDuration> = None;
        let mut relative_to_val: Option<Token<AlertRelativeTo>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "offset" => {
                    offset_val = Some(
                        SignedDuration::try_from_json(val).map_err(|e| field_err("offset", e))?,
                    );
                }
                "relativeTo" => {
                    relative_to_val = Some(
                        Token::<AlertRelativeTo>::try_from_json(val)
                            .map_err(|e| type_field_err("relativeTo", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let offset = offset_val.ok_or_else(|| missing("offset"))?;
        let mut result = OffsetTrigger::new(offset);
        if let Some(v) = relative_to_val {
            result.set_relative_to(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// AbsoluteTrigger TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for AbsoluteTrigger<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut when_val: Option<DateTime<Utc>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "when" => {
                    when_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("when", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let when = when_val.ok_or_else(|| missing("when"))?;
        let mut result = AbsoluteTrigger::new(when);
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// Trigger TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Trigger<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let type_str = value
            .try_as_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?
            .get("@type")
            .and_then(|v| v.try_as_string().ok())
            .map(|s| s.as_ref().to_owned());

        match type_str.as_deref() {
            Some("OffsetTrigger") => OffsetTrigger::try_from_json(value).map(Trigger::Offset),
            Some("AbsoluteTrigger") => AbsoluteTrigger::try_from_json(value).map(Trigger::Absolute),
            _ => Err(missing("@type")),
        }
    }
}

// ============================================================================
// ReplyTo TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for ReplyTo {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut imip_val: Option<Box<CalAddress>> = None;
        let mut web_val: Option<Box<Uri>> = None;
        let mut other_parts: Vec<(String, Box<Uri>)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "imip" => {
                    imip_val = Some(
                        Box::<CalAddress>::try_from_json(val).map_err(|e| field_err("imip", e))?,
                    );
                }
                "web" => {
                    web_val =
                        Some(Box::<Uri>::try_from_json(val).map_err(|e| field_err("web", e))?);
                }
                other => {
                    // Try to parse value as Uri for other methods
                    if let Ok(uri) = Box::<Uri>::try_from_json(val) {
                        other_parts.push((other.into(), uri));
                    }
                }
            }
        }

        let mut result = ReplyTo::new();
        if let Some(v) = imip_val {
            result.set_imip(v);
        }
        if let Some(v) = web_val {
            result.set_web(v);
        }
        for (k, v) in other_parts {
            if let Ok(ak) = AlphaNumeric::new(k.as_ref()) {
                result.insert_other(ak.into(), v);
            }
        }
        Ok(result)
    }
}

// ============================================================================
// SendToParticipant TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for SendToParticipant {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut imip_val: Option<Box<CalAddress>> = None;
        let mut other_parts: Vec<(String, Box<Uri>)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "imip" => {
                    imip_val = Some(
                        Box::<CalAddress>::try_from_json(val).map_err(|e| field_err("imip", e))?,
                    );
                }
                other => {
                    if let Ok(uri) = Box::<Uri>::try_from_json(val) {
                        other_parts.push((other.into(), uri));
                    }
                }
            }
        }

        let mut result = SendToParticipant::new();
        if let Some(v) = imip_val {
            result.set_imip(v);
        }
        for (k, v) in other_parts {
            if let Ok(ak) = AlphaNumeric::new(k.as_ref()) {
                result.insert_other(ak.into(), v);
            }
        }
        Ok(result)
    }
}

// ============================================================================
// Link TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Link<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut href_val: Option<Box<Uri>> = None;
        let mut content_id_val: Option<Box<ContentId>> = None;
        let mut media_type_val: Option<Box<MediaType>> = None;
        let mut size_val: Option<UnsignedInt> = None;
        let mut relation_val: Option<LinkRelation> = None;
        let mut display_val: Option<Token<DisplayPurpose>> = None;
        let mut title_val: Option<String> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "href" => {
                    href_val =
                        Some(Box::<Uri>::try_from_json(val).map_err(|e| field_err("href", e))?);
                }
                "contentId" => {
                    content_id_val = Some(
                        Box::<ContentId>::try_from_json(val)
                            .map_err(|e| field_err("contentId", e))?,
                    );
                }
                "mediaType" => {
                    media_type_val = Some(
                        Box::<MediaType>::try_from_json(val)
                            .map_err(|e| field_err("mediaType", e))?,
                    );
                }
                "size" => {
                    size_val =
                        Some(UnsignedInt::try_from_json(val).map_err(|e| field_err("size", e))?);
                }
                "rel" => {
                    let s = val
                        .try_into_string()
                        .map_err(|e| type_field_err("rel", e))?;
                    use std::str::FromStr;
                    relation_val = Some(
                        LinkRelation::from_str(s.as_ref())
                            .map_err(|e| field_err("rel", TypeErrorOr::Other(e)))?,
                    );
                }
                "display" => {
                    display_val = Some(
                        Token::<DisplayPurpose>::try_from_json(val)
                            .map_err(|e| type_field_err("display", e))?,
                    );
                }
                "title" => {
                    title_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("title", e))?);
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let href = href_val.ok_or_else(|| missing("href"))?;
        let mut result = Link::new(href);
        if let Some(v) = content_id_val {
            result.set_content_id(v);
        }
        if let Some(v) = media_type_val {
            result.set_media_type(v);
        }
        if let Some(v) = size_val {
            result.set_size(v);
        }
        if let Some(v) = relation_val {
            result.set_relation(v);
        }
        if let Some(v) = display_val {
            result.set_display(v);
        }
        if let Some(v) = title_val {
            result.set_title(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// Helper functions for parsing arrays, maps, and sets
// ============================================================================

fn parse_vec<V, T, F>(value: V, parse_elem: F) -> Result<Vec<T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    let arr = value
        .try_into_array()
        .map_err(TypeErrorOr::from)
        .map_err(DocumentError::root)?;
    let mut out = Vec::new();
    for (i, elem) in arr.into_iter().enumerate() {
        let v = parse_elem(elem).map_err(|mut e| {
            e.path.push_front(PathSegment::Index(i));
            e
        })?;
        out.push(v);
    }
    Ok(out)
}

fn parse_map<V, K, T, KF, VF>(
    value: V,
    parse_key: KF,
    parse_val: VF,
) -> Result<HashMap<K, T>, ObjErr>
where
    V: DestructibleJsonValue,
    K: Eq + Hash,
    KF: Fn(&str) -> Result<K, ObjErr>,
    VF: Fn(V) -> Result<T, ObjErr>,
{
    let obj = value
        .try_into_object()
        .map_err(TypeErrorOr::from)
        .map_err(DocumentError::root)?;
    let mut out = HashMap::new();
    for (key, val) in obj.into_iter() {
        let k_str = <V::Object as JsonObject>::key_into_string(key);
        let k = parse_key(k_str.as_str())?;
        let v = parse_val(val).map_err(|mut e| {
            e.path
                .push_front(PathSegment::String(k_str.into_boxed_str()));
            e
        })?;
        out.insert(k, v);
    }
    Ok(out)
}

fn parse_id_set<V: DestructibleJsonValue>(value: V) -> Result<HashSet<Box<Id>>, ObjErr> {
    let arr = value
        .try_into_array()
        .map_err(TypeErrorOr::from)
        .map_err(DocumentError::root)?;
    let mut out = HashSet::new();
    for (i, elem) in arr.into_iter().enumerate() {
        let s = elem.try_into_string().map_err(|e| DocumentError {
            path: [PathSegment::Index(i)].into(),
            error: TypeErrorOr::TypeError(e),
        })?;
        let id: Box<Id> = Id::new(s.as_ref())
            .map(Into::into)
            .map_err(|e| DocumentError {
                path: [PathSegment::Index(i)].into(),
                error: TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    e.to_string().into_boxed_str(),
                )),
            })?;
        out.insert(id);
    }
    Ok(out)
}

fn parse_str_set<V: DestructibleJsonValue>(value: V) -> Result<HashSet<Box<str>>, ObjErr> {
    let arr = value
        .try_into_array()
        .map_err(TypeErrorOr::from)
        .map_err(DocumentError::root)?;
    let mut out = HashSet::new();
    for (i, elem) in arr.into_iter().enumerate() {
        let s = elem.try_into_string().map_err(|e| DocumentError {
            path: [PathSegment::Index(i)].into(),
            error: TypeErrorOr::TypeError(e),
        })?;
        out.insert(Box::<str>::from(s.as_ref()));
    }
    Ok(out)
}

fn rrule_vec<V: DestructibleJsonValue>(value: V) -> Result<Vec<RRule>, ObjErr> {
    parse_vec(value, |elem| {
        RRule::try_from_json(elem).map_err(|e| {
            let error = match e.error {
                TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
                TypeErrorOr::Other(re) => TypeErrorOr::Other(
                    ObjectFromJsonError::InvalidFieldValue(re.to_string().into_boxed_str()),
                ),
            };
            DocumentError {
                path: e.path,
                error,
            }
        })
    })
}

fn parse_id_map<V, T, F>(value: V, parse_val: F) -> Result<HashMap<Box<Id>, T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    parse_map(
        value,
        |k| {
            Id::new(k).map(Box::<Id>::from).map_err(|e| {
                DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    e.to_string().into_boxed_str(),
                )))
            })
        },
        parse_val,
    )
}

fn parse_tz_map<V, T, F>(
    value: V,
    parse_val: F,
) -> Result<HashMap<Box<CustomTimeZoneId>, T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    parse_map(
        value,
        |k| {
            CustomTimeZoneId::new(k)
                .map(Box::<CustomTimeZoneId>::from)
                .map_err(|e| {
                    DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                        e.to_string().into_boxed_str(),
                    )))
                })
        },
        parse_val,
    )
}

fn parse_uid_map<V, T, F>(value: V, parse_val: F) -> Result<HashMap<Box<Uid>, T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    parse_map(
        value,
        |k| {
            Uid::new(k).map(Box::<Uid>::from).map_err(|e| {
                DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    e.to_string().into_boxed_str(),
                )))
            })
        },
        parse_val,
    )
}

fn parse_dt_local_map<V, T, F>(
    value: V,
    parse_val: F,
) -> Result<HashMap<DateTime<Local>, T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    parse_map(
        value,
        |k| {
            crate::parser::parse_full(crate::parser::local_date_time)(k).map_err(|_| {
                DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    format!("invalid local datetime key: {k:?}").into_boxed_str(),
                )))
            })
        },
        parse_val,
    )
}

fn parse_lang_map<V, T, F>(value: V, parse_val: F) -> Result<HashMap<LanguageTag, T>, ObjErr>
where
    V: DestructibleJsonValue,
    F: Fn(V) -> Result<T, ObjErr>,
{
    parse_map(
        value,
        |k| {
            LanguageTag::parse(k).map_err(|e| {
                DocumentError::root(TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    e.to_string().into_boxed_str(),
                )))
            })
        },
        parse_val,
    )
}

fn parse_status_code_vec<V: DestructibleJsonValue>(value: V) -> Result<Vec<StatusCode>, ObjErr> {
    parse_vec(value, |elem| {
        StatusCode::try_from_json(elem).map_err(|e| {
            let error = match e {
                TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
                TypeErrorOr::Other(se) => TypeErrorOr::Other(
                    ObjectFromJsonError::InvalidFieldValue(se.to_string().into_boxed_str()),
                ),
            };
            DocumentError::root(error)
        })
    })
}

fn patch_object_from_json<V: DestructibleJsonValue>(value: V) -> Result<PatchObject<V>, ObjErr> {
    PatchObject::try_from_json(value).map_err(|e| match e {
        TypeErrorOr::TypeError(t) => DocumentError::root(TypeErrorOr::TypeError(t)),
        TypeErrorOr::Other(patch_err) => {
            let doc = patch_err.into_document_error();
            DocumentError {
                path: doc.path,
                error: TypeErrorOr::Other(ObjectFromJsonError::InvalidFieldValue(
                    doc.error.to_string().into_boxed_str(),
                )),
            }
        }
    })
}

fn parse_str_vec<V: DestructibleJsonValue>(value: V) -> Result<Vec<String>, ObjErr> {
    parse_vec(value, |elem| {
        String::try_from_json(elem).map_err(|e| DocumentError::root(TypeErrorOr::TypeError(e)))
    })
}

// ============================================================================
// Location TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Location<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut name_val: Option<String> = None;
        let mut description_val: Option<String> = None;
        let mut location_types_val: Option<HashSet<LocationType>> = None;
        let mut relative_to_val: Option<Token<RelationValue>> = None;
        let mut time_zone_val: Option<String> = None;
        let mut coordinates_val: Option<Box<GeoUri>> = None;
        let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "name" => {
                    name_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("name", e))?);
                }
                "description" => {
                    description_val = Some(
                        String::try_from_json(val).map_err(|e| type_field_err("description", e))?,
                    );
                }
                "locationTypes" => {
                    location_types_val = Some(
                        HashSet::<LocationType>::try_from_json(val)
                            .map_err(|e| doc_field_err("locationTypes", e))?,
                    );
                }
                "relativeTo" => {
                    relative_to_val = Some(
                        Token::<RelationValue>::try_from_json(val)
                            .map_err(|e| type_field_err("relativeTo", e))?,
                    );
                }
                "timeZone" => {
                    time_zone_val = Some(
                        String::try_from_json(val).map_err(|e| type_field_err("timeZone", e))?,
                    );
                }
                "coordinates" => {
                    coordinates_val = Some(
                        Box::<GeoUri>::try_from_json(val)
                            .map_err(|e| field_err("coordinates", e))?,
                    );
                }
                "links" => {
                    links_val = Some(
                        parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let mut result = Location::new();
        if let Some(v) = name_val {
            result.set_name(v);
        }
        if let Some(v) = description_val {
            result.set_description(v);
        }
        if let Some(v) = location_types_val {
            result.set_location_types(v);
        }
        if let Some(v) = relative_to_val {
            result.set_relative_to(v);
        }
        if let Some(v) = time_zone_val {
            result.set_time_zone(v);
        }
        if let Some(v) = coordinates_val {
            result.set_coordinates(v);
        }
        if let Some(v) = links_val {
            result.set_links(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// VirtualLocation TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for VirtualLocation<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut name_val: Option<String> = None;
        let mut description_val: Option<String> = None;
        let mut uri_val: Option<Box<Uri>> = None;
        let mut features_val: Option<HashSet<Token<VirtualLocationFeature>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "name" => {
                    name_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("name", e))?);
                }
                "description" => {
                    description_val = Some(
                        String::try_from_json(val).map_err(|e| type_field_err("description", e))?,
                    );
                }
                "uri" => {
                    uri_val =
                        Some(Box::<Uri>::try_from_json(val).map_err(|e| field_err("uri", e))?);
                }
                "features" => {
                    features_val = Some(
                        HashSet::<Token<VirtualLocationFeature>>::try_from_json(val)
                            .map_err(|e| doc_field_err("features", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let uri = uri_val.ok_or_else(|| missing("uri"))?;
        let mut result = VirtualLocation::new(uri);
        if let Some(v) = name_val {
            result.set_name(v);
        }
        if let Some(v) = description_val {
            result.set_description(v);
        }
        if let Some(v) = features_val {
            result.set_features(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// Alert TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Alert<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut trigger_val: Option<Trigger<V>> = None;
        let mut acknowledged_val: Option<DateTime<Utc>> = None;
        let mut related_to_val: Option<HashMap<Box<str>, Relation<V>>> = None;
        let mut action_val: Option<Token<AlertAction>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "trigger" => {
                    trigger_val =
                        Some(Trigger::try_from_json(val).map_err(|e| prepend("trigger", e))?);
                }
                "acknowledged" => {
                    acknowledged_val = Some(
                        DateTime::<Utc>::try_from_json(val)
                            .map_err(|e| field_err("acknowledged", e))?,
                    );
                }
                "relatedTo" => {
                    related_to_val = Some(
                        parse_map(val, |k| Ok(Box::<str>::from(k)), Relation::try_from_json)
                            .map_err(|e| prepend("relatedTo", e))?,
                    );
                }
                "action" => {
                    action_val = Some(
                        Token::<AlertAction>::try_from_json(val)
                            .map_err(|e| type_field_err("action", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let trigger = trigger_val.ok_or_else(|| missing("trigger"))?;
        let mut result = Alert::new(trigger);
        if let Some(v) = acknowledged_val {
            result.set_acknowledged(v);
        }
        if let Some(v) = related_to_val {
            result.set_related_to(v);
        }
        if let Some(v) = action_val {
            result.set_action(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// TimeZoneRule TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for TimeZoneRule<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut start_val: Option<DateTime<Local>> = None;
        let mut offset_from_val: Option<UtcOffset> = None;
        let mut offset_to_val: Option<UtcOffset> = None;
        let mut recurrence_rules_val: Option<Vec<RRule>> = None;
        let mut recurrence_overrides_val: Option<HashMap<DateTime<Local>, PatchObject<V>>> = None;
        let mut names_val: Option<HashSet<String>> = None;
        let mut comments_val: Option<Vec<String>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "start" => {
                    start_val = Some(
                        DateTime::<Local>::try_from_json(val).map_err(|e| field_err("start", e))?,
                    );
                }
                "offsetFrom" => {
                    offset_from_val = Some(
                        UtcOffset::try_from_json(val).map_err(|e| field_err("offsetFrom", e))?,
                    );
                }
                "offsetTo" => {
                    offset_to_val =
                        Some(UtcOffset::try_from_json(val).map_err(|e| field_err("offsetTo", e))?);
                }
                "recurrenceRules" => {
                    recurrence_rules_val =
                        Some(rrule_vec(val).map_err(|e| prepend("recurrenceRules", e))?);
                }
                "recurrenceOverrides" => {
                    recurrence_overrides_val = Some(
                        parse_dt_local_map(val, patch_object_from_json)
                            .map_err(|e| prepend("recurrenceOverrides", e))?,
                    );
                }
                "names" => {
                    names_val = Some(
                        HashSet::<String>::try_from_json(val)
                            .map_err(|e| doc_field_err("names", e))?,
                    );
                }
                "comments" => {
                    comments_val = Some(parse_str_vec(val).map_err(|e| prepend("comments", e))?);
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let start = start_val.ok_or_else(|| missing("start"))?;
        let offset_from = offset_from_val.ok_or_else(|| missing("offsetFrom"))?;
        let offset_to = offset_to_val.ok_or_else(|| missing("offsetTo"))?;
        let mut result = TimeZoneRule::new(start, offset_from, offset_to);
        if let Some(v) = recurrence_rules_val {
            result.set_recurrence_rules(v);
        }
        if let Some(v) = recurrence_overrides_val {
            result.set_recurrence_overrides(v);
        }
        if let Some(v) = names_val {
            result.set_names(v);
        }
        if let Some(v) = comments_val {
            result.set_comments(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// TimeZone TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for TimeZone<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut tz_id_val: Option<String> = None;
        let mut updated_val: Option<DateTime<Utc>> = None;
        let mut url_val: Option<Box<Uri>> = None;
        let mut valid_until_val: Option<DateTime<Utc>> = None;
        let mut aliases_val: Option<HashSet<Box<str>>> = None;
        let mut standard_val: Option<Vec<TimeZoneRule<V>>> = None;
        let mut daylight_val: Option<Vec<TimeZoneRule<V>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "tzId" => {
                    tz_id_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("tzId", e))?);
                }
                "updated" => {
                    updated_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("updated", e))?,
                    );
                }
                "url" => {
                    url_val =
                        Some(Box::<Uri>::try_from_json(val).map_err(|e| field_err("url", e))?);
                }
                "validUntil" => {
                    valid_until_val = Some(
                        DateTime::<Utc>::try_from_json(val)
                            .map_err(|e| field_err("validUntil", e))?,
                    );
                }
                "aliases" => {
                    aliases_val = Some(parse_str_set(val).map_err(|e| prepend("aliases", e))?);
                }
                "standard" => {
                    standard_val = Some(
                        parse_vec(val, TimeZoneRule::try_from_json)
                            .map_err(|e| prepend("standard", e))?,
                    );
                }
                "daylight" => {
                    daylight_val = Some(
                        parse_vec(val, TimeZoneRule::try_from_json)
                            .map_err(|e| prepend("daylight", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let tz_id = tz_id_val.ok_or_else(|| missing("tzId"))?;
        let mut result = TimeZone::new(tz_id);
        if let Some(v) = updated_val {
            result.set_updated(v);
        }
        if let Some(v) = url_val {
            result.set_url(v);
        }
        if let Some(v) = valid_until_val {
            result.set_valid_until(v);
        }
        if let Some(v) = aliases_val {
            result.set_aliases(v);
        }
        if let Some(v) = standard_val {
            result.set_standard(v);
        }
        if let Some(v) = daylight_val {
            result.set_daylight(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// Participant TryFromJson
// ============================================================================

// TODO: refactor this to remove the clippy lint about too many parameters, maybe by defining a
// struct type to use for the argument?

impl<V: DestructibleJsonValue> TryFromJson<V> for Participant<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut name_val: Option<String> = None;
        let mut email_val: Option<Box<EmailAddr>> = None;
        let mut description_val: Option<String> = None;
        let mut send_to_val: Option<SendToParticipant> = None;
        let mut kind_val: Option<Token<ParticipantKind>> = None;
        let mut roles_val: Option<HashSet<Token<ParticipantRole>>> = None;
        let mut location_id_val: Option<Box<Id>> = None;
        let mut language_val: Option<LanguageTag> = None;
        let mut participation_status_val: Option<Token<ParticipationStatus>> = None;
        let mut participation_comment_val: Option<String> = None;
        let mut expect_reply_val: Option<bool> = None;
        let mut schedule_agent_val: Option<Token<ScheduleAgent>> = None;
        let mut schedule_force_send_val: Option<bool> = None;
        let mut schedule_sequence_val: Option<UnsignedInt> = None;
        let mut schedule_status_val: Option<Vec<StatusCode>> = None;
        let mut schedule_updated_val: Option<DateTime<Utc>> = None;
        let mut sent_by_val: Option<Box<EmailAddr>> = None;
        let mut invited_by_val: Option<Box<Id>> = None;
        let mut delegated_to_val: Option<HashSet<Box<Id>>> = None;
        let mut delegated_from_val: Option<HashSet<Box<Id>>> = None;
        let mut member_of_val: Option<HashSet<Box<Id>>> = None;
        let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "name" => {
                    name_val = Some(String::try_from_json(val).map_err(|e| type_field_err("name", e))?);
                }
                "email" => {
                    email_val =
                        Some(Box::<EmailAddr>::try_from_json(val).map_err(|e| field_err("email", e))?);
                }
                "description" => {
                    description_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("description", e))?);
                }
                "sendTo" => {
                    send_to_val =
                        Some(SendToParticipant::try_from_json(val).map_err(|e| prepend("sendTo", e))?);
                }
                "kind" => {
                    kind_val = Some(
                        Token::<ParticipantKind>::try_from_json(val)
                            .map_err(|e| type_field_err("kind", e))?,
                    );
                }
                "roles" => {
                    roles_val = Some(
                        HashSet::<Token<ParticipantRole>>::try_from_json(val)
                            .map_err(|e| doc_field_err("roles", e))?,
                    );
                }
                "locationId" => {
                    location_id_val =
                        Some(Box::<Id>::try_from_json(val).map_err(|e| field_err("locationId", e))?);
                }
                "language" => {
                    language_val =
                        Some(LanguageTag::try_from_json(val).map_err(|e| field_err("language", e))?);
                }
                "participationStatus" => {
                    participation_status_val = Some(
                        Token::<ParticipationStatus>::try_from_json(val)
                            .map_err(|e| type_field_err("participationStatus", e))?,
                    );
                }
                "participationComment" => {
                    participation_comment_val = Some(
                        String::try_from_json(val)
                            .map_err(|e| type_field_err("participationComment", e))?,
                    );
                }
                "expectReply" => {
                    expect_reply_val =
                        Some(bool::try_from_json(val).map_err(|e| type_field_err("expectReply", e))?);
                }
                "scheduleAgent" => {
                    schedule_agent_val = Some(
                        Token::<ScheduleAgent>::try_from_json(val)
                            .map_err(|e| type_field_err("scheduleAgent", e))?,
                    );
                }
                "scheduleForceSend" => {
                    schedule_force_send_val =
                        Some(bool::try_from_json(val).map_err(|e| type_field_err("scheduleForceSend", e))?);
                }
                "scheduleSequence" => {
                    schedule_sequence_val = Some(
                        UnsignedInt::try_from_json(val).map_err(|e| field_err("scheduleSequence", e))?,
                    );
                }
                "scheduleStatus" => {
                    schedule_status_val =
                        Some(parse_status_code_vec(val).map_err(|e| prepend("scheduleStatus", e))?);
                }
                "scheduleUpdated" => {
                    schedule_updated_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("scheduleUpdated", e))?,
                    );
                }
                "sentBy" => {
                    sent_by_val =
                        Some(Box::<EmailAddr>::try_from_json(val).map_err(|e| field_err("sentBy", e))?);
                }
                "invitedBy" => {
                    invited_by_val =
                        Some(Box::<Id>::try_from_json(val).map_err(|e| field_err("invitedBy", e))?);
                }
                "delegatedTo" => {
                    delegated_to_val = Some(parse_id_set(val).map_err(|e| prepend("delegatedTo", e))?);
                }
                "delegatedFrom" => {
                    delegated_from_val = Some(parse_id_set(val).map_err(|e| prepend("delegatedFrom", e))?);
                }
                "memberOf" => {
                    member_of_val = Some(parse_id_set(val).map_err(|e| prepend("memberOf", e))?);
                }
                "links" => {
                    links_val =
                        Some(parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?);
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
                }
        }

        let mut result = Participant::new();
        if let Some(v) = name_val {
            result.set_name(v);
        }
        if let Some(v) = email_val {
            result.set_email(v);
        }
        if let Some(v) = description_val {
            result.set_description(v);
        }
        if let Some(v) = send_to_val {
            result.set_send_to(v);
        }
        if let Some(v) = kind_val {
            result.set_kind(v);
        }
        if let Some(v) = roles_val {
            result.set_roles(v);
        }
        if let Some(v) = location_id_val {
            result.set_location_id(v);
        }
        if let Some(v) = language_val {
            result.set_language(v);
        }
        if let Some(v) = participation_status_val {
            result.set_participation_status(v);
        }
        if let Some(v) = participation_comment_val {
            result.set_participation_comment(v);
        }
        if let Some(v) = expect_reply_val {
            result.set_expect_reply(v);
        }
        if let Some(v) = schedule_agent_val {
            result.set_schedule_agent(v);
        }
        if let Some(v) = schedule_force_send_val {
            result.set_schedule_force_send(v);
        }
        if let Some(v) = schedule_sequence_val {
            result.set_schedule_sequence(v);
        }
        if let Some(v) = schedule_status_val {
            result.set_schedule_status(v);
        }
        if let Some(v) = schedule_updated_val {
            result.set_schedule_updated(v);
        }
        if let Some(v) = sent_by_val {
            result.set_sent_by(v);
        }
        if let Some(v) = invited_by_val {
            result.set_invited_by(v);
        }
        if let Some(v) = delegated_to_val {
            result.set_delegated_to(v);
        }
        if let Some(v) = delegated_from_val {
            result.set_delegated_from(v);
        }
        if let Some(v) = member_of_val {
            result.set_member_of(v);
        }
        if let Some(v) = links_val {
            result.set_links(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// TaskParticipant TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for TaskParticipant<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut name_val: Option<String> = None;
        let mut email_val: Option<Box<EmailAddr>> = None;
        let mut description_val: Option<String> = None;
        let mut send_to_val: Option<SendToParticipant> = None;
        let mut kind_val: Option<Token<ParticipantKind>> = None;
        let mut roles_val: Option<HashSet<Token<ParticipantRole>>> = None;
        let mut location_id_val: Option<Box<Id>> = None;
        let mut language_val: Option<LanguageTag> = None;
        let mut participation_status_val: Option<Token<ParticipationStatus>> = None;
        let mut participation_comment_val: Option<String> = None;
        let mut expect_reply_val: Option<bool> = None;
        let mut schedule_agent_val: Option<Token<ScheduleAgent>> = None;
        let mut schedule_force_send_val: Option<bool> = None;
        let mut schedule_sequence_val: Option<UnsignedInt> = None;
        let mut schedule_status_val: Option<Vec<StatusCode>> = None;
        let mut schedule_updated_val: Option<DateTime<Utc>> = None;
        let mut sent_by_val: Option<Box<EmailAddr>> = None;
        let mut invited_by_val: Option<Box<Id>> = None;
        let mut delegated_to_val: Option<HashSet<Box<Id>>> = None;
        let mut delegated_from_val: Option<HashSet<Box<Id>>> = None;
        let mut member_of_val: Option<HashSet<Box<Id>>> = None;
        let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
        let mut progress_val: Option<Token<TaskProgress>> = None;
        let mut progress_updated_val: Option<DateTime<Utc>> = None;
        let mut percent_complete_val: Option<Percent> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "progress" => {
                    progress_val = Some(
                        Token::<TaskProgress>::try_from_json(val)
                            .map_err(|e| type_field_err("progress", e))?,
                    );
                }
                "progressUpdated" => {
                    progress_updated_val = Some(
                        DateTime::<Utc>::try_from_json(val)
                            .map_err(|e| field_err("progressUpdated", e))?,
                    );
                }
                "percentComplete" => {
                    percent_complete_val = Some(
                        Percent::try_from_json(val).map_err(|e| field_err("percentComplete", e))?,
                    );
                }
                "name" => {
                    name_val = Some(String::try_from_json(val).map_err(|e| type_field_err("name", e))?);
                }
                "email" => {
                    email_val =
                        Some(Box::<EmailAddr>::try_from_json(val).map_err(|e| field_err("email", e))?);
                }
                "description" => {
                    description_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("description", e))?);
                }
                "sendTo" => {
                    send_to_val =
                        Some(SendToParticipant::try_from_json(val).map_err(|e| prepend("sendTo", e))?);
                }
                "kind" => {
                    kind_val = Some(
                        Token::<ParticipantKind>::try_from_json(val)
                            .map_err(|e| type_field_err("kind", e))?,
                    );
                }
                "roles" => {
                    roles_val = Some(
                        HashSet::<Token<ParticipantRole>>::try_from_json(val)
                            .map_err(|e| doc_field_err("roles", e))?,
                    );
                }
                "locationId" => {
                    location_id_val =
                        Some(Box::<Id>::try_from_json(val).map_err(|e| field_err("locationId", e))?);
                }
                "language" => {
                    language_val =
                        Some(LanguageTag::try_from_json(val).map_err(|e| field_err("language", e))?);
                }
                "participationStatus" => {
                    participation_status_val = Some(
                        Token::<ParticipationStatus>::try_from_json(val)
                            .map_err(|e| type_field_err("participationStatus", e))?,
                    );
                }
                "participationComment" => {
                    participation_comment_val = Some(
                        String::try_from_json(val)
                            .map_err(|e| type_field_err("participationComment", e))?,
                    );
                }
                "expectReply" => {
                    expect_reply_val =
                        Some(bool::try_from_json(val).map_err(|e| type_field_err("expectReply", e))?);
                }
                "scheduleAgent" => {
                    schedule_agent_val = Some(
                        Token::<ScheduleAgent>::try_from_json(val)
                            .map_err(|e| type_field_err("scheduleAgent", e))?,
                    );
                }
                "scheduleForceSend" => {
                    schedule_force_send_val =
                        Some(bool::try_from_json(val).map_err(|e| type_field_err("scheduleForceSend", e))?);
                }
                "scheduleSequence" => {
                    schedule_sequence_val = Some(
                        UnsignedInt::try_from_json(val).map_err(|e| field_err("scheduleSequence", e))?,
                    );
                }
                "scheduleStatus" => {
                    schedule_status_val =
                        Some(parse_status_code_vec(val).map_err(|e| prepend("scheduleStatus", e))?);
                }
                "scheduleUpdated" => {
                    schedule_updated_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("scheduleUpdated", e))?,
                    );
                }
                "sentBy" => {
                    sent_by_val =
                        Some(Box::<EmailAddr>::try_from_json(val).map_err(|e| field_err("sentBy", e))?);
                }
                "invitedBy" => {
                    invited_by_val =
                        Some(Box::<Id>::try_from_json(val).map_err(|e| field_err("invitedBy", e))?);
                }
                "delegatedTo" => {
                    delegated_to_val = Some(parse_id_set(val).map_err(|e| prepend("delegatedTo", e))?);
                }
                "delegatedFrom" => {
                    delegated_from_val = Some(parse_id_set(val).map_err(|e| prepend("delegatedFrom", e))?);
                }
                "memberOf" => {
                    member_of_val = Some(parse_id_set(val).map_err(|e| prepend("memberOf", e))?);
                }
                "links" => {
                    links_val =
                        Some(parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?);
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
                }
        }

        let mut result = TaskParticipant::new();
        if let Some(v) = name_val {
            result.set_name(v);
        }
        if let Some(v) = email_val {
            result.set_email(v);
        }
        if let Some(v) = description_val {
            result.set_description(v);
        }
        if let Some(v) = send_to_val {
            result.set_send_to(v);
        }
        if let Some(v) = kind_val {
            result.set_kind(v);
        }
        if let Some(v) = roles_val {
            result.set_roles(v);
        }
        if let Some(v) = location_id_val {
            result.set_location_id(v);
        }
        if let Some(v) = language_val {
            result.set_language(v);
        }
        if let Some(v) = participation_status_val {
            result.set_participation_status(v);
        }
        if let Some(v) = participation_comment_val {
            result.set_participation_comment(v);
        }
        if let Some(v) = expect_reply_val {
            result.set_expect_reply(v);
        }
        if let Some(v) = schedule_agent_val {
            result.set_schedule_agent(v);
        }
        if let Some(v) = schedule_force_send_val {
            result.set_schedule_force_send(v);
        }
        if let Some(v) = schedule_sequence_val {
            result.set_schedule_sequence(v);
        }
        if let Some(v) = schedule_status_val {
            result.set_schedule_status(v);
        }
        if let Some(v) = schedule_updated_val {
            result.set_schedule_updated(v);
        }
        if let Some(v) = sent_by_val {
            result.set_sent_by(v);
        }
        if let Some(v) = invited_by_val {
            result.set_invited_by(v);
        }
        if let Some(v) = delegated_to_val {
            result.set_delegated_to(v);
        }
        if let Some(v) = delegated_from_val {
            result.set_delegated_from(v);
        }
        if let Some(v) = member_of_val {
            result.set_member_of(v);
        }
        if let Some(v) = links_val {
            result.set_links(v);
        }
        if let Some(v) = progress_val {
            result.set_progress(v);
        }
        if let Some(v) = progress_updated_val {
            result.set_progress_updated(v);
        }
        if let Some(v) = percent_complete_val {
            result.set_percent_complete(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// Event TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Event<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;


            let mut start_val: Option<DateTime<Local>> = None;
            let mut duration_val: Option<Duration> = None;
            let mut status_val: Option<Token<EventStatus>> = None;
            let mut uid_val: Option<Box<Uid>> = None;
            let mut related_to_val: Option<HashMap<Box<Uid>, Relation<V>>> = None;
            let mut prod_id_val: Option<String> = None;
            let mut created_val: Option<DateTime<Utc>> = None;
            let mut updated_val: Option<DateTime<Utc>> = None;
            let mut sequence_val: Option<UnsignedInt> = None;
            let mut method_val: Option<Token<Method>> = None;
            let mut title_val: Option<String> = None;
            let mut description_val: Option<String> = None;
            let mut description_content_type_val: Option<String> = None;
            let mut show_without_time_val: Option<bool> = None;
            let mut locations_val: Option<HashMap<Box<Id>, Location<V>>> = None;
            let mut virtual_locations_val: Option<HashMap<Box<Id>, VirtualLocation<V>>> = None;
            let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
            let mut locale_val: Option<LanguageTag> = None;
            let mut keywords_val: Option<HashSet<String>> = None;
            let mut categories_val: Option<HashSet<String>> = None;
            let mut color_val: Option<Color> = None;
            let mut recurrence_id_val: Option<DateTime<Local>> = None;
            let mut recurrence_id_time_zone_val: Option<String> = None;
            let mut recurrence_rules_val: Option<Vec<RRule>> = None;
            let mut excluded_recurrence_rules_val: Option<Vec<RRule>> = None;
            let mut recurrence_overrides_val: Option<HashMap<DateTime<Local>, PatchObject<V>>> = None;
            let mut excluded_val: Option<bool> = None;
            let mut priority_val: Option<Priority> = None;
            let mut free_busy_status_val: Option<Token<FreeBusyStatus>> = None;
            let mut privacy_val: Option<Token<Privacy>> = None;
            let mut reply_to_val: Option<ReplyTo> = None;
            let mut sent_by_val: Option<Box<CalAddress>> = None;
            let mut participants_val: Option<HashMap<Box<Id>, Participant<V>>> = None;
            let mut request_status_val: Option<RequestStatus> = None;
            let mut use_default_alerts_val: Option<bool> = None;
            let mut alerts_val: Option<HashMap<Box<Id>, Alert<V>>> = None;
            let mut localizations_val: Option<HashMap<LanguageTag, PatchObject<V>>> = None;
            let mut time_zone_val: Option<String> = None;
            let mut time_zones_val: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>> = None;
            let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

            for (key, val) in obj.into_iter() {
                let k = <V::Object as JsonObject>::key_into_string(key);
                match k.as_str() {
                    "@type" => {}
                    "start" => {
                        start_val =
                            Some(DateTime::<Local>::try_from_json(val).map_err(|e| field_err("start", e))?);
                    }
                    "duration" => {
                        duration_val =
                            Some(Duration::try_from_json(val).map_err(|e| field_err("duration", e))?);
                    }
                    "status" => {
                        status_val = Some(
                            Token::<EventStatus>::try_from_json(val)
                                .map_err(|e| type_field_err("status", e))?,
                        );
                    }
                    "uid" => {
                        uid_val = Some(Box::<Uid>::try_from_json(val).map_err(|e| field_err("uid", e))?);
                    }
                    "relatedTo" => {
                        related_to_val = Some(
                            parse_uid_map(val, Relation::try_from_json)
                                .map_err(|e| prepend("relatedTo", e))?,
                        );
                    }
                    "prodId" => {
                        prod_id_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("prodId", e))?);
                    }
                    "created" => {
                        created_val =
                            Some(DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("created", e))?);
                    }
                    "updated" => {
                        updated_val =
                            Some(DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("updated", e))?);
                    }
                    "sequence" => {
                        sequence_val =
                            Some(UnsignedInt::try_from_json(val).map_err(|e| field_err("sequence", e))?);
                    }
                    "method" => {
                        method_val = Some(
                            Token::<Method>::try_from_json(val).map_err(|e| type_field_err("method", e))?,
                        );
                    }
                    "title" => {
                        title_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("title", e))?);
                    }
                    "description" => {
                        description_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("description", e))?);
                    }
                    "descriptionContentType" => {
                        description_content_type_val = Some(
                            String::try_from_json(val)
                                .map_err(|e| type_field_err("descriptionContentType", e))?,
                        );
                    }
                    "showWithoutTime" => {
                        show_without_time_val = Some(
                            bool::try_from_json(val).map_err(|e| type_field_err("showWithoutTime", e))?,
                        );
                    }
                    "locations" => {
                        locations_val = Some(
                            parse_id_map(val, Location::try_from_json)
                                .map_err(|e| prepend("locations", e))?,
                        );
                    }
                    "virtualLocations" => {
                        virtual_locations_val = Some(
                            parse_id_map(val, VirtualLocation::try_from_json)
                                .map_err(|e| prepend("virtualLocations", e))?,
                        );
                    }
                    "links" => {
                        links_val =
                            Some(parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?);
                    }
                    "locale" => {
                        locale_val =
                            Some(LanguageTag::try_from_json(val).map_err(|e| field_err("locale", e))?);
                    }
                    "keywords" => {
                        keywords_val = Some(
                            HashSet::<String>::try_from_json(val)
                                .map_err(|e| doc_field_err("keywords", e))?,
                        );
                    }
                    "categories" => {
                        categories_val = Some(
                            HashSet::<String>::try_from_json(val)
                                .map_err(|e| doc_field_err("categories", e))?,
                        );
                    }
                    "color" => {
                        color_val = Some(Color::try_from_json(val).map_err(|e| field_err("color", e))?);
                    }
                    "recurrenceId" => {
                        recurrence_id_val = Some(
                            DateTime::<Local>::try_from_json(val)
                                .map_err(|e| field_err("recurrenceId", e))?,
                        );
                    }
                    "recurrenceIdTimeZone" => {
                        recurrence_id_time_zone_val = Some(
                            String::try_from_json(val)
                                .map_err(|e| type_field_err("recurrenceIdTimeZone", e))?,
                        );
                    }
                    "recurrenceRules" => {
                        recurrence_rules_val =
                            Some(rrule_vec(val).map_err(|e| prepend("recurrenceRules", e))?);
                    }
                    "excludedRecurrenceRules" => {
                        excluded_recurrence_rules_val =
                            Some(rrule_vec(val).map_err(|e| prepend("excludedRecurrenceRules", e))?);
                    }
                    "recurrenceOverrides" => {
                        recurrence_overrides_val = Some(
                            parse_dt_local_map(val, patch_object_from_json)
                                .map_err(|e| prepend("recurrenceOverrides", e))?,
                        );
                    }
                    "excluded" => {
                        excluded_val =
                            Some(bool::try_from_json(val).map_err(|e| type_field_err("excluded", e))?);
                    }
                    "priority" => {
                        priority_val =
                            Some(Priority::try_from_json(val).map_err(|e| field_err("priority", e))?);
                    }
                    "freeBusyStatus" => {
                        free_busy_status_val = Some(
                            Token::<FreeBusyStatus>::try_from_json(val)
                                .map_err(|e| type_field_err("freeBusyStatus", e))?,
                        );
                    }
                    "privacy" => {
                        privacy_val = Some(
                            Token::<Privacy>::try_from_json(val)
                                .map_err(|e| type_field_err("privacy", e))?,
                        );
                    }
                    "replyTo" => {
                        reply_to_val =
                            Some(ReplyTo::try_from_json(val).map_err(|e| prepend("replyTo", e))?);
                    }
                    "sentBy" => {
                        sent_by_val = Some(
                            Box::<CalAddress>::try_from_json(val).map_err(|e| field_err("sentBy", e))?,
                        );
                    }
                    "participants" => {
                        participants_val = Some(
                            parse_id_map(val, Participant::try_from_json)
                                .map_err(|e| prepend("participants", e))?,
                        );
                    }
                    "requestStatus" => {
                        request_status_val = Some(
                            RequestStatus::try_from_json(val).map_err(|e| field_err("requestStatus", e))?,
                        );
                    }
                    "useDefaultAlerts" => {
                        use_default_alerts_val = Some(
                            bool::try_from_json(val).map_err(|e| type_field_err("useDefaultAlerts", e))?,
                        );
                    }
                    "alerts" => {
                        alerts_val = Some(
                            parse_id_map(val, Alert::try_from_json).map_err(|e| prepend("alerts", e))?,
                        );
                    }
                    "localizations" => {
                        localizations_val = Some(
                            parse_lang_map(val, patch_object_from_json)
                                .map_err(|e| prepend("localizations", e))?,
                        );
                    }
                    "timeZone" => {
                        time_zone_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("timeZone", e))?);
                    }
                    "timeZones" => {
                        time_zones_val = Some(
                            parse_tz_map(val, TimeZone::try_from_json)
                                .map_err(|e| prepend("timeZones", e))?,
                        );
                    }
                    _ => vendor_parts.push((k.into_boxed_str(), val)),
                }
            }

            let start = start_val.ok_or_else(|| missing("start"))?;
            let uid = uid_val.ok_or_else(|| missing("uid"))?;
            let mut result = Event::new(start, uid);
            if let Some(v) = duration_val {
                result.set_duration(v);
            }
            if let Some(v) = status_val {
                result.set_status(v);
            }
            if let Some(v) = related_to_val {
                result.set_related_to(v);
            }
            if let Some(v) = prod_id_val {
                result.set_prod_id(v);
            }
            if let Some(v) = created_val {
                result.set_created(v);
            }
            if let Some(v) = updated_val {
                result.set_updated(v);
            }
            if let Some(v) = sequence_val {
                result.set_sequence(v);
            }
            if let Some(v) = method_val {
                result.set_method(v);
            }
            if let Some(v) = title_val {
                result.set_title(v);
            }
            if let Some(v) = description_val {
                result.set_description(v);
            }
            if let Some(v) = description_content_type_val {
                result.set_description_content_type(v);
            }
            if let Some(v) = show_without_time_val {
                result.set_show_without_time(v);
            }
            if let Some(v) = locations_val {
                result.set_locations(v);
            }
            if let Some(v) = virtual_locations_val {
                result.set_virtual_locations(v);
            }
            if let Some(v) = links_val {
                result.set_links(v);
            }
            if let Some(v) = locale_val {
                result.set_locale(v);
            }
            if let Some(v) = keywords_val {
                result.set_keywords(v);
            }
            if let Some(v) = categories_val {
                result.set_categories(v);
            }
            if let Some(v) = color_val {
                result.set_color(v);
            }
            if let Some(v) = recurrence_id_val {
                result.set_recurrence_id(v);
            }
            if let Some(v) = recurrence_id_time_zone_val {
                result.set_recurrence_id_time_zone(v);
            }
            if let Some(v) = recurrence_rules_val {
                result.set_recurrence_rules(v);
            }
            if let Some(v) = excluded_recurrence_rules_val {
                result.set_excluded_recurrence_rules(v);
            }
            if let Some(v) = recurrence_overrides_val {
                result.set_recurrence_overrides(v);
            }
            if let Some(v) = excluded_val {
                result.set_excluded(v);
            }
            if let Some(v) = priority_val {
                result.set_priority(v);
            }
            if let Some(v) = free_busy_status_val {
                result.set_free_busy_status(v);
            }
            if let Some(v) = privacy_val {
                result.set_privacy(v);
            }
            if let Some(v) = reply_to_val {
                result.set_reply_to(v);
            }
            if let Some(v) = sent_by_val {
                result.set_sent_by(v);
            }
            if let Some(v) = participants_val {
                result.set_participants(v);
            }
            if let Some(v) = request_status_val {
                result.set_request_status(v);
            }
            if let Some(v) = use_default_alerts_val {
                result.set_use_default_alerts(v);
            }
            if let Some(v) = alerts_val {
                result.set_alerts(v);
            }
            if let Some(v) = localizations_val {
                result.set_localizations(v);
            }
            if let Some(v) = time_zone_val {
                result.set_time_zone(v);
            }
            if let Some(v) = time_zones_val {
                result.set_time_zones(v);
            }
            for (k, v) in vendor_parts {
                result.insert_vendor_property(k, v);
            }
            Ok(result)
    }
}

// ============================================================================
// Task TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Task<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;


            let mut due_val: Option<DateTime<Local>> = None;
            let mut start_val: Option<DateTime<Local>> = None;
            let mut estimated_duration_val: Option<Duration> = None;
            let mut percent_complete_val: Option<Percent> = None;
            let mut progress_val: Option<Token<TaskProgress>> = None;
            let mut progress_updated_val: Option<DateTime<Utc>> = None;
            let mut uid_val: Option<Box<Uid>> = None;
            let mut related_to_val: Option<HashMap<Box<Uid>, Relation<V>>> = None;
            let mut prod_id_val: Option<String> = None;
            let mut created_val: Option<DateTime<Utc>> = None;
            let mut updated_val: Option<DateTime<Utc>> = None;
            let mut sequence_val: Option<UnsignedInt> = None;
            let mut method_val: Option<Token<Method>> = None;
            let mut title_val: Option<String> = None;
            let mut description_val: Option<String> = None;
            let mut description_content_type_val: Option<String> = None;
            let mut show_without_time_val: Option<bool> = None;
            let mut locations_val: Option<HashMap<Box<Id>, Location<V>>> = None;
            let mut virtual_locations_val: Option<HashMap<Box<Id>, VirtualLocation<V>>> = None;
            let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
            let mut locale_val: Option<LanguageTag> = None;
            let mut keywords_val: Option<HashSet<String>> = None;
            let mut categories_val: Option<HashSet<String>> = None;
            let mut color_val: Option<Color> = None;
            let mut recurrence_id_val: Option<DateTime<Local>> = None;
            let mut recurrence_id_time_zone_val: Option<String> = None;
            let mut recurrence_rules_val: Option<Vec<RRule>> = None;
            let mut excluded_recurrence_rules_val: Option<Vec<RRule>> = None;
            let mut recurrence_overrides_val: Option<HashMap<DateTime<Local>, PatchObject<V>>> = None;
            let mut excluded_val: Option<bool> = None;
            let mut priority_val: Option<Priority> = None;
            let mut free_busy_status_val: Option<Token<FreeBusyStatus>> = None;
            let mut privacy_val: Option<Token<Privacy>> = None;
            let mut reply_to_val: Option<ReplyTo> = None;
            let mut sent_by_val: Option<Box<CalAddress>> = None;
            let mut participants_val: Option<HashMap<Box<Id>, TaskParticipant<V>>> = None;
            let mut request_status_val: Option<RequestStatus> = None;
            let mut use_default_alerts_val: Option<bool> = None;
            let mut alerts_val: Option<HashMap<Box<Id>, Alert<V>>> = None;
            let mut localizations_val: Option<HashMap<LanguageTag, PatchObject<V>>> = None;
            let mut time_zone_val: Option<String> = None;
            let mut time_zones_val: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>> = None;
            let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

            for (key, val) in obj.into_iter() {
                let k = <V::Object as JsonObject>::key_into_string(key);
                match k.as_str() {
                    "@type" => {}
                    "due" => {
                        due_val =
                            Some(DateTime::<Local>::try_from_json(val).map_err(|e| field_err("due", e))?);
                    }
                    "start" => {
                        start_val =
                            Some(DateTime::<Local>::try_from_json(val).map_err(|e| field_err("start", e))?);
                    }
                    "estimatedDuration" => {
                        estimated_duration_val = Some(
                            Duration::try_from_json(val).map_err(|e| field_err("estimatedDuration", e))?,
                        );
                    }
                    "percentComplete" => {
                        percent_complete_val =
                            Some(Percent::try_from_json(val).map_err(|e| field_err("percentComplete", e))?);
                    }
                    "progress" => {
                        progress_val = Some(
                            Token::<TaskProgress>::try_from_json(val)
                                .map_err(|e| type_field_err("progress", e))?,
                        );
                    }
                    "progressUpdated" => {
                        progress_updated_val = Some(
                            DateTime::<Utc>::try_from_json(val)
                                .map_err(|e| field_err("progressUpdated", e))?,
                        );
                    }
                    "uid" => {
                        uid_val = Some(Box::<Uid>::try_from_json(val).map_err(|e| field_err("uid", e))?);
                    }
                    "relatedTo" => {
                        related_to_val = Some(
                            parse_uid_map(val, Relation::try_from_json)
                                .map_err(|e| prepend("relatedTo", e))?,
                        );
                    }
                    "prodId" => {
                        prod_id_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("prodId", e))?);
                    }
                    "created" => {
                        created_val =
                            Some(DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("created", e))?);
                    }
                    "updated" => {
                        updated_val =
                            Some(DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("updated", e))?);
                    }
                    "sequence" => {
                        sequence_val =
                            Some(UnsignedInt::try_from_json(val).map_err(|e| field_err("sequence", e))?);
                    }
                    "method" => {
                        method_val = Some(
                            Token::<Method>::try_from_json(val).map_err(|e| type_field_err("method", e))?,
                        );
                    }
                    "title" => {
                        title_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("title", e))?);
                    }
                    "description" => {
                        description_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("description", e))?);
                    }
                    "descriptionContentType" => {
                        description_content_type_val = Some(
                            String::try_from_json(val)
                                .map_err(|e| type_field_err("descriptionContentType", e))?,
                        );
                    }
                    "showWithoutTime" => {
                        show_without_time_val = Some(
                            bool::try_from_json(val).map_err(|e| type_field_err("showWithoutTime", e))?,
                        );
                    }
                    "locations" => {
                        locations_val = Some(
                            parse_id_map(val, Location::try_from_json)
                                .map_err(|e| prepend("locations", e))?,
                        );
                    }
                    "virtualLocations" => {
                        virtual_locations_val = Some(
                            parse_id_map(val, VirtualLocation::try_from_json)
                                .map_err(|e| prepend("virtualLocations", e))?,
                        );
                    }
                    "links" => {
                        links_val =
                            Some(parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?);
                    }
                    "locale" => {
                        locale_val =
                            Some(LanguageTag::try_from_json(val).map_err(|e| field_err("locale", e))?);
                    }
                    "keywords" => {
                        keywords_val = Some(
                            HashSet::<String>::try_from_json(val)
                                .map_err(|e| doc_field_err("keywords", e))?,
                        );
                    }
                    "categories" => {
                        categories_val = Some(
                            HashSet::<String>::try_from_json(val)
                                .map_err(|e| doc_field_err("categories", e))?,
                        );
                    }
                    "color" => {
                        color_val = Some(Color::try_from_json(val).map_err(|e| field_err("color", e))?);
                    }
                    "recurrenceId" => {
                        recurrence_id_val = Some(
                            DateTime::<Local>::try_from_json(val)
                                .map_err(|e| field_err("recurrenceId", e))?,
                        );
                    }
                    "recurrenceIdTimeZone" => {
                        recurrence_id_time_zone_val = Some(
                            String::try_from_json(val)
                                .map_err(|e| type_field_err("recurrenceIdTimeZone", e))?,
                        );
                    }
                    "recurrenceRules" => {
                        recurrence_rules_val =
                            Some(rrule_vec(val).map_err(|e| prepend("recurrenceRules", e))?);
                    }
                    "excludedRecurrenceRules" => {
                        excluded_recurrence_rules_val =
                            Some(rrule_vec(val).map_err(|e| prepend("excludedRecurrenceRules", e))?);
                    }
                    "recurrenceOverrides" => {
                        recurrence_overrides_val = Some(
                            parse_dt_local_map(val, patch_object_from_json)
                                .map_err(|e| prepend("recurrenceOverrides", e))?,
                        );
                    }
                    "excluded" => {
                        excluded_val =
                            Some(bool::try_from_json(val).map_err(|e| type_field_err("excluded", e))?);
                    }
                    "priority" => {
                        priority_val =
                            Some(Priority::try_from_json(val).map_err(|e| field_err("priority", e))?);
                    }
                    "freeBusyStatus" => {
                        free_busy_status_val = Some(
                            Token::<FreeBusyStatus>::try_from_json(val)
                                .map_err(|e| type_field_err("freeBusyStatus", e))?,
                        );
                    }
                    "privacy" => {
                        privacy_val = Some(
                            Token::<Privacy>::try_from_json(val)
                                .map_err(|e| type_field_err("privacy", e))?,
                        );
                    }
                    "replyTo" => {
                        reply_to_val =
                            Some(ReplyTo::try_from_json(val).map_err(|e| prepend("replyTo", e))?);
                    }
                    "sentBy" => {
                        sent_by_val = Some(
                            Box::<CalAddress>::try_from_json(val).map_err(|e| field_err("sentBy", e))?,
                        );
                    }
                    "participants" => {
                        participants_val = Some(
                            parse_id_map(val, TaskParticipant::try_from_json)
                                .map_err(|e| prepend("participants", e))?,
                        );
                    }
                    "requestStatus" => {
                        request_status_val = Some(
                            RequestStatus::try_from_json(val).map_err(|e| field_err("requestStatus", e))?,
                        );
                    }
                    "useDefaultAlerts" => {
                        use_default_alerts_val = Some(
                            bool::try_from_json(val).map_err(|e| type_field_err("useDefaultAlerts", e))?,
                        );
                    }
                    "alerts" => {
                        alerts_val = Some(
                            parse_id_map(val, Alert::try_from_json).map_err(|e| prepend("alerts", e))?,
                        );
                    }
                    "localizations" => {
                        localizations_val = Some(
                            parse_lang_map(val, patch_object_from_json)
                                .map_err(|e| prepend("localizations", e))?,
                        );
                    }
                    "timeZone" => {
                        time_zone_val =
                            Some(String::try_from_json(val).map_err(|e| type_field_err("timeZone", e))?);
                    }
                    "timeZones" => {
                        time_zones_val = Some(
                            parse_tz_map(val, TimeZone::try_from_json)
                                .map_err(|e| prepend("timeZones", e))?,
                        );
                    }
                    _ => vendor_parts.push((k.into_boxed_str(), val)),
                }
            }

            let uid = uid_val.ok_or_else(|| missing("uid"))?;
            let mut result = Task::new(uid);
            if let Some(v) = due_val {
                result.set_due(v);
            }
            if let Some(v) = start_val {
                result.set_start(v);
            }
            if let Some(v) = estimated_duration_val {
                result.set_estimated_duration(v);
            }
            if let Some(v) = percent_complete_val {
                result.set_percent_complete(v);
            }
            if let Some(v) = progress_val {
                result.set_progress(v);
            }
            if let Some(v) = progress_updated_val {
                result.set_progress_updated(v);
            }
            if let Some(v) = related_to_val {
                result.set_related_to(v);
            }
            if let Some(v) = prod_id_val {
                result.set_prod_id(v);
            }
            if let Some(v) = created_val {
                result.set_created(v);
            }
            if let Some(v) = updated_val {
                result.set_updated(v);
            }
            if let Some(v) = sequence_val {
                result.set_sequence(v);
            }
            if let Some(v) = method_val {
                result.set_method(v);
            }
            if let Some(v) = title_val {
                result.set_title(v);
            }
            if let Some(v) = description_val {
                result.set_description(v);
            }
            if let Some(v) = description_content_type_val {
                result.set_description_content_type(v);
            }
            if let Some(v) = show_without_time_val {
                result.set_show_without_time(v);
            }
            if let Some(v) = locations_val {
                result.set_locations(v);
            }
            if let Some(v) = virtual_locations_val {
                result.set_virtual_locations(v);
            }
            if let Some(v) = links_val {
                result.set_links(v);
            }
            if let Some(v) = locale_val {
                result.set_locale(v);
            }
            if let Some(v) = keywords_val {
                result.set_keywords(v);
            }
            if let Some(v) = categories_val {
                result.set_categories(v);
            }
            if let Some(v) = color_val {
                result.set_color(v);
            }
            if let Some(v) = recurrence_id_val {
                result.set_recurrence_id(v);
            }
            if let Some(v) = recurrence_id_time_zone_val {
                result.set_recurrence_id_time_zone(v);
            }
            if let Some(v) = recurrence_rules_val {
                result.set_recurrence_rules(v);
            }
            if let Some(v) = excluded_recurrence_rules_val {
                result.set_excluded_recurrence_rules(v);
            }
            if let Some(v) = recurrence_overrides_val {
                result.set_recurrence_overrides(v);
            }
            if let Some(v) = excluded_val {
                result.set_excluded(v);
            }
            if let Some(v) = priority_val {
                result.set_priority(v);
            }
            if let Some(v) = free_busy_status_val {
                result.set_free_busy_status(v);
            }
            if let Some(v) = privacy_val {
                result.set_privacy(v);
            }
            if let Some(v) = reply_to_val {
                result.set_reply_to(v);
            }
            if let Some(v) = sent_by_val {
                result.set_sent_by(v);
            }
            if let Some(v) = participants_val {
                result.set_participants(v);
            }
            if let Some(v) = request_status_val {
                result.set_request_status(v);
            }
            if let Some(v) = use_default_alerts_val {
                result.set_use_default_alerts(v);
            }
            if let Some(v) = alerts_val {
                result.set_alerts(v);
            }
            if let Some(v) = localizations_val {
                result.set_localizations(v);
            }
            if let Some(v) = time_zone_val {
                result.set_time_zone(v);
            }
            if let Some(v) = time_zones_val {
                result.set_time_zones(v);
            }
            for (k, v) in vendor_parts {
                result.insert_vendor_property(k, v);
            }
            Ok(result)
    }
}

// ============================================================================
// Group TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for Group<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let obj = value
            .try_into_object()
            .map_err(TypeErrorOr::from)
            .map_err(DocumentError::root)?;

        let mut entries_val: Option<Vec<TaskOrEvent<V>>> = None;
        let mut source_val: Option<Box<Uri>> = None;
        let mut uid_val: Option<Box<Uid>> = None;
        let mut prod_id_val: Option<String> = None;
        let mut created_val: Option<DateTime<Utc>> = None;
        let mut updated_val: Option<DateTime<Utc>> = None;
        let mut title_val: Option<String> = None;
        let mut description_val: Option<String> = None;
        let mut description_content_type_val: Option<String> = None;
        let mut links_val: Option<HashMap<Box<Id>, Link<V>>> = None;
        let mut locale_val: Option<LanguageTag> = None;
        let mut keywords_val: Option<HashSet<String>> = None;
        let mut categories_val: Option<HashSet<String>> = None;
        let mut color_val: Option<Color> = None;
        let mut time_zones_val: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>> = None;
        let mut vendor_parts: Vec<(Box<str>, V)> = Vec::new();

        for (key, val) in obj.into_iter() {
            let k = <V::Object as JsonObject>::key_into_string(key);
            match k.as_str() {
                "@type" => {}
                "entries" => {
                    entries_val = Some(
                        parse_vec(val, TaskOrEvent::try_from_json)
                            .map_err(|e| prepend("entries", e))?,
                    );
                }
                "source" => {
                    source_val =
                        Some(Box::<Uri>::try_from_json(val).map_err(|e| field_err("source", e))?);
                }
                "uid" => {
                    uid_val =
                        Some(Box::<Uid>::try_from_json(val).map_err(|e| field_err("uid", e))?);
                }
                "prodId" => {
                    prod_id_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("prodId", e))?);
                }
                "created" => {
                    created_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("created", e))?,
                    );
                }
                "updated" => {
                    updated_val = Some(
                        DateTime::<Utc>::try_from_json(val).map_err(|e| field_err("updated", e))?,
                    );
                }
                "title" => {
                    title_val =
                        Some(String::try_from_json(val).map_err(|e| type_field_err("title", e))?);
                }
                "description" => {
                    description_val = Some(
                        String::try_from_json(val).map_err(|e| type_field_err("description", e))?,
                    );
                }
                "descriptionContentType" => {
                    description_content_type_val = Some(
                        String::try_from_json(val)
                            .map_err(|e| type_field_err("descriptionContentType", e))?,
                    );
                }
                "links" => {
                    links_val = Some(
                        parse_id_map(val, Link::try_from_json).map_err(|e| prepend("links", e))?,
                    );
                }
                "locale" => {
                    locale_val =
                        Some(LanguageTag::try_from_json(val).map_err(|e| field_err("locale", e))?);
                }
                "keywords" => {
                    keywords_val = Some(
                        HashSet::<String>::try_from_json(val)
                            .map_err(|e| doc_field_err("keywords", e))?,
                    );
                }
                "categories" => {
                    categories_val = Some(
                        HashSet::<String>::try_from_json(val)
                            .map_err(|e| doc_field_err("categories", e))?,
                    );
                }
                "color" => {
                    color_val = Some(Color::try_from_json(val).map_err(|e| field_err("color", e))?);
                }
                "timeZones" => {
                    time_zones_val = Some(
                        parse_tz_map(val, TimeZone::try_from_json)
                            .map_err(|e| prepend("timeZones", e))?,
                    );
                }
                _ => vendor_parts.push((k.into_boxed_str(), val)),
            }
        }

        let entries = entries_val.unwrap_or_default();
        let uid = uid_val.ok_or_else(|| missing("uid"))?;
        let mut result = Group::new(entries, uid);
        if let Some(v) = source_val {
            result.set_source(v);
        }
        if let Some(v) = prod_id_val {
            result.set_prod_id(v);
        }
        if let Some(v) = created_val {
            result.set_created(v);
        }
        if let Some(v) = updated_val {
            result.set_updated(v);
        }
        if let Some(v) = title_val {
            result.set_title(v);
        }
        if let Some(v) = description_val {
            result.set_description(v);
        }
        if let Some(v) = description_content_type_val {
            result.set_description_content_type(v);
        }
        if let Some(v) = links_val {
            result.set_links(v);
        }
        if let Some(v) = locale_val {
            result.set_locale(v);
        }
        if let Some(v) = keywords_val {
            result.set_keywords(v);
        }
        if let Some(v) = categories_val {
            result.set_categories(v);
        }
        if let Some(v) = color_val {
            result.set_color(v);
        }
        if let Some(v) = time_zones_val {
            result.set_time_zones(v);
        }
        for (k, v) in vendor_parts {
            result.insert_vendor_property(k, v);
        }
        Ok(result)
    }
}

// ============================================================================
// TaskOrEvent TryFromJson
// ============================================================================

impl<V: DestructibleJsonValue> TryFromJson<V> for TaskOrEvent<V> {
    type Error = ObjErr;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let is_event = {
            let obj = value
                .try_as_object()
                .map_err(TypeErrorOr::from)
                .map_err(DocumentError::root)?;
            match obj.get("@type").and_then(|v| v.try_as_string().ok()) {
                Some(s) if s.as_ref() == "Event" => true,
                Some(s) if s.as_ref() == "Task" => false,
                _ => return Err(missing("@type")),
            }
        };

        if is_event {
            Event::try_from_json(value).map(TaskOrEvent::Event)
        } else {
            Task::try_from_json(value).map(TaskOrEvent::Task)
        }
    }
}

// ============================================================================
// IntoJson implementations
// ============================================================================

/// Helper: insert an optional field into a JSON object, skipping if None.
macro_rules! insert_optional {
    ($obj:expr, $key:expr, $val:expr) => {
        if let Some(v) = $val {
            $obj.insert($key.to_string(), v.into_json());
        }
    };
}

/// Helper: insert a required field into a JSON object.
macro_rules! insert_required {
    ($obj:expr, $key:expr, $val:expr) => {
        $obj.insert($key.to_string(), $val.into_json());
    };
}

/// Helper: insert vendor properties (consuming) into a JSON object.
macro_rules! insert_vendor_properties {
    ($obj:expr, $fields:expr) => {
        for (key, value) in $fields.drain_vendor_property() {
            $obj.insert(String::from(key), value);
        }
    };
}

impl<V: ConstructibleJsonValue> IntoJson<V> for UtcOffset {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for StatusCode {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for RequestStatus {
    fn into_json(self) -> V {
        V::string(self.to_string())
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for PatchObject<V> {
    fn into_json(self) -> V {
        let inner = self.into_inner();
        let mut obj = V::Object::with_capacity(inner.len());
        for (key, value) in inner {
            obj.insert(key.to_string(), value);
        }
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Relation<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Relation"));
        if let Some(relations) = f.take_relations()
            && !relations.is_empty()
        {
            insert_required!(obj, "relation", relations);
        }
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for OffsetTrigger<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("OffsetTrigger"));
        insert_required!(obj, "offset", f.take_offset().unwrap());
        insert_optional!(obj, "relativeTo", f.take_relative_to());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for AbsoluteTrigger<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("AbsoluteTrigger"));
        insert_required!(obj, "when", f.take_when().unwrap());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Trigger<V> {
    fn into_json(self) -> V {
        match self {
            Trigger::Offset(t) => t.into_json(),
            Trigger::Absolute(t) => t.into_json(),
            Trigger::Unknown(obj) => V::object(obj),
        }
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for ReplyTo {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        insert_optional!(obj, "imip", f.take_imip());
        insert_optional!(obj, "web", f.take_web());
        for (key, value) in f.drain_other() {
            obj.insert(key.as_str().to_owned(), value.into_json());
        }
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for SendToParticipant {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        insert_optional!(obj, "imip", f.take_imip());
        for (key, value) in f.drain_other() {
            obj.insert(key.as_str().to_owned(), value.into_json());
        }
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Link<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Link"));
        insert_required!(obj, "href", f.take_href().unwrap());
        insert_optional!(obj, "contentId", f.take_content_id());
        insert_optional!(obj, "mediaType", f.take_media_type());
        insert_optional!(obj, "size", f.take_size());
        if let Some(rel) = f.take_relation() {
            obj.insert("rel".to_string(), V::string(rel.to_string()));
        }
        insert_optional!(obj, "display", f.take_display());
        insert_optional!(obj, "title", f.take_title());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Location<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Location"));
        insert_optional!(obj, "name", f.take_name());
        insert_optional!(obj, "description", f.take_description());
        insert_optional!(obj, "locationTypes", f.take_location_types());
        insert_optional!(obj, "relativeTo", f.take_relative_to());
        insert_optional!(obj, "timeZone", f.take_time_zone());
        insert_optional!(obj, "coordinates", f.take_coordinates());
        insert_optional!(obj, "links", f.take_links());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for VirtualLocation<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("VirtualLocation"));
        insert_optional!(obj, "name", f.take_name());
        insert_optional!(obj, "description", f.take_description());
        insert_required!(obj, "uri", f.take_uri().unwrap());
        insert_optional!(obj, "features", f.take_features());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Alert<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Alert"));
        insert_required!(obj, "trigger", f.take_trigger().unwrap());
        insert_optional!(obj, "acknowledged", f.take_acknowledged());
        insert_optional!(obj, "relatedTo", f.take_related_to());
        insert_optional!(obj, "action", f.take_action());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for TimeZoneRule<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("TimeZoneRule"));
        insert_required!(obj, "start", f.take_start().unwrap());
        insert_required!(obj, "offsetFrom", f.take_offset_from().unwrap());
        insert_required!(obj, "offsetTo", f.take_offset_to().unwrap());
        insert_optional!(obj, "recurrenceRules", f.take_recurrence_rules());
        insert_optional!(obj, "recurrenceOverrides", f.take_recurrence_overrides());
        insert_optional!(obj, "names", f.take_names());
        insert_optional!(obj, "comments", f.take_comments());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for TimeZone<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("TimeZone"));
        insert_required!(obj, "tzId", f.take_tz_id().unwrap());
        insert_optional!(obj, "updated", f.take_updated());
        insert_optional!(obj, "url", f.take_url());
        insert_optional!(obj, "validUntil", f.take_valid_until());
        insert_optional!(obj, "aliases", f.take_aliases());
        insert_optional!(obj, "standard", f.take_standard());
        insert_optional!(obj, "daylight", f.take_daylight());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

fn serialize_participant_fields<V: ConstructibleJsonValue>(
    obj: &mut V::Object,
    f: &mut ParticipantFields<V>,
) {
    insert_optional!(obj, "name", f.take_name());
    insert_optional!(obj, "email", f.take_email());
    insert_optional!(obj, "description", f.take_description());
    insert_optional!(obj, "sendTo", f.take_send_to());
    insert_optional!(obj, "kind", f.take_kind());
    insert_optional!(obj, "roles", f.take_roles());
    insert_optional!(obj, "locationId", f.take_location_id());
    insert_optional!(obj, "language", f.take_language());
    insert_optional!(obj, "participationStatus", f.take_participation_status());
    insert_optional!(obj, "participationComment", f.take_participation_comment());
    insert_optional!(obj, "expectReply", f.take_expect_reply());
    insert_optional!(obj, "scheduleAgent", f.take_schedule_agent());
    insert_optional!(obj, "scheduleForceSend", f.take_schedule_force_send());
    insert_optional!(obj, "scheduleSequence", f.take_schedule_sequence());
    insert_optional!(obj, "scheduleStatus", f.take_schedule_status());
    insert_optional!(obj, "scheduleUpdated", f.take_schedule_updated());
    insert_optional!(obj, "sentBy", f.take_sent_by());
    insert_optional!(obj, "invitedBy", f.take_invited_by());
    insert_optional!(obj, "delegatedTo", f.take_delegated_to());
    insert_optional!(obj, "delegatedFrom", f.take_delegated_from());
    insert_optional!(obj, "memberOf", f.take_member_of());
    insert_optional!(obj, "links", f.take_links());
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Participant<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Participant"));
        serialize_participant_fields::<V>(&mut obj, &mut f);
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for TaskParticipant<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Participant"));
        // Common participant fields
        insert_optional!(obj, "name", f.take_name());
        insert_optional!(obj, "email", f.take_email());
        insert_optional!(obj, "description", f.take_description());
        insert_optional!(obj, "sendTo", f.take_send_to());
        insert_optional!(obj, "kind", f.take_kind());
        insert_optional!(obj, "roles", f.take_roles());
        insert_optional!(obj, "locationId", f.take_location_id());
        insert_optional!(obj, "language", f.take_language());
        insert_optional!(obj, "participationStatus", f.take_participation_status());
        insert_optional!(obj, "participationComment", f.take_participation_comment());
        insert_optional!(obj, "expectReply", f.take_expect_reply());
        insert_optional!(obj, "scheduleAgent", f.take_schedule_agent());
        insert_optional!(obj, "scheduleForceSend", f.take_schedule_force_send());
        insert_optional!(obj, "scheduleSequence", f.take_schedule_sequence());
        insert_optional!(obj, "scheduleStatus", f.take_schedule_status());
        insert_optional!(obj, "scheduleUpdated", f.take_schedule_updated());
        insert_optional!(obj, "sentBy", f.take_sent_by());
        insert_optional!(obj, "invitedBy", f.take_invited_by());
        insert_optional!(obj, "delegatedTo", f.take_delegated_to());
        insert_optional!(obj, "delegatedFrom", f.take_delegated_from());
        insert_optional!(obj, "memberOf", f.take_member_of());
        insert_optional!(obj, "links", f.take_links());
        // Task-specific fields
        insert_optional!(obj, "progress", f.take_progress());
        insert_optional!(obj, "progressUpdated", f.take_progress_updated());
        insert_optional!(obj, "percentComplete", f.take_percent_complete());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Event<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Event"));
        insert_required!(obj, "uid", f.take_uid().unwrap());
        insert_required!(obj, "start", f.take_start().unwrap());
        insert_optional!(obj, "duration", f.take_duration());
        insert_optional!(obj, "status", f.take_status());
        insert_optional!(obj, "relatedTo", f.take_related_to());
        insert_optional!(obj, "prodId", f.take_prod_id());
        insert_optional!(obj, "created", f.take_created());
        insert_optional!(obj, "updated", f.take_updated());
        insert_optional!(obj, "sequence", f.take_sequence());
        insert_optional!(obj, "method", f.take_method());
        insert_optional!(obj, "title", f.take_title());
        insert_optional!(obj, "description", f.take_description());
        insert_optional!(obj, "descriptionContentType", f.take_description_content_type());
        insert_optional!(obj, "showWithoutTime", f.take_show_without_time());
        insert_optional!(obj, "locations", f.take_locations());
        insert_optional!(obj, "virtualLocations", f.take_virtual_locations());
        insert_optional!(obj, "links", f.take_links());
        insert_optional!(obj, "locale", f.take_locale());
        insert_optional!(obj, "keywords", f.take_keywords());
        insert_optional!(obj, "categories", f.take_categories());
        insert_optional!(obj, "color", f.take_color());
        insert_optional!(obj, "recurrenceId", f.take_recurrence_id());
        insert_optional!(obj, "recurrenceIdTimeZone", f.take_recurrence_id_time_zone());
        insert_optional!(obj, "recurrenceRules", f.take_recurrence_rules());
        insert_optional!(obj, "excludedRecurrenceRules", f.take_excluded_recurrence_rules());
        insert_optional!(obj, "recurrenceOverrides", f.take_recurrence_overrides());
        insert_optional!(obj, "excluded", f.take_excluded());
        insert_optional!(obj, "priority", f.take_priority());
        insert_optional!(obj, "freeBusyStatus", f.take_free_busy_status());
        insert_optional!(obj, "privacy", f.take_privacy());
        insert_optional!(obj, "replyTo", f.take_reply_to());
        insert_optional!(obj, "sentBy", f.take_sent_by());
        insert_optional!(obj, "participants", f.take_participants());
        insert_optional!(obj, "requestStatus", f.take_request_status());
        insert_optional!(obj, "useDefaultAlerts", f.take_use_default_alerts());
        insert_optional!(obj, "alerts", f.take_alerts());
        insert_optional!(obj, "localizations", f.take_localizations());
        insert_optional!(obj, "timeZone", f.take_time_zone());
        insert_optional!(obj, "timeZones", f.take_time_zones());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Task<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Task"));
        insert_required!(obj, "uid", f.take_uid().unwrap());
        insert_optional!(obj, "due", f.take_due());
        insert_optional!(obj, "start", f.take_start());
        insert_optional!(obj, "estimatedDuration", f.take_estimated_duration());
        insert_optional!(obj, "percentComplete", f.take_percent_complete());
        insert_optional!(obj, "progress", f.take_progress());
        insert_optional!(obj, "progressUpdated", f.take_progress_updated());
        insert_optional!(obj, "relatedTo", f.take_related_to());
        insert_optional!(obj, "prodId", f.take_prod_id());
        insert_optional!(obj, "created", f.take_created());
        insert_optional!(obj, "updated", f.take_updated());
        insert_optional!(obj, "sequence", f.take_sequence());
        insert_optional!(obj, "method", f.take_method());
        insert_optional!(obj, "title", f.take_title());
        insert_optional!(obj, "description", f.take_description());
        insert_optional!(obj, "descriptionContentType", f.take_description_content_type());
        insert_optional!(obj, "showWithoutTime", f.take_show_without_time());
        insert_optional!(obj, "locations", f.take_locations());
        insert_optional!(obj, "virtualLocations", f.take_virtual_locations());
        insert_optional!(obj, "links", f.take_links());
        insert_optional!(obj, "locale", f.take_locale());
        insert_optional!(obj, "keywords", f.take_keywords());
        insert_optional!(obj, "categories", f.take_categories());
        insert_optional!(obj, "color", f.take_color());
        insert_optional!(obj, "recurrenceId", f.take_recurrence_id());
        insert_optional!(obj, "recurrenceIdTimeZone", f.take_recurrence_id_time_zone());
        insert_optional!(obj, "recurrenceRules", f.take_recurrence_rules());
        insert_optional!(obj, "excludedRecurrenceRules", f.take_excluded_recurrence_rules());
        insert_optional!(obj, "recurrenceOverrides", f.take_recurrence_overrides());
        insert_optional!(obj, "excluded", f.take_excluded());
        insert_optional!(obj, "priority", f.take_priority());
        insert_optional!(obj, "freeBusyStatus", f.take_free_busy_status());
        insert_optional!(obj, "privacy", f.take_privacy());
        insert_optional!(obj, "replyTo", f.take_reply_to());
        insert_optional!(obj, "sentBy", f.take_sent_by());
        insert_optional!(obj, "participants", f.take_participants());
        insert_optional!(obj, "requestStatus", f.take_request_status());
        insert_optional!(obj, "useDefaultAlerts", f.take_use_default_alerts());
        insert_optional!(obj, "alerts", f.take_alerts());
        insert_optional!(obj, "localizations", f.take_localizations());
        insert_optional!(obj, "timeZone", f.take_time_zone());
        insert_optional!(obj, "timeZones", f.take_time_zones());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for Group<V> {
    fn into_json(self) -> V {
        let mut f = self.into_fields();
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("Group"));
        insert_required!(obj, "uid", f.take_uid().unwrap());
        if let Some(entries) = f.take_entries()
            && !entries.is_empty()
        {
            insert_required!(obj, "entries", entries);
        }
        insert_optional!(obj, "source", f.take_source());
        insert_optional!(obj, "prodId", f.take_prod_id());
        insert_optional!(obj, "created", f.take_created());
        insert_optional!(obj, "updated", f.take_updated());
        insert_optional!(obj, "title", f.take_title());
        insert_optional!(obj, "description", f.take_description());
        insert_optional!(obj, "descriptionContentType", f.take_description_content_type());
        insert_optional!(obj, "links", f.take_links());
        insert_optional!(obj, "locale", f.take_locale());
        insert_optional!(obj, "keywords", f.take_keywords());
        insert_optional!(obj, "categories", f.take_categories());
        insert_optional!(obj, "color", f.take_color());
        insert_optional!(obj, "timeZones", f.take_time_zones());
        insert_vendor_properties!(obj, f);
        V::object(obj)
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for TaskOrEvent<V> {
    fn into_json(self) -> V {
        match self {
            TaskOrEvent::Task(t) => t.into_json(),
            TaskOrEvent::Event(e) => e.into_json(),
        }
    }
}

// ============================================================================
// RRule IntoJson
// ============================================================================

fn weekday_code(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "mo",
        Weekday::Tuesday => "tu",
        Weekday::Wednesday => "we",
        Weekday::Thursday => "th",
        Weekday::Friday => "fr",
        Weekday::Saturday => "sa",
        Weekday::Sunday => "su",
    }
}

fn serialize_by_day<V: ConstructibleJsonValue>(set: &WeekdayNumSet) -> V {
    let mut arr = V::Array::with_capacity(set.len());
    for wdn in set.iter() {
        let mut day_obj = V::Object::new();
        day_obj.insert("@type".to_string(), V::str("NDay"));
        day_obj.insert("day".to_string(), V::str(weekday_code(wdn.weekday)));
        if let Some((sign, week)) = wdn.ordinal {
            let n = (sign as i64) * (week as i64);
            day_obj.insert("nthOfPeriod".to_string(), V::int(crate::json::Int::new(n).unwrap()));
        }
        arr.push(V::object(day_obj));
    }
    V::array(arr)
}

fn serialize_second_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::SecondSet) -> V {
    let mut arr = V::Array::new();
    for sec in rfc5545_types::rrule::Second::iter() {
        if set.get(sec) {
            arr.push(V::unsigned_int(UnsignedInt::new(sec as u64).unwrap()));
        }
    }
    V::array(arr)
}

fn serialize_minute_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::MinuteSet) -> V {
    let mut arr = V::Array::new();
    for min in rfc5545_types::rrule::Minute::iter() {
        if set.get(min) {
            arr.push(V::unsigned_int(UnsignedInt::new(min as u64).unwrap()));
        }
    }
    V::array(arr)
}

fn serialize_hour_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::HourSet) -> V {
    let mut arr = V::Array::new();
    for hr in rfc5545_types::rrule::Hour::iter() {
        if set.get(hr) {
            arr.push(V::unsigned_int(UnsignedInt::new(hr as u64).unwrap()));
        }
    }
    V::array(arr)
}

fn serialize_month_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::MonthSet) -> V {
    let mut arr = V::Array::new();
    for m in Month::iter() {
        if set.get(m) {
            arr.push(V::unsigned_int(UnsignedInt::new(m.number().get() as u64).unwrap()));
        }
    }
    V::array(arr)
}

fn serialize_month_day_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::MonthDaySet) -> V {
    use rfc5545_types::rrule::{MonthDay, MonthDaySetIndex};
    let mut arr = V::Array::new();
    // Positive days 1..=31
    for d in 1..=31u8 {
        if let Some(md) = MonthDay::from_repr(d) {
            let idx = MonthDaySetIndex::from_signed_month_day(Sign::Pos, md);
            if set.get(idx) {
                arr.push(V::int(crate::json::Int::new(d as i64).unwrap()));
            }
        }
    }
    // Negative days -31..=-1
    for d in 1..=31u8 {
        if let Some(md) = MonthDay::from_repr(d) {
            let idx = MonthDaySetIndex::from_signed_month_day(Sign::Neg, md);
            if set.get(idx) {
                arr.push(V::int(crate::json::Int::new(-(d as i64)).unwrap()));
            }
        }
    }
    V::array(arr)
}

fn serialize_year_day_nums<V: ConstructibleJsonValue>(set: &BTreeSet<rfc5545_types::rrule::YearDayNum>) -> V {
    let mut arr = V::Array::with_capacity(set.len());
    for ydn in set {
        // YearDayNum wraps a NonZero<i16>
        let n = ydn.get();
        arr.push(V::int(crate::json::Int::new(n as i64).unwrap()));
    }
    V::array(arr)
}

fn serialize_week_no_set<V: ConstructibleJsonValue>(set: &rfc5545_types::rrule::WeekNoSet) -> V {
    use rfc5545_types::rrule::WeekNoSetIndex;
    let mut arr = V::Array::new();
    // Positive weeks 1..=53
    for w in 1..=53u8 {
        if let Some(iw) = IsoWeek::from_index(w) {
            let idx = WeekNoSetIndex::from_signed_week(Sign::Pos, iw);
            if set.get(idx) {
                arr.push(V::int(crate::json::Int::new(w as i64).unwrap()));
            }
        }
    }
    // Negative weeks -53..=-1
    for w in 1..=53u8 {
        if let Some(iw) = IsoWeek::from_index(w) {
            let idx = WeekNoSetIndex::from_signed_week(Sign::Neg, iw);
            if set.get(idx) {
                arr.push(V::int(crate::json::Int::new(-(w as i64)).unwrap()));
            }
        }
    }
    V::array(arr)
}

fn serialize_date_or_datetime(dod: &DateTimeOrDate<Local>) -> String {
    match dod {
        DateTimeOrDate::DateTime(dt) => dt.to_string(),
        DateTimeOrDate::Date(d) => d.to_string(),
    }
}

impl<V: ConstructibleJsonValue> IntoJson<V> for RRule {
    fn into_json(self) -> V {
        let mut obj = V::Object::new();
        obj.insert("@type".to_string(), V::str("RecurrenceRule"));

        // Frequency and freq-dependent by-rules
        let (freq_str, by_month_day, by_year_day, by_week_no) = match self.freq {
            rfc5545_types::rrule::FreqByRules::Secondly(r) => {
                ("secondly", r.by_month_day, r.by_year_day, None)
            }
            rfc5545_types::rrule::FreqByRules::Minutely(r) => {
                ("minutely", r.by_month_day, r.by_year_day, None)
            }
            rfc5545_types::rrule::FreqByRules::Hourly(r) => {
                ("hourly", r.by_month_day, r.by_year_day, None)
            }
            rfc5545_types::rrule::FreqByRules::Daily(r) => ("daily", r.by_month_day, None, None),
            rfc5545_types::rrule::FreqByRules::Weekly => ("weekly", None, None, None),
            rfc5545_types::rrule::FreqByRules::Monthly(r) => {
                ("monthly", r.by_month_day, None, None)
            }
            rfc5545_types::rrule::FreqByRules::Yearly(r) => {
                ("yearly", r.by_month_day, r.by_year_day, r.by_week_no)
            }
        };

        obj.insert("frequency".to_string(), V::str(freq_str));

        if let Some(interval) = self.interval {
            obj.insert(
                "interval".to_string(),
                V::unsigned_int(UnsignedInt::new(interval.get().get()).unwrap()),
            );
        }

        match self.termination {
            Some(rfc5545_types::rrule::Termination::Count(c)) => {
                obj.insert(
                    "count".to_string(),
                    V::unsigned_int(UnsignedInt::new(c).unwrap()),
                );
            }
            Some(rfc5545_types::rrule::Termination::Until(ref u)) => {
                obj.insert("until".to_string(), V::string(serialize_date_or_datetime(u)));
            }
            None => {}
        }

        if let Some(ws) = self.week_start {
            obj.insert("firstDayOfWeek".to_string(), V::str(weekday_code(ws)));
        }

        // Core by-rules
        if let Some(ref set) = self.core_by_rules.by_second {
            obj.insert("bySecond".to_string(), serialize_second_set::<V>(set));
        }
        if let Some(ref set) = self.core_by_rules.by_minute {
            obj.insert("byMinute".to_string(), serialize_minute_set::<V>(set));
        }
        if let Some(ref set) = self.core_by_rules.by_hour {
            obj.insert("byHour".to_string(), serialize_hour_set::<V>(set));
        }
        if let Some(ref set) = self.core_by_rules.by_month {
            obj.insert("byMonth".to_string(), serialize_month_set::<V>(set));
        }
        if let Some(ref set) = self.core_by_rules.by_day {
            obj.insert("byDay".to_string(), serialize_by_day::<V>(set));
        }
        if let Some(ref set) = self.core_by_rules.by_set_pos {
            obj.insert("bySetPosition".to_string(), serialize_year_day_nums::<V>(set));
        }

        // Freq-dependent by-rules
        if let Some(ref set) = by_month_day {
            obj.insert("byMonthDay".to_string(), serialize_month_day_set::<V>(set));
        }
        if let Some(ref set) = by_year_day {
            obj.insert("byYearDay".to_string(), serialize_year_day_nums::<V>(set));
        }
        if let Some(ref set) = by_week_no {
            obj.insert("byWeekNo".to_string(), serialize_week_no_set::<V>(set));
        }

        V::object(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde_json")]
    #[test]
    fn path_object_from_serde_json() {
        use serde_json::{Value, json};

        let input = json!({
            "foo/bar" : null,
            "baz/12/bar" : {},
        });

        assert!(PatchObject::<Value>::try_from_json(input).is_ok());

        let input = json!({
            "foo/bar" : null,
            "baz/12/bar" : {},
            "/foo" : true, // invalid because this pointer begins with a forward slash
        });

        assert_eq!(
            PatchObject::try_from_json(input),
            Err(TypeErrorOr::Other(InvalidPatchObjectError {
                key: "/foo".into(),
                error: InvalidImplicitJsonPointerError::Explicit
            }))
        );

        let input = json!({
            "foo/bar" : null,
            "baz/12/bar" : {},
            "abc~" : true, // invalid because this contains a bare tilde
        });

        assert_eq!(
            PatchObject::try_from_json(input),
            Err(TypeErrorOr::Other(InvalidPatchObjectError {
                key: "abc~".into(),
                error: InvalidImplicitJsonPointerError::BareTilde { index: 3 }
            }))
        );
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn link_from_serde_json() {
        use serde_json::json;

        let input = json!({
            "@type": "Link",
            "href": "https://example.com/file.pdf",
            "mediaType": "application/pdf",
            "title": "The Specification",
            "size": 42000,
        });

        let link = Link::try_from_json(input).expect("valid link");
        assert!(link.title().is_some());
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn event_from_serde_json() {
        use serde_json::json;

        let input = json!({
            "@type": "Event",
            "uid": "test-event-uid-1",
            "start": "2024-01-15T09:00:00",
            "title": "Team Meeting",
            "duration": "PT1H",
        });

        let event = Event::try_from_json(input).expect("valid event");
        assert!(event.title().is_some());
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn task_or_event_dispatch() {
        use serde_json::json;

        let event_input = json!({
            "@type": "Event",
            "uid": "event-1",
            "start": "2024-03-01T10:00:00",
        });

        let task_input = json!({
            "@type": "Task",
            "uid": "task-1",
        });

        let toe1 = TaskOrEvent::try_from_json(event_input).expect("valid event");
        let toe2 = TaskOrEvent::try_from_json(task_input).expect("valid task");

        assert!(matches!(toe1, TaskOrEvent::Event(_)));
        assert!(matches!(toe2, TaskOrEvent::Task(_)));
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn missing_required_field_error() {
        use serde_json::json;

        // Event missing uid
        let input = json!({ "@type": "Event", "start": "2024-01-01T00:00:00" });
        let err = Event::try_from_json(input).unwrap_err();
        assert!(matches!(
            err.error,
            TypeErrorOr::Other(ObjectFromJsonError::MissingField("uid"))
        ));

        // Link missing href
        let input = json!({ "@type": "Link", "title": "test" });
        let err = Link::try_from_json(input).unwrap_err();
        assert!(matches!(
            err.error,
            TypeErrorOr::Other(ObjectFromJsonError::MissingField("href"))
        ));
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn wrong_type_field_error() {
        use serde_json::json;

        // Event uid is not a string
        let input = json!({ "@type": "Event", "uid": 123, "start": "2024-01-01T00:00:00" });
        let err = Event::try_from_json(input).unwrap_err();
        assert!(matches!(err.error, TypeErrorOr::TypeError(_)));
        assert_eq!(err.path.front(), Some(&PathSegment::Static("uid")));
    }
}
