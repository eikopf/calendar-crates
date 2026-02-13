//! Types for values which appear in sets.

pub use calendar_types::css::Css3Color;
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

/// A reply method identifier (RFC 8984 §4.4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ReplyMethod<S> {
    Imip,
    Web,
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
