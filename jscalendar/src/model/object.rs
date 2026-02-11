//! Distinguished object types.

use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hash, RandomState},
};

use hashbrown::HashTable;

use crate::{
    json::{Int, UnsignedInt},
    model::{
        set::RelationValue,
        string::{Id, ImplicitJsonPointer, VendorStr},
        time::{DateTime, Duration, Local, Utc},
    },
};

/// A set of patches to be applied to a JSON object (RFC 8984 §1.4.9).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchObject<V>(HashMap<Box<ImplicitJsonPointer>, V>);

/// A set of relationship types (RFC 8984 §1.4.10).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Relation {
    relations: HashSet<RelationValue<Box<VendorStr>>>,
}

/// A link to an external resource (RFC 8984 §1.4.11).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Link {
    uri: String,
    content_id: Option<String>,
    media_type: Option<String>,
    size: Option<UnsignedInt>,
    relation: Option<String>,
    display: Option<String>,
    title: Option<String>,
}

struct JsCalendarObject<V, S = RandomState> {
    properties: HashTable<Property<V>>,
    hasher: S,
}

impl<V, S: BuildHasher> JsCalendarObject<V, S> {
    fn hash_key<K: Hash + ?Sized>(hasher: &S, key: &K) -> u64 {
        hasher.hash_one(key)
    }

    fn get(&self, key: PropertyName<&VendorStr>) -> Option<&Property<V>> {
        let hash = Self::hash_key(&self.hasher, &key);
        self.properties.find(hash, |p| p.as_name() == key)
    }

    fn get_mut(&mut self, key: PropertyName<&VendorStr>) -> Option<&mut Property<V>> {
        let hash = Self::hash_key(&self.hasher, &key);
        self.properties.find_mut(hash, |p| p.as_name() == key)
    }

    fn remove(&mut self, key: PropertyName<&VendorStr>) -> Option<Property<V>> {
        let hash = Self::hash_key(&self.hasher, &key);
        match self.properties.find_entry(hash, |p| p.as_name() == key) {
            Ok(entry) => Some(entry.remove().0),
            Err(_) => None,
        }
    }
}

/// A JSCalendar property, excluding `@type`.
#[derive(Debug, Clone, PartialEq)]
enum Property<V> {
    /// A vendor-specific property (RFC 8984 §3.3).
    Vendor {
        name: Box<VendorStr>,
        value: V,
    },
    // RFC 8984 §4.1
    Uid(String),
    RelatedTo(HashMap<String, Relation>),
    ProductId(String),
    Created(DateTime<Utc>),
    Updated(DateTime<Utc>),
    Sequence(UnsignedInt),
    Method(String),
    // RFC 8984 §4.2
    Title(String),
    Description(String),
    DescriptionContentType(String),
    ShowWithoutTime(bool),
    Locations(HashMap<Box<Id>, ()>), // HashMap<Box<Id>, Location>
    VirtualLocations(HashMap<Box<Id>, ()>), // HashMap<Box<Id>, VirtualLocation>
    Links(HashMap<Box<Id>, Link>),
    Locale(String),
    Keywords(HashSet<String>),
    Categories(HashSet<String>),
    Color(String),
    // RFC 8984 §4.3
    RecurrenceId(DateTime<Local>),
    RecurrenceIdTimeZone(String),     // optional TimeZoneId
    RecurrenceRules(Vec<()>),         // Vec<RecurrenceRule>
    ExcludedRecurrenceRules(Vec<()>), // Vec<RecurrenceRule>
    RecurrenceOverrides(HashMap<DateTime<Local>, PatchObject<V>>),
    Excluded(bool),
    // RFC 8984 §4.4
    Priority(Int),
    FreeBusyStatus(String),
    Privacy(String),
    ReplyTo(HashMap<String, String>),
    SentBy(String),
    Participants(HashMap<Box<Id>, ()>), // HashMap<Box<Id>, Participant>
    RequestStatus(String),
    // RFC 8984 §4.5
    UseDefaultAlerts(bool),
    Alerts(HashMap<Box<Id>, ()>), // HashMap<Box<Id>, Alert>
    // RFC 8984 §4.6
    Localizations(HashMap<String, PatchObject<V>>),
    // RFC 8984 §4.7
    TimeZone(String),               // optional TimeZoneId
    TimeZones(HashMap<String, ()>), // HashMap<TimeZoneId, TimeZone>
    // RFC 8984 §5
    /// RFC 8984 §5.1.1, §5.2.2
    Start(DateTime<Local>),
    /// RFC 8984 §5.1.2
    Duration(Duration),
    /// RFC 8984 §5.1.3
    Status(String), // essentially EventStatus from RFC 5545
    /// RFC 8984 §5.2.1
    Due(DateTime<Local>),
    /// RFC 8984 §5.2.3
    EstimatedDuration(Duration),
    /// RFC 8984 §5.2.4
    PercentComplete(UnsignedInt),
    /// RFC 8984 §5.2.5
    Progress(String), // essentially TodoStatus from RFC 5545
    /// RFC 8984 §5.2.6
    ProgressUpdated(DateTime<Utc>),
    /// RFC 8984 §5.3.1
    Entries(Vec<()>), // Vec<TaskOrEvent>
    /// RFC 8984 §5.3.2
    Source(String),
}

impl<V> Property<V> {
    pub fn as_name(&self) -> PropertyName<&VendorStr> {
        match self {
            Property::Vendor { name, .. } => PropertyName::Vendor(name),
            Property::Uid(_) => PropertyName::Uid,
            Property::RelatedTo(_) => PropertyName::RelatedTo,
            Property::ProductId(_) => PropertyName::ProductId,
            Property::Created(_) => PropertyName::Created,
            Property::Updated(_) => PropertyName::Updated,
            Property::Sequence(_) => PropertyName::Sequence,
            Property::Method(_) => PropertyName::Method,
            Property::Title(_) => PropertyName::Title,
            Property::Description(_) => PropertyName::Description,
            Property::DescriptionContentType(_) => PropertyName::DescriptionContentType,
            Property::ShowWithoutTime(_) => PropertyName::ShowWithoutTime,
            Property::Locations(_) => PropertyName::Locations,
            Property::VirtualLocations(_) => PropertyName::VirtualLocations,
            Property::Links(_) => PropertyName::Links,
            Property::Locale(_) => PropertyName::Locale,
            Property::Keywords(_) => PropertyName::Keywords,
            Property::Categories(_) => PropertyName::Categories,
            Property::Color(_) => PropertyName::Color,
            Property::RecurrenceId(_) => PropertyName::RecurrenceId,
            Property::RecurrenceIdTimeZone(_) => PropertyName::RecurrenceIdTimeZone,
            Property::RecurrenceRules(_) => PropertyName::RecurrenceRules,
            Property::ExcludedRecurrenceRules(_) => PropertyName::ExcludedRecurrenceRules,
            Property::RecurrenceOverrides(_) => PropertyName::RecurrenceOverrides,
            Property::Excluded(_) => PropertyName::Excluded,
            Property::Priority(_) => PropertyName::Priority,
            Property::FreeBusyStatus(_) => PropertyName::FreeBusyStatus,
            Property::Privacy(_) => PropertyName::Privacy,
            Property::ReplyTo(_) => PropertyName::ReplyTo,
            Property::SentBy(_) => PropertyName::SentBy,
            Property::Participants(_) => PropertyName::Participants,
            Property::RequestStatus(_) => PropertyName::RequestStatus,
            Property::UseDefaultAlerts(_) => PropertyName::UseDefaultAlerts,
            Property::Alerts(_) => PropertyName::Alerts,
            Property::Localizations(_) => PropertyName::Localizations,
            Property::TimeZone(_) => PropertyName::TimeZone,
            Property::TimeZones(_) => PropertyName::TimeZones,
            Property::Start(_) => PropertyName::Start,
            Property::Duration(_) => PropertyName::Duration,
            Property::Status(_) => PropertyName::Status,
            Property::Due(_) => PropertyName::Due,
            Property::EstimatedDuration(_) => PropertyName::EstimatedDuration,
            Property::PercentComplete(_) => PropertyName::PercentComplete,
            Property::Progress(_) => PropertyName::Progress,
            Property::ProgressUpdated(_) => PropertyName::ProgressUpdated,
            Property::Entries(_) => PropertyName::Entries,
            Property::Source(_) => PropertyName::Source,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PropertyName<S> {
    Vendor(S),
    Uid,
    RelatedTo,
    ProductId,
    Created,
    Updated,
    Sequence,
    Method,
    Title,
    Description,
    DescriptionContentType,
    ShowWithoutTime,
    Locations,
    VirtualLocations,
    Links,
    Locale,
    Keywords,
    Categories,
    Color,
    RecurrenceId,
    RecurrenceIdTimeZone,
    RecurrenceRules,
    ExcludedRecurrenceRules,
    RecurrenceOverrides,
    Excluded,
    Priority,
    FreeBusyStatus,
    Privacy,
    ReplyTo,
    SentBy,
    Participants,
    RequestStatus,
    UseDefaultAlerts,
    Alerts,
    Localizations,
    TimeZone,
    TimeZones,
    Start,
    Duration,
    Status,
    Due,
    EstimatedDuration,
    PercentComplete,
    Progress,
    ProgressUpdated,
    Entries,
    Source,
}
