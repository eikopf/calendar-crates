//! Types for values which appear in sets.

pub use calendar_types::{
    css::Css3Color,
    set::{LinkRelation, LocationType, Token},
};
pub use rfc5545_types::set::{Method, Percent, Priority};
use strum::EnumString;

/// A value which may appear in the `relation` field of a `Relation` object (RFC 8984 §1.4.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum RelationValue {
    First,
    Next,
    Child,
    Parent,
}

/// The intended purpose of a link to an image (RFC 8984 §1.4.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum DisplayPurpose {
    Badge,
    Graphic,
    FullSize,
    Thumbnail,
}

/// A free/busy status (RFC 8984 §4.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum FreeBusyStatus {
    Free,
    Busy,
}

/// A privacy level (RFC 8984 §4.4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum Privacy {
    Public,
    Private,
    Secret,
}

/// An event status (RFC 8984 §5.1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum EventStatus {
    Confirmed,
    Cancelled,
    Tentative,
}

/// A task progress status (RFC 8984 §5.2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum TaskProgress {
    #[strum(serialize = "needs-action")]
    NeedsAction,
    #[strum(serialize = "in-process")]
    InProcess,
    Completed,
    Cancelled,
}

/// A feature supported by a virutal location (RFC 8984 §4.2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum VirtualLocationFeature {
    Audio,
    Chat,
    Feed,
    Moderator,
    Phone,
    Screen,
    Video,
}

/// A reply method identifier (RFC 8984 §4.4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ReplyMethod {
    Imip,
    Web,
}

/// The kind of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipantKind {
    Individual,
    Group,
    Location,
    Resource,
}

/// The role of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipantRole {
    Owner,
    Attendee,
    Optional,
    Informational,
    Chair,
    Contact,
}

/// The status of a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ParticipationStatus {
    #[strum(serialize = "needs-action")]
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
}

/// The agent responsible for sending scheduling messages to a participant (RFC 8984 §4.4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum ScheduleAgent {
    Server,
    Client,
    None,
}

/// The time property that an alert is relative to (RFC 8984 §4.5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum AlertRelativeTo {
    Start,
    End,
}

/// The action by which an alert is conveyed (RFC 8984 §4.5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum AlertAction {
    Display,
    Email,
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
