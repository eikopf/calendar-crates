//! Date, time, and string primitive types for calendar data.
//!
//! This crate provides the foundational types shared by both iCalendar (RFC 5545)
//! and JSCalendar (RFC 8984) implementations. It includes:
//!
//! - **Date and time types** ([`time`]): [`Year`](time::Year), [`Month`](time::Month),
//!   [`Day`](time::Day), [`Hour`](time::Hour), [`Minute`](time::Minute),
//!   [`Second`](time::Second), [`Date`](time::Date), [`Time`](time::Time), and
//!   [`DateTime`](time::DateTime) with compile-time timezone markers.
//! - **Duration types** ([`duration`]): [`Duration`](duration::Duration) and
//!   [`SignedDuration`](duration::SignedDuration) following RFC 8984 §1.4.6–7.
//! - **String types** ([`string`]): validated [`Uid`](string::Uid) and [`Uri`](string::Uri)
//!   newtypes.
//! - **Primitives** ([`primitive`]): [`Sign`](primitive::Sign) for positive/negative values.
//! - **CSS colors** ([`css`]): [`Css3Color`](css::Css3Color) enum for the W3C CSS3 color names.
//! - **Token sets** ([`set`]): [`Token`](set::Token) for extensible enum values, and
//!   IANA registry types ([`LinkRelation`](set::LinkRelation),
//!   [`LocationType`](set::LocationType)).

pub mod css;
pub mod duration;
pub mod primitive;
pub mod set;
pub mod string;
pub mod time;
