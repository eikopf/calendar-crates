//! iCalendar (RFC 5545) data model types.
//!
//! This crate provides the type-level representation of iCalendar components,
//! properties, and parameters. It builds on [`calendar_types`] for date/time
//! primitives and adds:
//!
//! - **Recurrence rules** ([`rrule`]): [`RRule`](rrule::RRule) with frequency-dependent
//!   BYxxx rules, efficient bitset types ([`SecondSet`](rrule::SecondSet),
//!   [`MinuteSet`](rrule::MinuteSet), [`HourSet`](rrule::HourSet),
//!   [`MonthSet`](rrule::MonthSet), [`MonthDaySet`](rrule::MonthDaySet),
//!   [`WeekNoSet`](rrule::WeekNoSet)), and the
//!   [`WeekdayNumSet`](rrule::weekday_num_set::WeekdayNumSet).
//! - **Time types** ([`time`]): [`DateTimeOrDate`](time::DateTimeOrDate),
//!   [`Period`](time::Period), [`RDate`](time::RDate), [`TriggerValue`](time::TriggerValue),
//!   and [`UtcOffset`](time::UtcOffset).
//! - **Property value enums** ([`set`]): status types, parameter value enums, and
//!   alarm action markers.
//! - **String types** ([`string`]): validated iCalendar string newtypes
//!   ([`ParamText`](string::ParamText), [`Text`](string::Text),
//!   [`Name`](string::Name), [`CaselessStr`](string::CaselessStr)).
//! - **Compound values** ([`value`]): [`Geo`](value::Geo),
//!   [`Attachment`](value::Attachment), and [`FormatType`](value::FormatType).
//! - **Request status** ([`request_status`]): [`RequestStatus`](request_status::RequestStatus)
//!   and [`StatusCode`](request_status::StatusCode).
//! - **Primitives** ([`primitive`]): type aliases for iCalendar integer and float values.

pub mod request_status;
pub mod rrule;
pub mod set;
pub mod string;
pub mod time;
pub mod value;

/// iCalendar primitive value types.
pub mod primitive {
    use std::num::NonZero;

    /// A signed 32-bit integer (RFC 5545 §3.3.8).
    pub type Integer = i32;
    /// A 64-bit floating-point number (RFC 5545 §3.3.7).
    pub type Float = f64;
    /// A positive integer (nonzero unsigned 32-bit).
    pub type PositiveInteger = NonZero<u32>;
}
