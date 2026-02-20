//! iCalendar properties.

use super::{
    css::Css3Color,
    parameter::{Params, StructuredDataParams},
    primitive::{
        AlarmAction, Attachment, ClassValue, CompletionPercentage, Date, DateTime, DateTimeOrDate,
        Geo, Gregorian, Integer, Method, ParticipantType, Period, PositiveInteger, Priority,
        ProximityValue, RDateSeq, RequestStatus, ResourceType, SignedDuration, Status,
        StyledDescriptionValue, TimeTransparency, Token, TriggerValue, Utc, UtcOffset, Value,
        Version,
    },
    rrule::RRule,
    string::{TzId, Uid, Uri},
};

use derive_more::{From, TryInto};
use strum::EnumDiscriminants;

type VP<V, P> = Vec<Prop<V, P>>;

/// A homogeneous sequence of `(value, params)` pairs whose types may range over a closed set. In
/// essence, this is a dependent pair `((v : V, p : P), Vec<(v, p)>)` where the choice of `v` and
/// `p` is dictated by the active variant.
///
/// Each statically-known property is associated with a specific variant (e.g. `ATTACH` to
/// [`PropertySeq::Attach`]), but this relationship is not unique; several properties may be
/// associated with the same variant.
#[derive(Debug, Clone, PartialEq, TryInto, From, EnumDiscriminants)]
#[try_into(owned, ref, ref_mut)]
#[strum_discriminants(name(PropertySeqVariant))]
#[strum_discriminants(derive(PartialOrd, Ord))]
pub enum PropertySeq {
    AlarmAction(VP<Token<AlarmAction, String>, Params>),
    Attach(VP<Attachment, Params>),
    Class(VP<Token<ClassValue, String>, Params>),
    Color(VP<Css3Color, Params>),
    Date(VP<Date, Params>),
    DtUtc(VP<DateTime<Utc>, Params>),
    DtOrDate(VP<DateTimeOrDate, Params>),
    Duration(VP<SignedDuration, Params>),
    ExDateSeq(VP<super::primitive::ExDateSeq, Params>),
    Geo(VP<Geo, Params>),
    Gregorian(VP<Gregorian, Params>),
    Integer(VP<Integer, Params>),
    Method(VP<Token<Method, String>, Params>),
    ParticipantType(VP<Token<ParticipantType, String>, Params>),
    Percent(VP<CompletionPercentage, Params>),
    PeriodSeq(VP<Vec<Period>, Params>),
    PositiveInteger(VP<PositiveInteger, Params>),
    Priority(VP<Priority, Params>),
    Proximity(VP<Token<ProximityValue, String>, Params>),
    RDateSeq(VP<RDateSeq, Params>),
    RequestStatus(VP<RequestStatus, Params>),
    ResourceType(VP<Token<ResourceType, String>, Params>),
    RRule(VP<RRule, Params>),
    Status(VP<Status, Params>),
    StructuredData(Vec<StructuredDataProp>),
    StyledDescription(VP<StyledDescriptionValue, Params>),
    Text(VP<String, Params>),
    TextSeq(VP<Vec<String>, Params>),
    TimeTransparency(VP<TimeTransparency, Params>),
    Trigger(VP<TriggerValue, Params>),
    TzId(VP<Box<TzId>, Params>),
    Uid(VP<Box<Uid>, Params>),
    Unknown(VP<Value<String>, Params>),
    Uri(VP<Box<Uri>, Params>),
    UtcOffset(VP<UtcOffset, Params>),
    Version(VP<Version, Params>),
}

impl PropertySeq {
    #[inline(always)]
    pub fn variant(&self) -> PropertySeqVariant {
        self.into()
    }

    pub fn push<T>(&mut self, value: T) -> Result<(), PropertySeqVariant>
    where
        for<'a> &'a mut Self: TryInto<&'a mut Vec<T>>,
    {
        let variant = self.variant();
        let vec: &mut Vec<T> = self.try_into().map_err(|_| variant)?;
        vec.push(value);
        Ok(())
    }
}

// TODO: add methods for converting PropertySeq into {Event, Todo, Journal}Status when it has only
// a single active value in the Status variant, and similar for AlarmAction and its subtypes. this
// works because we can only downcast to these subtypes with specific information about the
// component, and these subtypes may only occur once each in their respective components.

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
