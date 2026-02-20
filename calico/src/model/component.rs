//! Model types for iCalendar components.

use std::num::NonZero;

use enumflags2::{BitFlags, bitflags};
use hashbrown::{Equivalent, HashMap};

use super::{
    property::{PropertySeq, StaticProp},
    string::{CaselessStr, NeverStr},
};

// TODO: extend this macro to support doc comments

macro_rules! define_component_newtype {
    ($v:vis $name:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        #[repr(transparent)]
        $v struct $name(Component);
    };
}

define_component_newtype!(pub Calendar);
define_component_newtype!(pub Event);
define_component_newtype!(pub Todo);
define_component_newtype!(pub Journal);
define_component_newtype!(pub FreeBusy);
define_component_newtype!(pub TimeZone);
define_component_newtype!(pub Standard);
define_component_newtype!(pub Daylight);
define_component_newtype!(pub Alarm);
define_component_newtype!(pub AudioAlarm);
define_component_newtype!(pub DisplayAlarm);
define_component_newtype!(pub EmailAlarm);
define_component_newtype!(pub Participant);
define_component_newtype!(pub Location);
define_component_newtype!(pub Resource);

/// An iCalendar component.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    tag: ComponentTag,
    properties: HashMap<PropKey<Box<CaselessStr>>, PropertySeq>,
    subcomponents: Vec<Self>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ComponentTag {
    // RFC 5545 KNOWN COMPONENTS
    Calendar,
    Event {
        alarm_len: u32,
        participant_len: u32,
        location_len: u32,
        resource_len: u32,
    },
    Todo {
        alarm_len: u32,
        participant_len: u32,
        location_len: u32,
        resource_len: u32,
    },
    Journal {
        participant_len: u32,
        location_len: u32,
        resource_len: u32,
    },
    FreeBusy {
        participant_len: u32,
        location_len: u32,
        resource_len: u32,
    },
    TimeZone {
        rule_len: NonZero<u32>,
    },
    Standard,
    Daylight,
    Alarm {
        kind: AlarmKind,
        location_len: u32,
    },
    Unknown(Box<str>),

    // RFC 9073 KNOWN COMPONENTS
    Participant {
        location_len: u32,
        resource_len: u32,
    },
    Location,
    Resource,

    // MALFORMED COMPONENTS
    MalformedCalendar(BitFlags<MalformedCalendarError>),
    MalformedEvent(BitFlags<MalformedEventError>),
    MalformedTodo(BitFlags<MalformedTodoError>),
    MalformedJournal(BitFlags<MalformedJournalError>),
    MalformedFreeBusy(BitFlags<MalformedFreeBusyError>),
    MalformedStandard(BitFlags<MalformedTzRuleError>),
    MalformedDaylight(BitFlags<MalformedTzRuleError>),
    MalformedAlarm(BitFlags<MalformedAlarmError>, AlarmActionErrors),
}

impl ComponentTag {
    pub fn name(&self) -> ComponentName<&str> {
        match self {
            Self::Calendar | Self::MalformedCalendar(_) => ComponentName::Calendar,
            Self::Event { .. } | Self::MalformedEvent(_) => ComponentName::Event,
            Self::Todo { .. } | Self::MalformedTodo(_) => ComponentName::Todo,
            Self::Journal { .. } | Self::MalformedJournal(_) => ComponentName::Journal,
            Self::FreeBusy { .. } | Self::MalformedFreeBusy(_) => ComponentName::FreeBusy,
            Self::TimeZone { .. } => ComponentName::TimeZone,
            Self::Standard | Self::MalformedStandard(_) => ComponentName::Standard,
            Self::Daylight | Self::MalformedDaylight(_) => ComponentName::Daylight,
            Self::Alarm { .. } | Self::MalformedAlarm(..) => ComponentName::Alarm,
            Self::Participant { .. } => ComponentName::Participant,
            Self::Location => ComponentName::Location,
            Self::Resource => ComponentName::Resource,
            Self::Unknown(value) => ComponentName::Unknown(value),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PropKey<S> {
    Known(StaticProp),
    Unknown(S),
}

impl Equivalent<PropKey<Box<CaselessStr>>> for PropKey<NeverStr> {
    fn equivalent(&self, key: &PropKey<Box<CaselessStr>>) -> bool {
        match (self, key) {
            (PropKey::Known(lhs), PropKey::Known(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Equivalent<PropKey<Box<CaselessStr>>> for PropKey<&str> {
    fn equivalent(&self, key: &PropKey<Box<CaselessStr>>) -> bool {
        match (self, key) {
            (PropKey::Known(lhs), PropKey::Known(rhs)) => lhs == rhs,
            (PropKey::Unknown(lhs), PropKey::Unknown(rhs)) => rhs.as_ref() == *lhs,
            _ => false,
        }
    }
}

// macro_rules! mandatory_accessors {
//     ($([$key:ident, $name:ident, $ret:ty]),* $(,)?) => {
//         $(
//             pub fn $name(&self) -> &$ret {
//                 match self.props.get_known(StaticProp::$key).unwrap().try_into() {
//                     Ok(value) => value,
//                     Err(_) => panic!(),
//                 }
//             }
//
//             paste! {
//             pub fn [<$name _mut>](&mut self) -> &mut $ret {
//                 match self.props.get_known_mut(StaticProp::$key).unwrap().try_into() {
//                     Ok(value) => value,
//                     Err(_) => panic!(),
//                 }
//             }
//             }
//         )*
//     };
// }
//
// macro_rules! check_property_mult {
//     ($props:expr, $comp:ident; { $($key:ident : $mult:ident),+ $(,)? }) => {{
//         let props = &$props;
//         $(
//             check_mult(props, StaticProp::$key, Mult::$mult)
//                 .map_err(|received| FromRawComponentError::InvalidPropMult {
//                     component: StaticComponentName::$comp,
//                     prop: StaticProp::$key,
//                     expected: Mult::$mult,
//                     received,
//                 })?;
//         )+
//     }};
// }

// /// An iCalendar object (RFC 5545 §3.4).
// #[derive(Debug, Clone)]
// pub struct Calendar<S> {
//     props: PropertyTable<S>,
//     components: Vec<CalendarComponent<S>>,
// }
//
// impl<S> Calendar<S> {
//     pub(crate) const fn new(
//         props: PropertyTable<S>,
//         components: Vec<CalendarComponent<S>>,
//     ) -> Self {
//         Self { props, components }
//     }
//
//     pub const fn components(&self) -> &[CalendarComponent<S>] {
//         self.components.as_slice()
//     }
//
//     pub const fn components_mut(&mut self) -> &mut Vec<CalendarComponent<S>> {
//         &mut self.components
//     }
// }
//
// impl<S> Calendar<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [ProdId, prod_id, Prop<String, Params>],
//         [Version, version, Prop<Version, Params>],
//     }
//
//     optional_accessors! {
//         [CalScale, scale, Prop<Gregorian, Params>],
//         [Method, method, Prop<Method<S>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//         [RefreshInterval, refresh_interval, Prop<Duration, Params>],
//         [Source, source, Prop<Uri<S>, Params>],
//         [Color, color, Prop<Css3Color, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Name, names, Prop<String, Params>],
//     //     [Description, description, Prop<String, Params>],
//     //     [Categories, categories, Prop<Vec<String>, Params>],
//     //     [Image, images, Prop<Attachment<S>, Params>],
//     // }
// }
//
// /// An immediate subcomponent of a [`Calendar`].
// #[derive(Debug, Clone)]
// pub enum CalendarComponent<S> {
//     Event(Event<S>),
//     Todo(Todo<S>),
//     Journal(Journal<S>),
//     FreeBusy(FreeBusy<S>),
//     TimeZone(TimeZone<S>),
//     Other(OtherComponent<S>),
// }
//
// /// A VEVENT component (RFC 5545 §3.6.1).
// #[derive(Debug, Clone)]
// pub struct Event<S> {
//     props: PropertyTable<S>,
//     alarms: Vec<Alarm<S>>,
// }
//
// impl<S> Event<S> {
//     pub(crate) const fn new(props: PropertyTable<S>, alarms: Vec<Alarm<S>>) -> Self {
//         Self { props, alarms }
//     }
//
//     pub const fn alarms(&self) -> &[Alarm<S>] {
//         self.alarms.as_slice()
//     }
//
//     pub const fn alarms_mut(&mut self) -> &mut Vec<Alarm<S>> {
//         &mut self.alarms
//     }
// }
//
// impl<S> Event<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [DtStamp, timestamp, Prop<DateTime<Utc>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     pub fn termination(&self) -> Option<EventTerminationRef<'_>> {
//         self.end()
//             .map(EventTerminationRef::End)
//             .or_else(|| self.duration().map(EventTerminationRef::Duration))
//     }
//
//     pub fn termination_mut(&mut self) -> Option<EventTerminationMut<'_>> {
//         if self.end().is_some() {
//             self.end_mut().map(EventTerminationMut::End)
//         } else if self.duration().is_some() {
//             self.duration_mut().map(EventTerminationMut::Duration)
//         } else {
//             None
//         }
//     }
//
//     optional_accessors! {
//         [Status, status, Prop<EventStatus, Params>],
//         [DtStart, start, Prop<DateTimeOrDate, Params>],
//         [DtEnd, end, Prop<DateTimeOrDate, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Class, class, Prop<ClassValue<S>, Params>],
//         [Created, created, Prop<DateTime<Utc>, Params>],
//         [Description, description, Prop<String, Params>],
//         [Geo, geo, Prop<Geo, Params>],
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [Location, location, Prop<String, Params>],
//         [Organizer, organizer, Prop<CalAddress<S>, Params>],
//         [Priority, priority, Prop<Priority, Params>],
//         [Sequence, sequence_number, Prop<Integer, Params>],
//         [Summary, summary, Prop<String, Params>],
//         [Transp, transparency, Prop<TimeTransparency, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//         [RecurId, recurrence_id, Prop<DateTimeOrDate, Params>],
//         [Color, color, Prop<Css3Color, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attach, attachments, Prop<Attachment<S>, Params<S>>],
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params<S>>],
//     //     [Categories, categories, Prop<Vec<Text<S>>, Params<S>>],
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [Contact, contacts, Prop<Text<S>, Params<S>>],
//     //     [RRule, rrule, Prop<Box<RRule>, Params<S>>],
//     //     [ExDate, exception_dates, Prop<ExDateSeq, Params<S>>],
//     //     [RequestStatus, request_statuses, Prop<RequestStatus<S>, Params<S>>],
//     //     [RelatedTo, relateds, Prop<Uid<S>, Params<S>>],
//     //     [Resources, resources, Prop<Vec<Text<S>>, Params<S>>],
//     //     [RDate, recurrence_dates, Prop<RDateSeq, Params<S>>],
//     //     [Conference, conferences, Prop<Uri<S>, Params<S>>],
//     //     [Image, images, Prop<Attachment<S>, Params<S>>],
//     // }
// }
//
// /// A VTODO component (RFC 5545 §3.6.2).
// #[derive(Debug, Clone)]
// pub struct Todo<S> {
//     props: PropertyTable<S>,
//     alarms: Vec<Alarm<S>>,
// }
//
// impl<S> Todo<S> {
//     pub(crate) const fn new(props: PropertyTable<S>, alarms: Vec<Alarm<S>>) -> Self {
//         Self { props, alarms }
//     }
//
//     pub const fn alarms(&self) -> &[Alarm<S>] {
//         self.alarms.as_slice()
//     }
//
//     pub const fn alarms_mut(&mut self) -> &mut Vec<Alarm<S>> {
//         &mut self.alarms
//     }
// }
//
// impl<S> Todo<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [DtStamp, timestamp, Prop<DateTime<Utc>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     pub fn termination(&self) -> Option<TodoTerminationRef<'_>> {
//         self.due()
//             .map(TodoTerminationRef::Due)
//             .or_else(|| self.duration().map(TodoTerminationRef::Duration))
//     }
//
//     pub fn termination_mut(&mut self) -> Option<TodoTerminationMut<'_>> {
//         if self.due().is_some() {
//             self.due_mut().map(TodoTerminationMut::Due)
//         } else if self.duration().is_some() {
//             self.duration_mut().map(TodoTerminationMut::Duration)
//         } else {
//             None
//         }
//     }
//
//     optional_accessors! {
//         [Status, status, Prop<TodoStatus, Params>],
//         [Class, class, Prop<ClassValue<S>, Params>],
//         [DtCompleted, completed, Prop<DateTime<Utc>, Params>],
//         [Created, created, Prop<DateTime<Utc>, Params>],
//         [Description, description, Prop<String, Params>],
//         [DtStart, start, Prop<DateTimeOrDate, Params>],
//         [DtDue, due, Prop<DateTimeOrDate, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Geo, geo, Prop<Geo, Params>],
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [Location, location, Prop<String, Params>],
//         [Organizer, organizer, Prop<CalAddress<S>, Params>],
//         [PercentComplete, percent, Prop<CompletionPercentage, Params>],
//         [Priority, priority, Prop<Priority, Params>],
//         [RecurId, recurrence_id, Prop<DateTimeOrDate, Params>],
//         [Sequence, sequence_number, Prop<Integer, Params>],
//         [Summary, summary, Prop<String, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//         [Color, color, Prop<Css3Color, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attach, attachments, Prop<Attachment<S>, Params<S>>],
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params<S>>],
//     //     [Categories, categories, Prop<Vec<Text<S>>, Params<S>>],
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [Contact, contacts, Prop<Text<S>, Params<S>>],
//     //     [RRule, rrule, Prop<Box<RRule>, Params<S>>],
//     //     [ExDate, exception_dates, Prop<ExDateSeq, Params<S>>],
//     //     [RequestStatus, request_statuses, Prop<RequestStatus<S>, Params<S>>],
//     //     [RelatedTo, relateds, Prop<Uid<S>, Params<S>>],
//     //     [Resources, resources, Prop<Vec<Text<S>>, Params<S>>],
//     //     [RDate, recurrence_dates, Prop<RDateSeq, Params<S>>],
//     //     [Conference, conferences, Prop<Uri<S>, Params<S>>],
//     //     [Image, images, Prop<Attachment<S>, Params<S>>],
//     // }
// }
//
// /// A VJOURNAL component (RFC 5545 §3.6.3).
// #[derive(Debug, Clone)]
// pub struct Journal<S> {
//     props: PropertyTable<S>,
// }
//
// impl<S> Journal<S> {
//     pub(crate) const fn new(props: PropertyTable<S>) -> Self {
//         Self { props }
//     }
// }
//
// impl<S> Journal<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [DtStamp, timestamp, Prop<DateTime<Utc>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     optional_accessors! {
//         [Status, status, Prop<JournalStatus, Params>],
//         [Class, class, Prop<ClassValue<S>, Params>],
//         [Created, created, Prop<DateTime<Utc>, Params>],
//         [DtStart, start, Prop<DateTimeOrDate, Params>],
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [Organizer, organizer, Prop<CalAddress<S>, Params>],
//         [RecurId, recurrence_id, Prop<DateTimeOrDate, Params>],
//         [Sequence, sequence_number, Prop<Integer, Params>],
//         [Summary, summary, Prop<String, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attach, attachments, Prop<Attachment<S>, Params<S>>],
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params<S>>],
//     //     [Categories, categories, Prop<Vec<Text<S>>, Params<S>>],
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [Contact, contacts, Prop<Text<S>, Params<S>>],
//     //     [Description, descriptions, Prop<Text<S>, Params<S>>],
//     //     [ExDate, exception_dates, Prop<ExDateSeq, Params<S>>],
//     //     [RelatedTo, relateds, Prop<Uid<S>, Params<S>>],
//     //     [RDate, recurrence_dates, Prop<RDateSeq, Params<S>>],
//     //     [RRule, rrule, Prop<Box<RRule>, Params<S>>],
//     //     [RequestStatus, request_statuses, Prop<RequestStatus<S>, Params<S>>],
//     // }
// }
//
// /// A VFREEBUSY component (RFC 5545 §3.6.4).
// #[derive(Debug, Clone)]
// pub struct FreeBusy<S> {
//     props: PropertyTable<S>,
// }
//
// impl<S> FreeBusy<S> {
//     pub(crate) const fn new(props: PropertyTable<S>) -> Self {
//         Self { props }
//     }
// }
//
// impl<S> FreeBusy<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [DtStamp, timestamp, Prop<DateTime<Utc>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     optional_accessors! {
//         [Contact, contact, Prop<String, Params>],
//         [DtStart, start, Prop<DateTimeOrDate, Params>],
//         [DtEnd, end, Prop<DateTimeOrDate, Params>],
//         [Organizer, organizer, Prop<CalAddress<S>, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params<S>>],
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [FreeBusy, free_busy_periods, Prop<Vec<Period>, Params<S>>],
//     //     [RequestStatus, request_statuses, Prop<RequestStatus<S>, Params<S>>],
//     // }
// }
//
// /// A VTIMEZONE component (RFC 5545 §3.6.5).
// #[derive(Debug, Clone)]
// pub struct TimeZone<S> {
//     props: PropertyTable<S>,
//     subcomponents: Vec<TzRule<S>>,
// }
//
// impl<S> TimeZone<S> {
//     pub(crate) const fn new(props: PropertyTable<S>, subcomponents: Vec<TzRule<S>>) -> Self {
//         Self {
//             props,
//             subcomponents,
//         }
//     }
//
//     pub const fn rules(&self) -> &[TzRule<S>] {
//         self.subcomponents.as_slice()
//     }
//
//     pub const fn rules_mut(&mut self) -> &mut Vec<TzRule<S>> {
//         &mut self.subcomponents
//     }
// }
//
// impl<S> TimeZone<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [TzId, id, Prop<TzId, Params>],
//     }
//
//     optional_accessors! {
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [TzUrl, url, Prop<Uri<S>, Params>],
//     }
// }
//
// /// A STANDARD or DAYLIGHT subcomponent of a [`TimeZone`].
// #[derive(Debug, Clone)]
// pub struct TzRule<S> {
//     kind: TzRuleKind,
//     props: PropertyTable<S>,
// }
//
// impl<S> TzRule<S> {
//     pub(crate) const fn new(props: PropertyTable<S>, kind: TzRuleKind) -> Self {
//         Self { kind, props }
//     }
//
//     pub fn kind(&self) -> TzRuleKind {
//         self.kind
//     }
// }
//
// impl<S> TzRule<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [DtStart, start, Prop<DateTimeOrDate, Params>],
//         [TzOffsetTo, offset_to, Prop<UtcOffset, Params>],
//         [TzOffsetFrom, offset_from, Prop<UtcOffset, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [RDate, recurrence_dates, Prop<RDateSeq, Params<S>>],
//     //     [RRule, rrule, Prop<Box<RRule>, Params<S>>],
//     //     [TzName, names, Prop<Text<S>, Params<S>>],
//     // }
// }
//
// /// The kind of a [`TzRule`], for which the default is [`Standard`].
// ///
// /// [`Standard`]: TzRuleKind::Standard
// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// pub enum TzRuleKind {
//     #[default]
//     Standard,
//     Daylight,
// }
//
// // TODO: VALARM should admit a tail of VLOCATION components when the PROXIMITY property is
// // present (RFC 9074 §8).
//
// macro_rules! alarm_accessors {
//     () => {
//         pub fn duration_and_repeat(
//             &self,
//         ) -> Option<(&Prop<Duration, Params>, &Prop<Integer, Params>)> {
//             let duration = self.duration();
//             let repeat = self.repeat();
//
//             match (duration, repeat) {
//                 (Some(duration), Some(repeat)) => Some((duration, repeat)),
//                 (Some(_), None) => panic!("DURATION without REPEAT in VALARM"),
//                 (None, Some(_)) => panic!("REPEAT without DURATION in VALARM"),
//                 (None, None) => None,
//             }
//         }
//
//         pub fn trigger(&self) -> TriggerPropRef<'_> {
//             self.props
//                 .get_known(StaticProp::Trigger)
//                 .unwrap()
//                 .try_into()
//                 .unwrap()
//         }
//
//         pub fn trigger_mut(&mut self) -> TriggerPropMut<'_> {
//             self.props
//                 .get_known_mut(StaticProp::Trigger)
//                 .unwrap()
//                 .try_into()
//                 .unwrap()
//         }
//     };
// }
//
// type DurAndRepeatRef<'a> = (&'a Prop<Duration, Params>, &'a Prop<Integer, Params>);
// type ProxAndLocations<S> = (Prop<ProximityValue<S>, Params>, Vec<Location<S>>);
// type ProxAndLocationsRef<'a, S> = (&'a Prop<ProximityValue<S>, Params>, &'a [Location<S>]);
// type ProxAndLocationsMut<'a, S> = (
//     &'a mut Prop<ProximityValue<S>, Params>,
//     &'a mut Vec<Location<S>>,
// );
//
// // TODO: handle alarms with unknown actions as a separate type
//
// pub struct RawAlarm<S, K> {
//     props: PropertyTable<S>,
//     prox_and_locations: Option<ProxAndLocations<S>>,
//     __kind: PhantomData<K>,
// }
//
// impl<S: Debug, K> Debug for RawAlarm<S, K> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("RawAlarm")
//             .field("props", &self.props)
//             .field("prox_and_locations", &self.prox_and_locations)
//             .finish()
//     }
// }
//
// impl<S, K: AlarmKind<S>> RawAlarm<S, K> {
//     pub fn action(&self) -> &Prop<K::Action, Params> {
//         K::get_action_ref(&self.props)
//     }
//
//     pub fn action_mut(&mut self) -> &mut Prop<K::Action, Params> {
//         K::get_action_mut(&mut self.props)
//     }
// }
//
// impl<S, K> RawAlarm<S, K>
// where
//     S: HashCaseless + Equiv,
//     K: AlarmKindWithMandatoryDescription,
// {
//     pub fn description(&self) -> &Prop<String, Params> {
//         todo!()
//     }
//
//     pub fn description_mut(&mut self) -> &mut Prop<String, Params> {
//         todo!()
//     }
// }
//
// impl<S> RawAlarm<S, AudioAlarmKind>
// where
//     S: HashCaseless + Equiv,
// {
//     optional_accessors! {
//         [Attach, attachment, Prop<Attachment<S>, Params>],
//     }
// }
//
// impl<S> RawAlarm<S, EmailAlarmKind>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Summary, summary, Prop<String, Params>]
//     }
//
//     // NOTE: there must be at least one attendee; maybe use the types provided by the mitsein crate
//     // to express this in the API?
//
//     pub fn attendees(&self) -> &[Prop<CalAddress<S>, Params>] {
//         todo!()
//     }
//
//     pub fn attendees_mut(&mut self) -> &mut Vec<Prop<CalAddress<S>, Params>> {
//         todo!()
//     }
//
//     pub fn attachments(&self) -> &[Prop<Attachment<S>, Params>] {
//         todo!()
//     }
//
//     pub fn attachments_mut(&mut self) -> &mut Vec<Prop<Attachment<S>, Params>> {
//         todo!()
//     }
// }
//
// impl<S, K> RawAlarm<S, K>
// where
//     S: HashCaseless + Equiv,
// {
//     optional_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Repeat, repeat, Prop<Integer, Params>],
//         [Acknowledged, acknowlegded, Prop<DateTime<Utc>, Params>],
//     }
//
//     pub fn duration_and_repeat(&self) -> Option<DurAndRepeatRef<'_>> {
//         let duration = self.duration();
//         let repeat = self.repeat();
//
//         match (duration, repeat) {
//             (Some(duration), Some(repeat)) => Some((duration, repeat)),
//             (Some(_), None) => panic!("DURATION without REPEAT in VALARM"),
//             (None, Some(_)) => panic!("REPEAT without DURATION in VALARM"),
//             (None, None) => None,
//         }
//     }
//
//     // TODO: duration_and_repeat_mut
//
//     pub fn trigger(&self) -> TriggerPropRef<'_> {
//         self.props
//             .get_known(StaticProp::Trigger)
//             .unwrap()
//             .try_into()
//             .unwrap()
//     }
//
//     pub fn trigger_mut(&mut self) -> TriggerPropMut<'_> {
//         self.props
//             .get_known_mut(StaticProp::Trigger)
//             .unwrap()
//             .try_into()
//             .unwrap()
//     }
//
//     pub const fn proximity_and_locations(&self) -> Option<ProxAndLocationsRef<'_, S>> {
//         match self.prox_and_locations.as_ref() {
//             Some((prox, locs)) => Some((prox, locs.as_slice())),
//             None => None,
//         }
//     }
//
//     pub const fn proximity_and_locations_mut(&mut self) -> Option<ProxAndLocationsMut<'_, S>> {
//         match self.prox_and_locations.as_mut() {
//             Some((prox, locs)) => Some((prox, locs)),
//             None => None,
//         }
//     }
//
//     pub const fn remove_proximity_and_locations(&mut self) -> Option<ProxAndLocations<S>> {
//         self.prox_and_locations.take()
//     }
//
//     pub const fn proximity(&self) -> Option<&Prop<ProximityValue<S>, Params>> {
//         match self.prox_and_locations.as_ref() {
//             Some((prox, _)) => Some(prox),
//             None => None,
//         }
//     }
//
//     pub const fn proximity_mut(&mut self) -> Option<&mut Prop<ProximityValue<S>, Params>> {
//         match self.prox_and_locations.as_mut() {
//             Some((prox, _)) => Some(prox),
//             None => None,
//         }
//     }
//
//     pub const fn locations(&self) -> Option<&[Location<S>]> {
//         match self.prox_and_locations.as_ref() {
//             Some((_, locs)) => Some(locs.as_slice()),
//             None => None,
//         }
//     }
//
//     pub const fn locations_mut(&mut self) -> Option<&mut Vec<Location<S>>> {
//         match self.prox_and_locations.as_mut() {
//             Some((_, locs)) => Some(locs),
//             None => None,
//         }
//     }
// }
//
// // TODO: we need a way to be generic over multiplicities for the following properties:
// // - DESCRIPTION (0 in AUDIO, 1 in DISPLAY and EMAIL)
// // - SUMMARY (0 in AUDIO and DISPLAY, 1 in EMAIL)
// // - ATTENDEE (0 in AUDIO and DISPLAY, 1 or more in EMAIL)
// // - ATTACH (0 or 1 in AUDIO, 0 in DISPLAY, 0 or more in EMAIL)
//
// pub struct AudioAlarmKind;
// pub struct DisplayAlarmKind;
// pub struct EmailAlarmKind;
//
// pub trait AlarmKind<S> {
//     type Action;
//
//     fn get_action_ref(props: &PropertyTable<S>) -> &Prop<Self::Action, Params>;
//     fn get_action_mut(props: &mut PropertyTable<S>) -> &mut Prop<Self::Action, Params>;
// }
//
// /// Marker trait for alarm kinds where the DESCRIPTION property must occur exactly once, used to
// /// define the [`description`] and [`description_mut`] methods on [`RawAlarm`].
// ///
// /// [`description`]: RawAlarm::description
// /// [`description_mut`]: RawAlarm::description_mut
// pub trait AlarmKindWithMandatoryDescription {}
// impl AlarmKindWithMandatoryDescription for DisplayAlarmKind {}
// impl AlarmKindWithMandatoryDescription for EmailAlarmKind {}
//
// impl<S: HashCaseless + Equiv> AlarmKind<S> for AudioAlarmKind {
//     type Action = AudioAction;
//
//     fn get_action_ref(props: &PropertyTable<S>) -> &Prop<Self::Action, Params> {
//         props
//             .get_known(StaticProp::Action)
//             .unwrap()
//             .downcast_ref::<Prop<AudioAction, Params>>()
//             .unwrap()
//     }
//
//     fn get_action_mut(props: &mut PropertyTable<S>) -> &mut Prop<Self::Action, Params> {
//         props
//             .get_known_mut(StaticProp::Action)
//             .unwrap()
//             .downcast_mut::<Prop<AudioAction, Params>>()
//             .unwrap()
//     }
// }
//
// impl<S: HashCaseless + Equiv> AlarmKind<S> for DisplayAlarmKind {
//     type Action = DisplayAction;
//
//     fn get_action_ref(props: &PropertyTable<S>) -> &Prop<Self::Action, Params> {
//         props
//             .get_known(StaticProp::Action)
//             .unwrap()
//             .downcast_ref::<Prop<DisplayAction, Params>>()
//             .unwrap()
//     }
//
//     fn get_action_mut(props: &mut PropertyTable<S>) -> &mut Prop<Self::Action, Params> {
//         props
//             .get_known_mut(StaticProp::Action)
//             .unwrap()
//             .downcast_mut::<Prop<DisplayAction, Params>>()
//             .unwrap()
//     }
// }
//
// impl<S: HashCaseless + Equiv> AlarmKind<S> for EmailAlarmKind {
//     type Action = EmailAction;
//
//     fn get_action_ref(props: &PropertyTable<S>) -> &Prop<Self::Action, Params> {
//         props
//             .get_known(StaticProp::Action)
//             .unwrap()
//             .downcast_ref::<Prop<EmailAction, Params>>()
//             .unwrap()
//     }
//
//     fn get_action_mut(props: &mut PropertyTable<S>) -> &mut Prop<Self::Action, Params> {
//         props
//             .get_known_mut(StaticProp::Action)
//             .unwrap()
//             .downcast_mut::<Prop<EmailAction, Params>>()
//             .unwrap()
//     }
// }
//
// /// A VALARM component (RFC 5545 §3.6.6).
// #[derive(Debug, Clone)]
// pub enum Alarm<S> {
//     Audio(AudioAlarm<S>),
//     Display(DisplayAlarm<S>),
//     Email(EmailAlarm<S>),
//     Other(OtherAlarm<S>),
// }
//
// impl<S> Alarm<S> {
//     pub const fn locations(&self) -> &[Location<S>] {
//         match self {
//             Alarm::Audio(alarm) => alarm.locations(),
//             Alarm::Display(alarm) => alarm.locations(),
//             Alarm::Email(alarm) => alarm.locations(),
//             Alarm::Other(alarm) => alarm.locations(),
//         }
//     }
//
//     pub const fn locations_mut(&mut self) -> &mut Vec<Location<S>> {
//         match self {
//             Alarm::Audio(alarm) => alarm.locations_mut(),
//             Alarm::Display(alarm) => alarm.locations_mut(),
//             Alarm::Email(alarm) => alarm.locations_mut(),
//             Alarm::Other(alarm) => alarm.locations_mut(),
//         }
//     }
//
//     pub const fn other_subcomponents(&self) -> &[OtherComponent<S>] {
//         match self {
//             Alarm::Audio(alarm) => alarm.other_subcomponents(),
//             Alarm::Display(alarm) => alarm.other_subcomponents(),
//             Alarm::Email(alarm) => alarm.other_subcomponents(),
//             Alarm::Other(alarm) => alarm.other_subcomponents(),
//         }
//     }
//
//     pub const fn other_subcomponents_mut(&mut self) -> &mut Vec<OtherComponent<S>> {
//         match self {
//             Alarm::Audio(alarm) => alarm.other_subcomponents_mut(),
//             Alarm::Display(alarm) => alarm.other_subcomponents_mut(),
//             Alarm::Email(alarm) => alarm.other_subcomponents_mut(),
//             Alarm::Other(alarm) => alarm.other_subcomponents_mut(),
//         }
//     }
// }
//
// macro_rules! alarm_subtype_methods {
//     () => {
//         pub(crate) const fn new(
//             props: PropertyTable<S>,
//             locations: Vec<Location<S>>,
//             other_subcomponents: Vec<OtherComponent<S>>,
//         ) -> Self {
//             Self {
//                 props,
//                 locations,
//                 other_subcomponents,
//             }
//         }
//
//         pub const fn locations(&self) -> &[Location<S>] {
//             self.locations.as_slice()
//         }
//
//         pub const fn locations_mut(&mut self) -> &mut Vec<Location<S>> {
//             &mut self.locations
//         }
//
//         pub const fn other_subcomponents(&self) -> &[OtherComponent<S>] {
//             self.other_subcomponents.as_slice()
//         }
//
//         pub const fn other_subcomponents_mut(&mut self) -> &mut Vec<OtherComponent<S>> {
//             &mut self.other_subcomponents
//         }
//     };
// }
//
// /// A VALARM with the AUDIO action.
// #[derive(Debug, Clone)]
// pub struct AudioAlarm<S> {
//     props: PropertyTable<S>,
//     locations: Vec<Location<S>>,
//     other_subcomponents: Vec<OtherComponent<S>>,
// }
//
// impl<S> AudioAlarm<S> {
//     alarm_subtype_methods!();
// }
//
// impl<S> AudioAlarm<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Action, action, Prop<AudioAction, Params>],
//     }
//
//     optional_accessors! {
//         [Attach, attachment, Prop<Attachment<S>, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Repeat, repeat, Prop<Integer, Params>],
//         [Acknowledged, acknowleded, Prop<DateTime<Utc>, Params>],
//         [Proximity, proximity, Prop<ProximityValue<S>, Params>],
//     }
//
//     alarm_accessors!();
// }
//
// /// A VALARM with the DISPLAY action.
// #[derive(Debug, Clone)]
// pub struct DisplayAlarm<S> {
//     props: PropertyTable<S>,
//     locations: Vec<Location<S>>,
//     other_subcomponents: Vec<OtherComponent<S>>,
// }
//
// impl<S> DisplayAlarm<S> {
//     alarm_subtype_methods!();
// }
//
// impl<S> DisplayAlarm<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Action, action, Prop<DisplayAction, Params>],
//         [Description, description, Prop<String, Params>],
//     }
//
//     optional_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Repeat, repeat, Prop<Integer, Params>],
//         [Acknowledged, acknowleded, Prop<DateTime<Utc>, Params>],
//         [Proximity, proximity, Prop<ProximityValue<S>, Params>],
//     }
//
//     alarm_accessors!();
// }
//
// /// A VALARM with the EMAIL action.
// #[derive(Debug, Clone)]
// pub struct EmailAlarm<S> {
//     props: PropertyTable<S>,
//     locations: Vec<Location<S>>,
//     other_subcomponents: Vec<OtherComponent<S>>,
// }
//
// impl<S> EmailAlarm<S> {
//     alarm_subtype_methods!();
// }
//
// impl<S> EmailAlarm<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Action, action, Prop<EmailAction, Params>],
//         [Description, description, Prop<String, Params>],
//         [Summary, summary, Prop<String, Params>],
//     }
//
//     optional_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Repeat, repeat, Prop<Integer, Params>],
//         [Acknowledged, acknowleded, Prop<DateTime<Utc>, Params>],
//         [Proximity, proximity, Prop<ProximityValue<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params>],
//     //     [Attach, attachments, Prop<Attachment<S>, Params>],
//     // }
//
//     alarm_accessors!();
// }
//
// /// A VALARM with an action other than AUDIO, DISPLAY, or EMAIL.
// #[derive(Debug, Clone)]
// pub struct OtherAlarm<S> {
//     props: PropertyTable<S>,
//     locations: Vec<Location<S>>,
//     other_subcomponents: Vec<OtherComponent<S>>,
// }
//
// impl<S> OtherAlarm<S> {
//     alarm_subtype_methods!();
// }
//
// impl<S> OtherAlarm<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Action, action, Prop<UnknownAction<S>, Params>],
//     }
//
//     optional_accessors! {
//         [Description, description, Prop<String, Params>],
//         [Summary, summary, Prop<String, Params>],
//         [Uid, uid, Prop<Uid, Params>],
//         [Duration, duration, Prop<Duration, Params>],
//         [Repeat, repeat, Prop<Integer, Params>],
//         [Acknowledged, acknowlegded, Prop<DateTime<Utc>, Params>],
//         [Proximity, proximity, Prop<ProximityValue<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attendee, attendees, Prop<CalAddress<S>, Params<S>>],
//     //     [Attach, attachments, Prop<Attachment<S>, Params<S>>],
//     // }
//
//     alarm_accessors!();
// }
//
// /// A PARTICIPANT component (RFC 9073 §7.1).
// #[derive(Debug, Clone)]
// pub struct Participant<S> {
//     props: PropertyTable<S>,
//     locations: Vec<Location<S>>,
//     resources: Vec<Resource<S>>,
// }
//
// impl<S: PartialEq + HashCaseless + Equiv> PartialEq for Participant<S> {
//     fn eq(&self, other: &Self) -> bool {
//         self.props == other.props
//             && self.locations == other.locations
//             && self.resources == other.resources
//     }
// }
//
// impl<S, T> TryFrom<RawComponent<S, T>> for Participant<S>
// where
//     S: HashCaseless,
//     T: SubcomponentSet<S>,
// {
//     type Error = FromRawComponentError<S>;
//
//     fn try_from(
//         RawComponent {
//             name,
//             props,
//             subcomponents,
//         }: RawComponent<S, T>,
//     ) -> Result<Self, Self::Error> {
//         if !matches!(name, ComponentName::Participant) {
//             return Err(FromRawComponentError::InvalidName {
//                 expected: StaticComponentName::Participant,
//                 received: name,
//             });
//         }
//
//         // NOTE: RFC 9073 includes "strucloc" and "strucres" as symbols in the grammer for this
//         // component, but these were removed in erratum EID 6829.
//
//         check_property_mult! {props, Participant; {
//             // mandatory
//             Uid: One,
//             ParticipantType: One,
//             // optional
//             CalendarAddress: Optional,
//             Created: Optional,
//             Description: Optional,
//             DtStamp: Optional,
//             Geo: Optional,
//             LastModified: Optional,
//             Priority: Optional,
//             Sequence: Optional,
//             Status: Optional,
//             Summary: Optional,
//             Url: Optional,
//             // sequential
//             Attach: Any,
//             Categories: Any,
//             Comment: Any,
//             Contact: Any,
//             Location: Any,
//             RequestStatus: Any,
//             RelatedTo: Any,
//             Resources: Any,
//             StyledDescription: Any,
//             StructuredData: Any,
//         }};
//
//         let ParticipantSubcomponents {
//             locations,
//             resources,
//         } = subcomponents.try_into_participant_subcomponents()?;
//
//         Ok(Self {
//             props,
//             locations,
//             resources,
//         })
//     }
// }
//
// impl<S> Participant<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//         [ParticipantType, participant_type, Prop<ParticipantType<S>, Params>],
//     }
//
//     optional_accessors! {
//         [CalendarAddress, calendar_address, Prop<CalAddress<S>, Params>],
//         [Created, created, Prop<DateTime<Utc>, Params>],
//         [Description, description, Prop<String, Params>],
//         [DtStamp, timestamp, Prop<DateTime<Utc>, Params>],
//         [Geo, geo, Prop<Geo, Params>],
//         [LastModified, last_modified, Prop<DateTime<Utc>, Params>],
//         [Priority, priority, Prop<Priority, Params>],
//         [Sequence, sequence, Prop<Integer, Params>],
//         [Status, status, Prop<Status, Params>],
//         [Summary, summary, Prop<String, Params>],
//         [Url, url, Prop<Uri<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [Attach, attachments, Prop<Attachment<S>, Params<S>>],
//     //     [Categories, categories, Prop<Vec<Text<S>>, Params<S>>],
//     //     [Comment, comments, Prop<Text<S>, Params<S>>],
//     //     [Contact, contacts, Prop<Text<S>, Params<S>>],
//     //     [Location, location_properties, Prop<Text<S>, Params<S>>],
//     //     [RequestStatus, request_statuses, Prop<RequestStatus<S>, Params<S>>],
//     //     [RelatedTo, relateds, Prop<Uid<S>, Params<S>>],
//     //     [Resources, resource_properties, Prop<Vec<Text<S>>, Params<S>>],
//     //     [StyledDescription, styled_descriptions, Prop<StyledDescriptionValue<S>, Params<S>>],
//     //     [StructuredData, structured_data, StructuredDataProp<S>],
//     // }
//
//     /// Returns the value of the CALENDAR-ADDRESS property if `self` is _schedulable_ as described
//     /// in RFC 9073 §7.1.1 and with respect to the given attendee addresses.
//     pub fn schedulable_calendar_address<'a>(
//         &'a self,
//         mut attendees: impl Iterator<Item = &'a CalAddress<S>>,
//     ) -> Option<&'a CalAddress<S>> {
//         let address = &self.calendar_address()?.value;
//
//         match attendees.any(|x| x.0.equiv(&address.0)) {
//             true => Some(address),
//             false => None,
//         }
//     }
// }
//
// impl<S> Participant<S> {
//     pub const fn locations(&self) -> &[Location<S>] {
//         self.locations.as_slice()
//     }
//
//     pub const fn resources(&self) -> &[Resource<S>] {
//         self.resources.as_slice()
//     }
//
//     pub const fn locations_mut(&mut self) -> &mut Vec<Location<S>> {
//         &mut self.locations
//     }
//
//     pub const fn resources_mut(&mut self) -> &mut Vec<Resource<S>> {
//         &mut self.resources
//     }
// }
//
// /// A VLOCATION component (RFC 9073 §7.2).
// #[derive(Debug, Clone)]
// pub struct Location<S> {
//     props: PropertyTable<S>,
// }
//
// impl<S: PartialEq + HashCaseless + Equiv> PartialEq for Location<S> {
//     fn eq(&self, other: &Self) -> bool {
//         self.props == other.props
//     }
// }
//
// impl<S> Location<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     optional_accessors! {
//         [Description, description, Prop<String, Params>],
//         [Geo, geo, Prop<Geo, Params>],
//         [Name, name, Prop<String, Params>],
//         [LocationType, location_type, Prop<String, Params>],
//     }
//
//     // seq_accessors! {
//     //     [StructuredData, structured_data, StructuredDataProp<S>],
//     // }
// }
//
// impl<S, T> TryFrom<RawComponent<S, T>> for Location<S>
// where
//     S: HashCaseless,
//     T: SubcomponentSet<S>,
// {
//     type Error = FromRawComponentError<S>;
//
//     fn try_from(
//         RawComponent {
//             name,
//             props,
//             subcomponents,
//         }: RawComponent<S, T>,
//     ) -> Result<Self, Self::Error> {
//         if !matches!(name, ComponentName::Location) {
//             return Err(FromRawComponentError::InvalidName {
//                 expected: StaticComponentName::Location,
//                 received: name,
//             });
//         }
//
//         check_property_mult! {props, Location; {
//             Uid: One,
//             Description: Optional,
//             Geo: Optional,
//             Name: Optional,
//             LocationType: Optional,
//             Url: Optional, // RFC 9073 EID 7381
//             StructuredData: Any,
//         }};
//
//         if !subcomponents.is_empty() {
//             return Err(FromRawComponentError::UnexpectedSubcomponents {
//                 component: StaticComponentName::Location,
//             });
//         }
//
//         Ok(Self { props })
//     }
// }
//
// /// A VRESOURCE component (RFC 9073 §7.3).
// #[derive(Debug, Clone)]
// pub struct Resource<S> {
//     props: PropertyTable<S>,
// }
//
// impl<S: PartialEq + HashCaseless + Equiv> PartialEq for Resource<S> {
//     fn eq(&self, other: &Self) -> bool {
//         self.props == other.props
//     }
// }
//
// impl<S, T> TryFrom<RawComponent<S, T>> for Resource<S>
// where
//     S: HashCaseless,
//     T: SubcomponentSet<S>,
// {
//     type Error = FromRawComponentError<S>;
//
//     fn try_from(
//         RawComponent {
//             name,
//             props,
//             subcomponents,
//         }: RawComponent<S, T>,
//     ) -> Result<Self, Self::Error> {
//         if !matches!(name, ComponentName::Resource) {
//             return Err(FromRawComponentError::InvalidName {
//                 expected: StaticComponentName::Resource,
//                 received: name,
//             });
//         }
//
//         check_property_mult! {props, Resource; {
//             Uid: One,
//             Description: Optional,
//             Geo: Optional,
//             Name: Optional,
//             ResourceType: Optional,
//             StructuredData: Any,
//         }};
//
//         if !subcomponents.is_empty() {
//             return Err(FromRawComponentError::UnexpectedSubcomponents {
//                 component: StaticComponentName::Resource,
//             });
//         }
//
//         Ok(Self { props })
//     }
// }
//
// impl<S> Resource<S>
// where
//     S: HashCaseless + Equiv,
// {
//     mandatory_accessors! {
//         [Uid, uid, Prop<Uid, Params>],
//     }
//
//     optional_accessors! {
//         [Description, description, Prop<String, Params>],
//         [Geo, geo, Prop<Geo, Params>],
//         [Name, name, Prop<String, Params>],
//         [ResourceType, resource_type, Prop<ResourceType<S>, Params>],
//     }
//
//     // seq_accessors! {
//     //     [StructuredData, structured_data, StructuredDataProp<S>],
//     // }
// }
//
// // TODO: should OtherComponent be using PropertyTable? or should it use a more stringly-typed map
// // internally instead?
//
// /// An arbitrary component which may have any properties and subcomponents.
// #[derive(Debug, Clone)]
// pub struct OtherComponent<S> {
//     name: UnknownName<S>,
//     props: PropertyTable<S>,
//     subcomponents: Vec<OtherComponent<S>>,
// }
//
// impl<S> OtherComponent<S> {
//     pub(crate) const fn new(
//         name: UnknownName<S>,
//         props: PropertyTable<S>,
//         subcomponents: Vec<OtherComponent<S>>,
//     ) -> Self {
//         Self {
//             name,
//             props,
//             subcomponents,
//         }
//     }
//
//     pub const fn name(&self) -> &UnknownName<S> {
//         &self.name
//     }
//
//     pub const fn name_mut(&mut self) -> &mut UnknownName<S> {
//         &mut self.name
//     }
//
//     pub const fn subcomponents(&self) -> &[OtherComponent<S>] {
//         self.subcomponents.as_slice()
//     }
//
//     pub const fn subcomponents_mut(&mut self) -> &mut Vec<OtherComponent<S>> {
//         &mut self.subcomponents
//     }
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct UnknownName<S> {
//     pub name: S,
//     pub kind: UnknownKind,
// }
//
// impl<S> UnknownName<S> {
//     pub const fn iana(name: S) -> Self {
//         Self {
//             name,
//             kind: UnknownKind::Iana,
//         }
//     }
//
//     pub const fn x(name: S) -> Self {
//         Self {
//             name,
//             kind: UnknownKind::X,
//         }
//     }
// }
//
// fn check_mult<S: HashCaseless>(
//     props: &PropertyTable<S>,
//     key: StaticProp,
//     mult: Mult,
// ) -> Result<(), usize> {
//     let len = props
//         .get_known(key)
//         .map(RawPropValue::len)
//         .unwrap_or_default();
//
//     match mult.admits(len) {
//         true => Ok(()),
//         false => Err(len),
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct RawComponent<S, T> {
//     pub name: ComponentName<S>,
//     pub props: PropertyTable<S>,
//     pub subcomponents: T,
// }
//
// pub enum FromRawComponentError<S> {
//     InvalidName {
//         expected: StaticComponentName,
//         received: ComponentName<S>,
//     },
//     InvalidPropMult {
//         component: StaticComponentName,
//         prop: StaticProp,
//         expected: Mult,
//         received: usize,
//     },
//     UnexpectedSubcomponents {
//         component: StaticComponentName,
//     },
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Mult {
//     Zero,
//     One,
//     Optional,
//     OneOrMore,
//     Any,
// }
//
// impl Mult {
//     #[inline(always)]
//     pub const fn admits(self, value: usize) -> bool {
//         match self {
//             Mult::Zero => value == 0,
//             Mult::One => value == 1,
//             Mult::Optional => value < 2,
//             Mult::OneOrMore => value > 0,
//             Mult::Any => true,
//         }
//     }
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum StaticComponentName {
//     Calendar,
//     Event,
//     Todo,
//     Journal,
//     FreeBusy,
//     TimeZone,
//     Alarm,
//     Participant,
//     Location,
//     Resource,
// }
//
// pub struct ParticipantSubcomponents<S> {
//     pub locations: Vec<Location<S>>,
//     pub resources: Vec<Resource<S>>,
// }
//
// impl<S> Default for ParticipantSubcomponents<S> {
//     fn default() -> Self {
//         Self {
//             locations: Default::default(),
//             resources: Default::default(),
//         }
//     }
// }
//
// /// A type representing a set of subcomponents, used when transforming from [`RawComponent`] into a
// /// specific component type.
// trait SubcomponentSet<S> {
//     fn is_empty(&self) -> bool;
//
//     fn try_into_participant_subcomponents(
//         self,
//     ) -> Result<ParticipantSubcomponents<S>, FromRawComponentError<S>>;
// }
//
// impl<S> SubcomponentSet<S> for () {
//     #[inline(always)]
//     fn is_empty(&self) -> bool {
//         true
//     }
//
//     #[inline(always)]
//     fn try_into_participant_subcomponents(
//         self,
//     ) -> Result<ParticipantSubcomponents<S>, FromRawComponentError<S>> {
//         Ok(Default::default())
//     }
// }
//
// impl<S> SubcomponentSet<S> for ParticipantSubcomponents<S> {
//     fn is_empty(&self) -> bool {
//         self.locations.is_empty() && self.resources.is_empty()
//     }
//
//     fn try_into_participant_subcomponents(
//         self,
//     ) -> Result<ParticipantSubcomponents<S>, FromRawComponentError<S>> {
//         Ok(self)
//     }
// }
