//! Types for values which appear in sets.

pub use calendar_types::{css::Css3Color, set::LocationType};
pub use rfc5545_types::set::{Percent, Priority};

/// A value which may appear in the `relation` field of a `Relation` object (RFC 8984 §1.4.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RelationValue<S> {
    First,
    Next,
    Child,
    Parent,
    Other(S),
}

/// The intended purpose of a link to an image (RFC 8984 §1.4.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DisplayPurpose<S> {
    Badge,
    Graphic,
    FullSize,
    Thumbnail,
    Other(S),
}

/// An iCalendar method (RFC 8984 §4.1.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Method<S> {
    Publish,
    Request,
    Reply,
    Add,
    Cancel,
    Refresh,
    Counter,
    DeclineCounter,
    Other(S),
}

/// A free/busy status (RFC 8984 §4.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FreeBusyStatus<S> {
    Free,
    Busy,
    Other(S),
}

/// A privacy level (RFC 8984 §4.4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Privacy<S> {
    Public,
    Private,
    Secret,
    Other(S),
}

/// An event status (RFC 8984 §5.1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EventStatus<S> {
    Confirmed,
    Cancelled,
    Tentative,
    Other(S),
}

/// A task progress status (RFC 8984 §5.2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TaskProgress<S> {
    NeedsAction,
    InProcess,
    Completed,
    Cancelled,
    Other(S),
}

/// A feature supported by a virutal location (RFC 8984 §4.2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum VirtualLocationFeature<S> {
    Audio,
    Chat,
    Feed,
    Moderator,
    Phone,
    Screen,
    Video,
    Other(S),
}

/// A reply method identifier (RFC 8984 §4.4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ReplyMethod<S> {
    Imip,
    Web,
    Other(S),
}

/// The kind of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ParticipantKind<S> {
    Individual,
    Group,
    Location,
    Resource,
    Other(S),
}

/// The role of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ParticipantRole<S> {
    Owner,
    Attendee,
    Optional,
    Informational,
    Chair,
    Contact,
    Other(S),
}

/// The status of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ParticipationStatus<S> {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
    Other(S),
}

/// The agent responsible for sending scheduling messages to a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ScheduleAgent<S> {
    Server,
    Client,
    None,
    Other(S),
}

/// The time property that an alert is relative to (RFC 8984 §4.5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AlertRelativeTo<S> {
    Start,
    End,
    Other(S),
}

/// The action by which an alert is conveyed (RFC 8984 §4.5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AlertAction<S> {
    Display,
    Email,
    Other(S),
}

/// An RGB color value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// A color, which may be either a CSS3 color name or an RGB value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    Css(Css3Color),
    Rgb(Rgb),
}
