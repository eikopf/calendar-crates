//! Distinguished object types.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use structible::structible;

use crate::{
    json::{JsonValue, UnsignedInt},
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
            ImplicitJsonPointer, LanguageTag, MediaType, Uid, Uri, VendorStr,
        },
        time::{DateTime, Duration, Local, SignedDuration, Utc, UtcOffset},
    },
};

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
    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// A [`Task`] or an [`Event`].
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
    #[structible(key = Box<VendorStr>)]
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
    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// A description of a virtual location (RFC 8984 §4.2.6).
#[structible]
pub struct VirtualLocation<V> {
    pub name: Option<String>,
    pub description: Option<String>,
    pub uri: Box<Uri>,
    pub features: Option<HashSet<Token<VirtualLocationFeature>>>,

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
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

    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// The trigger of an [`Alert`].
#[derive(PartialEq)]
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

    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// A trigger defined at an absolute time (RFC 8984 §4.5.2).
#[structible]
pub struct AbsoluteTrigger<V> {
    pub when: DateTime<Utc>,

    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// A set of relationship types (RFC 8984 §1.4.10).
#[structible]
pub struct Relation<V> {
    pub relations: HashSet<Token<RelationValue>>,

    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

/// A set of patches to be applied to a JSON object (RFC 8984 §1.4.9).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchObject<V>(HashMap<Box<ImplicitJsonPointer>, V>);
