//! iCalendar properties.

use super::{
    parameter::{Params, StructuredDataParams},
    primitive::{DateTime, DateTimeOrDate, SignedDuration, Utc},
    string::Uri,
};

/// A property generic over values and parameters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Prop<V, P> {
    pub params: P,
    pub value: V,
}

impl<V, P> Prop<V, P> {
    pub fn from_value(value: V) -> Self
    where
        P: Default,
    {
        Self {
            value,
            params: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTerminationRef<'a> {
    End(&'a Prop<DateTimeOrDate, Params>),
    Duration(&'a Prop<SignedDuration, Params>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventTerminationMut<'a> {
    End(&'a mut Prop<DateTimeOrDate, Params>),
    Duration(&'a mut Prop<SignedDuration, Params>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoTerminationRef<'a> {
    Due(&'a Prop<DateTimeOrDate, Params>),
    Duration(&'a Prop<SignedDuration, Params>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TodoTerminationMut<'a> {
    Due(&'a mut Prop<DateTimeOrDate, Params>),
    Duration(&'a mut Prop<SignedDuration, Params>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerPropRef<'a> {
    Relative(&'a Prop<SignedDuration, Params>),
    Absolute(&'a Prop<DateTime<Utc>, Params>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TriggerPropMut<'a> {
    Relative(&'a mut Prop<SignedDuration, Params>),
    Absolute(&'a mut Prop<DateTime<Utc>, Params>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredDataProp {
    Binary(Prop<Vec<u8>, StructuredDataParams>),
    Text(Prop<String, StructuredDataParams>),
    Uri(Prop<Box<Uri>, Params>),
}

/// Statically-known property names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StaticProp {
    // CALENDAR PROPERTIES
    CalScale,
    Method,
    ProdId,
    Version,
    // DESCRIPTIVE COMPONENT PROPERTIES
    Attach,
    Categories,
    Class,
    Comment,
    Description,
    Geo,
    Location,
    PercentComplete,
    Priority,
    Resources,
    Status,
    Summary,
    // DATE AND TIME COMPONENT PROPERTIES
    DtCompleted,
    DtEnd,
    DtDue,
    DtStart,
    Duration,
    FreeBusy,
    Transp,
    // TIME ZONE COMPONENT PROPERTIES
    TzId,
    TzName,
    TzOffsetFrom,
    TzOffsetTo,
    TzUrl,
    // RELATIONSHIP COMPONENT PROPERTIES
    Attendee,
    Contact,
    Organizer,
    RecurId,
    RelatedTo,
    Url,
    Uid,
    // RECURRENCE COMPONENT PROPERTIES
    ExDate,
    RDate,
    RRule,
    // ALARM COMPONENT PROPERTIES
    Action,
    Repeat,
    Trigger,
    // CHANGE MANAGEMENT COMPONENT PROPERTIES
    Created,
    DtStamp,
    LastModified,
    Sequence,
    // MISCELLANEOUS COMPONENT PROPERTIES
    RequestStatus,
    // RFC 7986 PROPERTIES
    Name,
    RefreshInterval,
    Source,
    Color,
    Image,
    Conference,
    // RFC 9073 PROPERTIES
    LocationType,
    ParticipantType,
    ResourceType,
    CalendarAddress,
    StyledDescription,
    StructuredData,
    // RFC 9074 PROPERTIES
    Acknowledged,
    Proximity,
}
