//! Types for values which appear in sets.

pub use calendar_types::{
    css::Css3Color,
    set::{LinkRelation, LocationType, Token},
};
pub use rfc5545_types::set::{Method, Percent, Priority};
use strum::EnumString;
use thiserror::Error;

use crate::json::{DestructibleJsonValue, TryFromJson, TypeErrorOr, UnsignedInt};

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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("not a known CSS3 color name or #RRGGBB hex string: {0:?}")]
pub struct InvalidColorError(pub Box<str>);

impl<V: DestructibleJsonValue> TryFromJson<V> for Color {
    type Error = TypeErrorOr<InvalidColorError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let s = value.try_into_string()?;
        // Try CSS3 name first (case-insensitive)
        if let Ok(css) = s.as_ref().parse::<Css3Color>() {
            return Ok(Color::Css(css));
        }
        // Try #RRGGBB
        if let Some(hex) = s.as_ref().strip_prefix('#') {
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Ok(Color::Rgb(Rgb { red: r, green: g, blue: b }));
                }
            }
        }
        Err(TypeErrorOr::Other(InvalidColorError(
            String::from(s.as_ref()).into_boxed_str(),
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("priority must be an integer in the range 0..=9, got {0}")]
pub struct InvalidPriorityError(u64);

impl<V: DestructibleJsonValue> TryFromJson<V> for Priority {
    type Error = TypeErrorOr<InvalidPriorityError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let n = UnsignedInt::try_from_json(value).map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(InvalidPriorityError(u64::MAX)),
        })?;
        match n.get() {
            0 => Ok(Priority::Zero),
            1 => Ok(Priority::A1),
            2 => Ok(Priority::A2),
            3 => Ok(Priority::A3),
            4 => Ok(Priority::B1),
            5 => Ok(Priority::B2),
            6 => Ok(Priority::B3),
            7 => Ok(Priority::C1),
            8 => Ok(Priority::C2),
            9 => Ok(Priority::C3),
            v => Err(TypeErrorOr::Other(InvalidPriorityError(v))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("percent must be an integer in the range 0..=100, got {0}")]
pub struct InvalidPercentError(u64);

impl<V: DestructibleJsonValue> TryFromJson<V> for Percent {
    type Error = TypeErrorOr<InvalidPercentError>;

    fn try_from_json(value: V) -> Result<Self, Self::Error> {
        let n = UnsignedInt::try_from_json(value).map_err(|e| match e {
            TypeErrorOr::TypeError(t) => TypeErrorOr::TypeError(t),
            TypeErrorOr::Other(_) => TypeErrorOr::Other(InvalidPercentError(u64::MAX)),
        })?;
        let v = u8::try_from(n.get())
            .map_err(|_| TypeErrorOr::Other(InvalidPercentError(n.get())))?;
        Percent::new(v)
            .ok_or(InvalidPercentError(n.get()))
            .map_err(TypeErrorOr::Other)
    }
}
