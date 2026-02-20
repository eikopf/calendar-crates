//! [JSCalendar (RFC 8984)](https://datatracker.ietf.org/doc/html/rfc8984) data model in Rust.
//!
//! This crate provides strongly-typed Rust representations of JSCalendar objects —
//! [`Event`](model::object::Event), [`Task`](model::object::Task), and
//! [`Group`](model::object::Group) — along with traits for converting to and from
//! JSON values.
//!
//! # Parser-agnostic design
//!
//! All object types are generic over `V: JsonValue`, meaning they are not tied to any
//! particular JSON library. The [`json`] module defines the [`DestructibleJsonValue`] and
//! [`ConstructibleJsonValue`] traits that abstract over JSON deserialization and
//! serialization respectively. Any JSON library can be used by implementing these traits.
//!
//! [`DestructibleJsonValue`]: json::DestructibleJsonValue
//! [`ConstructibleJsonValue`]: json::ConstructibleJsonValue
//!
//! # Feature flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | `serde_json` | off | Implements `JsonValue`, `DestructibleJsonValue`, and `ConstructibleJsonValue` for `serde_json::Value` |
//!
//! # Example
//!
//! Parsing a JSCalendar event from JSON and serializing it back:
//!
//! ```
//! # #[cfg(feature = "serde_json")]
//! # {
//! use serde_json::json;
//! use jscalendar::json::{TryFromJson, IntoJson};
//! use jscalendar::model::object::Event;
//!
//! let input = json!({
//!     "@type": "Event",
//!     "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
//!     "updated": "2020-01-02T18:23:04Z",
//!     "title": "Team meeting",
//!     "start": "2020-01-15T13:00:00",
//!     "timeZone": "America/New_York",
//!     "duration": "PT1H"
//! });
//!
//! let event: Event<serde_json::Value> = Event::try_from_json(input).unwrap();
//! assert_eq!(event.title(), Some(&String::from("Team meeting")));
//!
//! let json_value: serde_json::Value = event.into_json();
//! assert_eq!(json_value["title"], "Team meeting");
//! # }
//! ```
//!
//! # Scope
//!
//! This crate covers the JSCalendar **data model** and **JSON conversion** only.
//! It does not provide recurrence expansion, IANA time zone resolution, or
//! iCalendar (RFC 5545) conversion.
//!
//! # Modules
//!
//! - [`json`] — JSON value traits and conversion infrastructure
//! - [`model`] — JSCalendar object types, enumerations, and string newtypes
//! - [`parser`] — Incremental parsers for date/time and duration strings

pub mod json;
pub mod model;
pub mod parser;
