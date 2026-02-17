//! Distinguished object types.
//!
//! # TODO
//!
//! - `RequestStatus`: a structured request status value (RFC 8984 §4.4.7).
//! - `Participant`: a calendar participant (RFC 8984 §4.4.6).
//! - `Alert`: a calendar alert (RFC 8984 §4.5.2).
//! - `AbsoluteTrigger`, `OffsetTrigger`, `UnknownTrigger`: trigger types for alerts (RFC 8984 §4.5.2).

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use structible::structible;

use crate::{
    json::UnsignedInt,
    model::{
        rrule::RRule,
        set::{
            Color, EventStatus, FreeBusyStatus, Method, Percent, Priority, Privacy, RelationValue,
            ReplyMethod, TaskProgress,
        },
        string::{
            CalAddress, CustomTimeZoneId, Id, ImplicitJsonPointer, LanguageTag, Uid, Uri, VendorStr,
        },
        time::{DateTime, Duration, Local, Utc, UtcOffset},
    },
};

#[structible]
pub struct Group<V> {
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
    pub links: Option<HashMap<Box<Id>, Link>>,
    pub locale: Option<LanguageTag>,
    pub keywords: Option<HashSet<String>>,
    pub categories: Option<HashSet<String>>,
    pub color: Option<Color>,
    pub time_zones: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>>,

    // Custom vendor properties (RFC 8984 §3.3)
    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskOrEvent<V> {
    Task(Task<V>),
    Event(Event<V>),
}

#[structible]
pub struct Event<V> {
    // Event Properties (RFC 8984 §5.1)
    pub start: DateTime<Local>,
    pub duration: Option<Duration>,
    pub status: Option<EventStatus<Box<VendorStr>>>,

    // Metadata Properties (RFC 8984 §4.1)
    pub uid: Box<Uid>,
    pub related_to: Option<HashMap<Box<Uid>, Relation>>,
    pub prod_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub sequence: Option<UnsignedInt>,
    pub method: Option<Method<Box<VendorStr>>>,

    // What and Where Properties (RFC 8984 §4.2)
    pub title: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub show_without_time: Option<bool>,
    pub locations: Option<HashMap<Box<Id>, Location>>,
    pub virtual_locations: Option<HashMap<Box<Id>, VirtualLocation>>,
    pub links: Option<HashMap<Box<Id>, Link>>,
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
    pub free_busy_status: Option<FreeBusyStatus<Box<VendorStr>>>,
    pub privacy: Option<Privacy<Box<VendorStr>>>,
    pub reply_to: Option<HashMap<ReplyMethod<Box<VendorStr>>, Box<Uri>>>,
    pub sent_by: Option<Box<CalAddress>>,
    pub participants: Option<HashMap<Box<Id>, ()>>, // HashMap<Box<Id>, Participant>
    pub request_status: Option<String>,

    // Alerts Properties (RFC 8984 §4.5)
    pub use_default_alerts: Option<bool>,
    pub alerts: Option<HashMap<Box<Id>, ()>>, // HashMap<Box<Id>, Alert>

    // Multilingual Properties (RFC 8984 §4.6)
    pub localizations: Option<HashMap<LanguageTag, PatchObject<V>>>,

    // Time Zone Properties (RFC 8984 §4.7)
    pub time_zone: Option<String>,
    pub time_zones: Option<HashMap<Box<CustomTimeZoneId>, TimeZone<V>>>,

    // Custom vendor properties (RFC 8984 §3.3)
    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

#[structible]
pub struct Task<V> {
    // Task Properties (RFC 8984 §5.2)
    pub due: Option<DateTime<Local>>,
    pub start: Option<DateTime<Local>>,
    pub estimated_duration: Option<Duration>,
    pub percent_complete: Option<Percent>,
    pub progress: Option<TaskProgress<Box<VendorStr>>>,
    pub progress_updated: Option<DateTime<Utc>>,

    // Metadata Properties (RFC 8984 §4.1)
    pub uid: Box<Uid>,
    pub related_to: Option<HashMap<Box<Uid>, Relation>>,
    pub prod_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub sequence: Option<UnsignedInt>,
    pub method: Option<Method<Box<VendorStr>>>,

    // What and Where Properties (RFC 8984 §4.2)
    pub title: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub show_without_time: Option<bool>,
    pub locations: Option<HashMap<Box<Id>, Location>>,
    pub virtual_locations: Option<HashMap<Box<Id>, VirtualLocation>>,
    pub links: Option<HashMap<Box<Id>, Link>>,
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
    pub free_busy_status: Option<FreeBusyStatus<Box<VendorStr>>>,
    pub privacy: Option<Privacy<Box<VendorStr>>>,
    pub reply_to: Option<HashMap<ReplyMethod<Box<VendorStr>>, Box<Uri>>>,
    pub sent_by: Option<Box<CalAddress>>,
    pub participants: Option<HashMap<Box<Id>, ()>>, // HashMap<Box<Id>, Participant>
    pub request_status: Option<String>,

    // Alerts Properties (RFC 8984 §4.5)
    pub use_default_alerts: Option<bool>,
    pub alerts: Option<HashMap<Box<Id>, ()>>, // HashMap<Box<Id>, Alert>

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
pub struct Location {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_types: Option<HashSet<String>>, // HashSet<LocationType<_>> (RFC 4589)
    pub relative_to: Option<RelationValue<Box<VendorStr>>>,
    pub time_zone: Option<String>,
    pub coordinates: Option<String>, // Box<GeoUri> (RFC 5870)
    pub links: Option<HashMap<Box<Id>, Link>>,
}

/// A description of a virutal location (RFC 8984 §4.2.6).
#[structible]
pub struct VirtualLocation {
    pub name: Option<String>,
    pub description: Option<String>,
    pub uri: Box<Uri>,
    pub features: Option<HashSet<String>>, // HashSet<VirtualLocationFeature<_>>
}

/// A link to an external resource (RFC 8984 §1.4.11).
#[structible]
pub struct Link {
    pub href: Box<Uri>,
    pub content_id: Option<String>, // Box<ContentId> (RFC 2392 §2)
    pub media_type: Option<String>, // MediaType (RFC 6838)
    pub size: Option<UnsignedInt>,
    pub relation: Option<String>, // RelationType<_> (RFC 8288)
    pub display: Option<String>,  // DisplayValue<_>
    pub title: Option<String>,
}

/// A description of a time zone (RFC 8984 §4.7.2).
#[structible]
pub struct TimeZone<V> {
    pub tz_id: String, // Box<ParamText> (RFC 5545 §3.1)
    pub updated: Option<DateTime<Utc>>,
    pub url: Option<Box<Uri>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub aliases: Option<HashSet<String>>, // TZID-ALIAS-OF (RFC 7808)
    pub standard: Option<Vec<TimeZoneRule<V>>>,
    pub daylight: Option<Vec<TimeZoneRule<V>>>,
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
    pub names: Option<HashSet<String>>, // RFC 5545, TZNAME
    pub comments: Option<Vec<String>>,  // RFC 5545, COMMENT
}

/// A set of patches to be applied to a JSON object (RFC 8984 §1.4.9).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchObject<V>(HashMap<Box<ImplicitJsonPointer>, V>);

/// A set of relationship types (RFC 8984 §1.4.10).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Relation {
    relations: HashSet<RelationValue<Box<VendorStr>>>,
}
