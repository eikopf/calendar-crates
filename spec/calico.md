# Introduction

Calico (henceforth also `calico`) is a Rust implementation of [the iCalendar data format](#icalendar). It is concerned with the following components:

1. [A set of Rust types describing the iCalendar data model](#model-types).
2. [A parser for converting iCalendar data to the corresponding Rust types](#parsing).
3. [A renderer for converting Rust types into iCalendar data](#rendering).

# iCalendar

The iCalendar data format is defined primarily by [RFC 5545](https://datatracker.ietf.org/doc/html/rfc5545), a document which obsoleted the earlier [RFC 2445](https://datatracker.ietf.org/doc/html/rfc2445). The following RFCs have updated the standard:

- [RFC 5546](https://datatracker.ietf.org/doc/html/rfc5546), *iCalendar Transport-Independent Interoperability Protocol (iTIP)*.
- [RFC 6868](https://datatracker.ietf.org/doc/html/rfc6868), *Parameter Value Encoding in iCalendar and vCard*.
- [RFC 7529](https://datatracker.ietf.org/doc/html/rfc7529), *Non-Gregorian Recurrence Rules in the Internet Calendaring and Scheduling Core Object Specification (iCalendar)*.
- [RFC 7953](https://datatracker.ietf.org/doc/html/rfc7953), *Calendar Availability*.
- [RFC 7986](https://datatracker.ietf.org/doc/html/rfc7986), *New Properties for iCalendar*.
- [RFC 9073](https://datatracker.ietf.org/doc/html/rfc9073), *Event Publishing Extensions to iCalendar*.
- [RFC 9074](https://datatracker.ietf.org/doc/html/rfc9074), *"VALARM" Extensions to iCalendar*.
- [RFC 9253](https://datatracker.ietf.org/doc/html/rfc9253), *Support for iCalendar Relationships*.

These "extension" RFCs are backwards-compatible, and hence are optional from the perspective of an implementation.

# Model Types

## Primitive Values

The following are primitive value types defined by RFC 5545 §3.3.

### Binary

The binary value type is defined by RFC 5545 §3.3.1. It is taken to represent binary data, for which RFC 5545 does not specify a maximum length. We assume that it is extremely unlikely any value will exceed the maximum possible length of a `Vec<T>` on 32-bit platforms.

r[model.prim.binary.domain]
A type representing a binary value MUST admit arbitrary sequences of bytes up to a length of at least 2^31 - 1.

Still, if a value *does* exceed this length then we must account for it.

r[model.prim.binary.truncation]
A type representing a binary value MUST NOT silently truncate byte sequences with length exceeding 2^31 - 1.

Panicking is permissible if truncation is otherwise unavoidable.

### Boolean

The boolean value type is defined by RFC 5545 §3.3.2. It represents a type with exactly two values, equivalent to the `true` and `false` values of Rust's `bool` type.

r[model.prim.boolean.domain]
A type representing a boolean value MUST admit only two values, corresponding to `true` and `false`.

This type has the exact semantics you would expect; it's very uncomplicated.

### Calendar User Address

The calendar user address type is defined by RFC 5545 §3.3.3. It represents a [URI](#model-types--primitive-values--uri) with the `mailto` scheme.

r[model.prim.cal-address.domain]
A type representing a calendar user address MUST admit only URIs with the `mailto` scheme.

### Date

The date type is defined by RFC 5545 §3.3.4. It represents a calendar date with year, month, and day components.

r[model.prim.date.year]
A type representing a date MUST admit year values in the range 0 to 9999 inclusive.

r[model.prim.date.month]
A type representing a date MUST admit month values in the range 1 to 12 inclusive.

r[model.prim.date.day]
A type representing a date MUST admit day values in the range 1 to 31 inclusive.

r[model.prim.date.validity]
A type representing a date MUST reject day values that are invalid for the given month and year. In particular, it MUST account for leap years as defined by RFC 3339 Appendix C: a year is a leap year if it is divisible by 4 and either not divisible by 100 or divisible by 400.

### Date-Time

The date-time type is defined by RFC 5545 §3.3.5. It is a composite of a [date](#model-types--primitive-values--date) and a [time](#model-types--primitive-values--time), along with a marker indicating whether the value is in UTC or local (floating) time.

r[model.prim.date-time.domain]
A type representing a date-time MUST admit any combination of a valid date value and a valid time value.

r[model.prim.date-time.marker]
A type representing a date-time MUST distinguish between UTC and local (floating) time.

RFC 5545 defines three forms of date-time: local (floating) time, UTC time (indicated by a `Z` suffix), and local time with a timezone reference (indicated by the `TZID` property parameter). From the perspective of the value type, the first and third forms are identical; the timezone reference is carried by the property parameter, not the value itself.

r[model.prim.date-time.leap-seconds]
A type representing a date-time is NOT REQUIRED to validate that a leap second (i.e. a time with second value 60) actually occurred on the given date. RFC 3339 §5.7 documents when leap seconds are valid, but enforcing this is not required.

### Duration

The duration type is defined by RFC 5545 §3.3.6. It represents a span of time, which may be either *nominal* (expressed in weeks and/or days) or *exact* (expressed in hours, minutes, and/or seconds). This distinction is significant because nominal durations are calendar-aware: a "day" may not always be 24 hours due to daylight saving time transitions.

r[model.prim.duration.sign]
A type representing a duration MUST admit both positive and negative durations.

r[model.prim.duration.nominal]
A type representing a nominal duration MUST admit non-negative integer values for weeks and days, and MAY additionally include an exact time component.

r[model.prim.duration.exact]
A type representing an exact duration MUST admit non-negative integer values for hours, minutes, and seconds.

r[model.prim.duration.distinction]
A type representing a duration MUST distinguish between nominal and exact durations at the type level.

A nominal duration of "1 day" is not the same as an exact duration of "24 hours", even though they are often equivalent. The distinction matters for recurrence calculations across daylight saving time boundaries.

### Float

The float type is defined by RFC 5545 §3.3.7. It represents a real number in decimal notation.

r[model.prim.float.domain]
A type representing a float MUST admit signed decimal numbers with at least 64-bit IEEE 754 precision.

r[model.prim.float.no-special]
A type representing a float MUST NOT admit special IEEE 754 values (NaN, positive infinity, negative infinity) through parsing.

The iCalendar text format does not define a representation for these special values. However, since the underlying storage is an `f64`, they may still arise through arithmetic operations on parsed values.

### Integer

The integer type is defined by RFC 5545 §3.3.8. It represents a signed integer value.

r[model.prim.integer.domain]
A type representing an integer MUST admit signed values in the range -2,147,483,648 to 2,147,483,647 inclusive (i.e. the range of a 32-bit two's complement integer).

This corresponds to Rust's `i32` type.

### Period of Time

The period of time type is defined by RFC 5545 §3.3.9. It represents a precise span of time, given either as an explicit start and end or as a start and a duration.

r[model.prim.period.explicit]
A type representing a period of time MUST admit an explicit form consisting of a start [date-time](#model-types--primitive-values--date-time) and an end date-time.

r[model.prim.period.start-duration]
A type representing a period of time MUST admit a start form consisting of a start [date-time](#model-types--primitive-values--date-time) and a [duration](#model-types--primitive-values--duration).

r[model.prim.period.distinction]
A type representing a period of time MUST distinguish between the explicit and start-duration forms at the type level.

### Recurrence Rule

The recurrence rule type is defined by RFC 5545 §3.3.10. It describes a pattern for recurring events, to-dos, or journal entries. Due to the complexity of this type, it is described in terms of its constituent parts.

r[model.prim.recur.freq]
A type representing a recurrence rule MUST include a frequency, which MUST be one of: secondly, minutely, hourly, daily, weekly, monthly, or yearly.

r[model.prim.recur.interval]
A type representing a recurrence rule MAY include an interval (a positive integer), indicating that occurrences repeat every *n*th frequency period. The default interval is 1.

r[model.prim.recur.termination]
A type representing a recurrence rule MAY include a termination condition, which MUST be either a count (a non-negative integer) or an until date/date-time, but not both.

r[model.prim.recur.by-rules]
A type representing a recurrence rule MAY include any combination of the following by-rules: BYSECOND, BYMINUTE, BYHOUR, BYDAY, BYMONTH, and BYSETPOS.

r[model.prim.recur.freq-by-rules]
A type representing a recurrence rule MAY include frequency-dependent by-rules. The following by-rules are only valid for certain frequencies:

- BYMONTHDAY: valid for secondly, minutely, hourly, daily, and monthly frequencies.
- BYYEARDAY: valid for secondly, minutely, and hourly frequencies.
- BYWEEKNO: valid only for the yearly frequency.

r[model.prim.recur.freq-by-rules-enforcement]
A type representing a recurrence rule SHOULD enforce the frequency-dependent by-rule restrictions at the type level.

r[model.prim.recur.week-start]
A type representing a recurrence rule MAY include a week start day, indicating which day of the week is considered the first.

### Text

The text type is defined by RFC 5545 §3.3.11. It represents human-readable text content.

r[model.prim.text.domain]
A type representing a text value MUST admit all Unicode characters except ASCII control characters (U+0000 through U+001F and U+007F), with the exceptions of HTAB (U+0009) and LF (U+000A) which MUST be admitted.

r[model.prim.text.escaping]
When parsing the iCalendar text format, a text parser MUST recognise the following escape sequences: `\\` (backslash), `\n` and `\N` (newline), `\;` (semicolon), `\,` (comma), `\"` (double quote), and `\ ` (space).

The semicolon, comma, and backslash characters have special meaning in the iCalendar text format and must be escaped when they appear in text values.

### Time

The time type is defined by RFC 5545 §3.3.12. It represents a time of day.

r[model.prim.time.hour]
A type representing a time MUST admit hour values in the range 0 to 23 inclusive.

r[model.prim.time.minute]
A type representing a time MUST admit minute values in the range 0 to 59 inclusive.

r[model.prim.time.second]
A type representing a time MUST admit second values in the range 0 to 60 inclusive, where 60 represents a leap second.

r[model.prim.time.marker]
A type representing a time MUST distinguish between UTC and local (floating) time through a time format marker, as with [date-time](#model-types--primitive-values--date-time).

### URI

The URI type is defined by RFC 5545 §3.3.13, with reference to RFC 3986. It represents a Uniform Resource Identifier.

r[model.prim.uri.domain]
A type representing a URI MUST admit any valid UTF-8 string that conforms to the syntax defined in RFC 3986.

r[model.prim.uri.opaque]
A type representing a URI is NOT REQUIRED to parse or validate the internal structure of the URI beyond accepting the characters permitted by RFC 3986.

The [calendar user address](#model-types--primitive-values--calendar-user-address) type is a special case of URI with the `mailto` scheme. Since the grammar for both types is identical at the value level, they share a parser.

### UTC Offset

The UTC offset type is defined by RFC 5545 §3.3.14. It represents a signed offset from UTC, used to describe time zone offsets.

r[model.prim.utc-offset.sign]
A type representing a UTC offset MUST include a sign (positive or negative), where positive indicates east of UTC and negative indicates west of UTC.

r[model.prim.utc-offset.hour]
A type representing a UTC offset MUST admit hour values in the range 0 to 23 inclusive.

r[model.prim.utc-offset.minute]
A type representing a UTC offset MUST admit minute values in the range 0 to 59 inclusive.

r[model.prim.utc-offset.second]
A type representing a UTC offset MUST admit second values in the range 0 to 59 inclusive. Leap seconds are not permitted in UTC offsets.

r[model.prim.utc-offset.negative-zero]
A type representing a UTC offset MUST reject negative zero (i.e. an offset of `-000000`). The zero offset MUST only be represented with a positive sign.

## Components

# Parsing

# Rendering
