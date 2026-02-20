//! Types representing the members of finite sets.
//!
//! These are the "known value" enums for RFC 5545 property values and parameters.
//! All extensible enums are `#[non_exhaustive]` — callers that need to handle unknown
//! values should wrap them with a discriminated union (e.g. `Token<T, S>`).

use strum::{Display, EnumString};

/// An iTIP method (RFC 5546 §1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum Method {
    #[strum(serialize = "PUBLISH")]
    Publish,
    #[strum(serialize = "REQUEST")]
    Request,
    #[strum(serialize = "REPLY")]
    Reply,
    #[strum(serialize = "ADD")]
    Add,
    #[strum(serialize = "CANCEL")]
    Cancel,
    #[strum(serialize = "REFRESH")]
    Refresh,
    #[strum(serialize = "COUNTER")]
    Counter,
    #[strum(serialize = "DECLINECOUNTER")]
    DeclineCounter,
}

/// An unsigned integer in the range `0..=100` (RFC 5545 §3.8.1.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percent(u8);

impl Percent {
    pub const MIN: Self = Percent(0);
    pub const MAX: Self = Percent(100);

    #[inline(always)]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn new(value: u8) -> Option<Self> {
        match value {
            0..=100 => Some(Self(value)),
            _ => None,
        }
    }
}

/// A priority value in the range `0..=9` (RFC 5545 §3.8.1.9).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    #[default]
    Zero,
    A1,
    A2,
    A3,
    B1,
    B2,
    B3,
    C1,
    C2,
    C3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityClass {
    Low,
    Medium,
    High,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self, Self::Zero) || matches!(other, Self::Zero) {
            None
        } else {
            let a = (*self) as u8;
            let b = (*other) as u8;

            match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(std::cmp::Ordering::Greater),
                std::cmp::Ordering::Equal => Some(std::cmp::Ordering::Equal),
                std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Less),
            }
        }
    }
}

impl Priority {
    pub const fn is_low(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::Low))
    }

    pub const fn is_medium(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::Medium))
    }

    pub const fn is_high(self) -> bool {
        matches!(self.into_class(), Some(PriorityClass::High))
    }

    pub const fn into_class(self) -> Option<PriorityClass> {
        match self {
            Self::Zero => None,
            Self::A1 | Self::A2 | Self::A3 | Self::B1 => Some(PriorityClass::High),
            Self::B2 => Some(PriorityClass::Medium),
            Self::B3 | Self::C1 | Self::C2 | Self::C3 => Some(PriorityClass::Low),
        }
    }
}

// ============================================================================
// Version / Gregorian
// ============================================================================

/// The iCalendar version (RFC 5545 §3.7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version {
    /// iCalendar version 2.0.
    V2_0,
}

/// The single allowed value of the CALSCALE property (RFC 5545 §3.7.1).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gregorian;

// ============================================================================
// Encoding
// ============================================================================

/// The possible values of the ENCODING parameter (RFC 5545 §3.2.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum Encoding {
    /// The `8BIT` text encoding (RFC 2045).
    #[strum(serialize = "8BIT")]
    Bit8,
    /// The `BASE64` binary encoding (RFC 4648).
    #[strum(serialize = "BASE64")]
    Base64,
}

// ============================================================================
// TimeTransparency
// ============================================================================

/// The value of the TRANSP property (RFC 5545 §3.8.2.7).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum TimeTransparency {
    /// The event is opaque (consumes time on the calendar).
    #[default]
    #[strum(serialize = "OPAQUE")]
    Opaque,
    /// The event is transparent (does not consume time).
    #[strum(serialize = "TRANSPARENT")]
    Transparent,
}

// ============================================================================
// Status enums
// ============================================================================

/// The status of a VEVENT component (RFC 5545 §3.8.1.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum EventStatus {
    #[strum(serialize = "TENTATIVE")]
    Tentative,
    #[strum(serialize = "CONFIRMED")]
    Confirmed,
    #[strum(serialize = "CANCELLED")]
    Cancelled,
}

/// The status of a VTODO component (RFC 5545 §3.8.1.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum TodoStatus {
    #[strum(serialize = "NEEDS-ACTION")]
    NeedsAction,
    #[strum(serialize = "COMPLETED")]
    Completed,
    #[strum(serialize = "IN-PROCESS")]
    InProcess,
    #[strum(serialize = "CANCELLED")]
    Cancelled,
}

/// The status of a VJOURNAL component (RFC 5545 §3.8.1.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum JournalStatus {
    #[strum(serialize = "DRAFT")]
    Draft,
    #[strum(serialize = "FINAL")]
    Final,
    #[strum(serialize = "CANCELLED")]
    Cancelled,
}

// ============================================================================
// CLASS property value
// ============================================================================

/// The value of the CLASS property (RFC 5545 §3.8.1.3).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ClassValue {
    #[default]
    #[strum(serialize = "PUBLIC")]
    Public,
    #[strum(serialize = "PRIVATE")]
    Private,
    #[strum(serialize = "CONFIDENTIAL")]
    Confidential,
}

// ============================================================================
// Parameter value enums
// ============================================================================

/// The CUTYPE parameter value (RFC 5545 §3.2.3).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum CalendarUserType {
    #[default]
    #[strum(serialize = "INDIVIDUAL")]
    Individual,
    #[strum(serialize = "GROUP")]
    Group,
    #[strum(serialize = "RESOURCE")]
    Resource,
    #[strum(serialize = "ROOM")]
    Room,
    #[strum(serialize = "UNKNOWN")]
    Unknown,
}

/// The ROLE parameter value (RFC 5545 §3.2.16).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipationRole {
    #[strum(serialize = "CHAIR")]
    Chair,
    #[default]
    #[strum(serialize = "REQ-PARTICIPANT")]
    ReqParticipant,
    #[strum(serialize = "OPT-PARTICIPANT")]
    OptParticipant,
    #[strum(serialize = "NON-PARTICIPANT")]
    NonParticipant,
}

/// The PARTSTAT parameter value (RFC 5545 §3.2.12).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipationStatus {
    #[default]
    #[strum(serialize = "NEEDS-ACTION")]
    NeedsAction,
    #[strum(serialize = "ACCEPTED")]
    Accepted,
    #[strum(serialize = "DECLINED")]
    Declined,
    #[strum(serialize = "TENTATIVE")]
    Tentative,
    #[strum(serialize = "DELEGATED")]
    Delegated,
    #[strum(serialize = "COMPLETED")]
    Completed,
    #[strum(serialize = "IN-PROCESS")]
    InProcess,
}

/// The FBTYPE parameter value (RFC 5545 §3.2.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum FreeBusyType {
    #[strum(serialize = "FREE")]
    Free,
    #[strum(serialize = "BUSY")]
    Busy,
    #[strum(serialize = "BUSY-UNAVAILABLE")]
    BusyUnavailable,
    #[strum(serialize = "BUSY-TENTATIVE")]
    BusyTentative,
}

/// The RELTYPE parameter value (RFC 5545 §3.2.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum RelationshipType {
    #[strum(serialize = "PARENT")]
    Parent,
    #[strum(serialize = "CHILD")]
    Child,
    #[strum(serialize = "SIBLING")]
    Sibling,
    /// RFC 9074 §7.1.
    #[strum(serialize = "SNOOZE")]
    Snooze,
}

/// The ACTION property value for alarms (RFC 5545 §3.8.6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum AlarmAction {
    #[strum(serialize = "AUDIO")]
    Audio,
    #[strum(serialize = "DISPLAY")]
    Display,
    #[strum(serialize = "EMAIL")]
    Email,
}

/// The RELATED parameter value for TRIGGER (RFC 5545 §3.8.6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum TriggerRelation {
    #[strum(serialize = "START")]
    Start,
    #[strum(serialize = "END")]
    End,
}

/// The VALUE parameter value (RFC 5545 §3.2.20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ValueType {
    #[strum(serialize = "BINARY")]
    Binary,
    #[strum(serialize = "BOOLEAN")]
    Boolean,
    #[strum(serialize = "CAL-ADDRESS")]
    CalAddress,
    #[strum(serialize = "DATE")]
    Date,
    #[strum(serialize = "DATE-TIME")]
    DateTime,
    #[strum(serialize = "DURATION")]
    Duration,
    #[strum(serialize = "FLOAT")]
    Float,
    #[strum(serialize = "INTEGER")]
    Integer,
    #[strum(serialize = "PERIOD")]
    Period,
    #[strum(serialize = "RECUR")]
    Recur,
    #[strum(serialize = "TEXT")]
    Text,
    #[strum(serialize = "TIME")]
    Time,
    #[strum(serialize = "URI")]
    Uri,
    #[strum(serialize = "UTC-OFFSET")]
    UtcOffset,
}

/// The only possible value of the RANGE parameter (RFC 5545 §3.2.13).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThisAndFuture;

// ============================================================================
// RFC 7986 enums
// ============================================================================

/// The DISPLAY parameter value (RFC 7986 §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum DisplayType {
    #[strum(serialize = "BADGE")]
    Badge,
    #[strum(serialize = "GRAPHIC")]
    Graphic,
    #[strum(serialize = "FULLSIZE")]
    Fullsize,
    #[strum(serialize = "THUMBNAIL")]
    Thumbnail,
}

/// The FEATURE parameter value (RFC 7986 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum FeatureType {
    #[strum(serialize = "AUDIO")]
    Audio,
    #[strum(serialize = "CHAT")]
    Chat,
    #[strum(serialize = "FEED")]
    Feed,
    #[strum(serialize = "MODERATOR")]
    Moderator,
    #[strum(serialize = "PHONE")]
    Phone,
    #[strum(serialize = "SCREEN")]
    Screen,
    #[strum(serialize = "VIDEO")]
    Video,
}

// ============================================================================
// RFC 9073 enums
// ============================================================================

/// The RESOURCE-TYPE property value (RFC 9073 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ResourceType {
    #[strum(serialize = "ROOM")]
    Room,
    #[strum(serialize = "PROJECTOR")]
    Projector,
    #[strum(serialize = "REMOTE-CONFERENCE-AUDIO")]
    RemoteConferenceAudio,
    #[strum(serialize = "REMOTE-CONFERENCE-VIDEO")]
    RemoteConferenceVideo,
}

/// The PARTICIPANT-TYPE property value (RFC 9073 §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipantType {
    #[strum(serialize = "ACTIVE")]
    Active,
    #[strum(serialize = "INACTIVE")]
    Inactive,
    #[strum(serialize = "SPONSOR")]
    Sponsor,
    #[strum(serialize = "CONTACT")]
    Contact,
    #[strum(serialize = "BOOKING-CONTACT")]
    BookingContact,
    #[strum(serialize = "EMERGENCY-CONTACT")]
    EmergencyContact,
    #[strum(serialize = "PUBLICITY-CONTACT")]
    PublicityContact,
    #[strum(serialize = "PLANNER-CONTACT")]
    PlannerContact,
    #[strum(serialize = "PERFORMER")]
    Performer,
    #[strum(serialize = "SPEAKER")]
    Speaker,
}

// ============================================================================
// RFC 9074 enums
// ============================================================================

/// A proximity value (RFC 9074 §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ProximityValue {
    #[strum(serialize = "ARRIVE")]
    Arrive,
    #[strum(serialize = "DEPART")]
    Depart,
    #[strum(serialize = "CONNECT")]
    Connect,
    #[strum(serialize = "DISCONNECT")]
    Disconnect,
}

// ============================================================================
// Unified status
// ============================================================================

/// A unified status value covering events, todos, and journals (RFC 5545 §3.8.1.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum Status {
    #[strum(serialize = "TENTATIVE")]
    Tentative,
    #[strum(serialize = "CONFIRMED")]
    Confirmed,
    #[strum(serialize = "CANCELLED")]
    Cancelled,
    #[strum(serialize = "NEEDS-ACTION")]
    NeedsAction,
    #[strum(serialize = "COMPLETED")]
    Completed,
    #[strum(serialize = "IN-PROCESS")]
    InProcess,
    #[strum(serialize = "DRAFT")]
    Draft,
    #[strum(serialize = "FINAL")]
    Final,
}

// ============================================================================
// Alarm action markers
// ============================================================================

/// The AUDIO alarm action marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct AudioAction;

/// The DISPLAY alarm action marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct DisplayAction;

/// The EMAIL alarm action marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct EmailAction;

/// An unknown alarm action, either IANA-registered or experimental.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub const fn kind(&self) -> crate::string::NameKind {
        match self {
            UnknownAction::Iana(_) => crate::string::NameKind::Iana,
            UnknownAction::X(_) => crate::string::NameKind::X,
        }
    }

    pub fn into_inner(self) -> S {
        match self {
            UnknownAction::Iana(action) | UnknownAction::X(action) => action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
