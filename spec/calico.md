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
A type representing a date MUST only admit year values in the range 0 to 9999 inclusive.

r[model.prim.date.month]
A type representing a date MUST only admit month values in the range 1 to 12 inclusive.

r[model.prim.date.day]
A type representing a date MUST only admit day values in the range 1 to 31 inclusive.

Not all days are valid for a given year and month. This is both because the months differ in length from one another, and because February has an extra day on leap years. The exact rule for deciding whether a year in the range from 0 to 9999 is a leap year is given in Appendix C of RFC 3339.

r[model.prim.date.validity]
A type representing a date MUST reject day values that are invalid for the given month and year.

### Date-Time

The date-time type is defined by RFC 5545 §3.3.5. It is a composite of a [date](#model-types--primitive-values--date), a [time](#model-types--primitive-values--time), and a marker indicating whether the value is in UTC or local (floating) time. While it is technically possible to implement this type in such a way that the time value is known to have definitely occurred on the date represented by the date value, it would be extremely difficult in practice.

r[model.prim.date-time.domain]
A type representing a date-time MUST admit any combination of a valid date value and a valid time value. In particular, checking for the validity of a leap second on a given date is not required.

r[model.prim.date-time.marker]
A type representing a date-time MUST distinguish between UTC and local (floating) time.

Whereas RFC 3339 defines a third kind of marker for directly encoding UTC offsets into date-time values, such markers are explicitly invalid in iCalendar.

r[model.prim.date-time.utc-offset-marker]
A type representing a date-time MUST NOT admit any kind of marker except the UTC and local (floating) markers.

### Duration

The duration type is defined by RFC 5545 §3.3.6. It represents a span of time, which may be either *nominal* (expressed in weeks and/or days) or *exact* (expressed in hours, minutes, and/or seconds). This distinction is significant because nominal durations do not correspond exactly to exact durations; the length of a day or year can and will drift from one to the next.

r[model.prim.duration.distinction]
A type representing a duration MUST distinguish between nominal and exact durations at the type level.

r[model.prim.duration.nominal]
A type representing a nominal duration MUST admit non-negative integer values for weeks and days, and MUST admit an optional exact time component.

r[model.prim.duration.exact]
A type representing an exact duration MUST admit non-negative integer values for hours, minutes, and seconds.

iCalendar has no notion of an "unsigned duration," although later standards (in particular JSCalendar) have since introduced the idea.

r[model.prim.duration.sign]
A type representing a duration MUST admit both positive and negative durations.

### Float

The float type is defined by RFC 5545 §3.3.7. It represents a real number in decimal notation, although the standard leaves the precision unspecified. In practice all real implementations use 64-bit IEEE 754 floating-point numbers to represent this type, so for compatibility we will do the same.

r[model.prim.float.domain]
A type representing a float MUST admit signed decimal numbers with at least 64-bit IEEE 754 precision.

r[model.prim.float.no-special]
A type representing a float MUST NOT admit special IEEE 754 values (NaN, positive infinity, negative infinity) through parsing.

### Integer

The integer type is defined by RFC 5545 §3.3.8. It represents a signed integer value, in the range from -2^32 to 2^32 - 1.

r[model.prim.integer.domain]
A type representing an integer MUST admit signed values in the range -2,147,483,648 to 2,147,483,647 inclusive.

### Period of Time

The period of time type is defined by RFC 5545 §3.3.9. It represents a span of time, given either as an explicit start and end or as a start and a duration.

r[model.prim.period.explicit]
A type representing a period of time MUST admit a form consisting of a start [date-time](#model-types--primitive-values--date-time) and an end date-time.

r[model.prim.period.start-duration]
A type representing a period of time MUST admit a form consisting of a start [date-time](#model-types--primitive-values--date-time) and a [duration](#model-types--primitive-values--duration).

r[model.prim.period.distinction]
A type representing a period of time MUST distinguish between the explicit and start-duration forms at the type level.

### Recurrence Rule

TODO: rewrite this definition pulling from the standard rather than the current implementation

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
A type representing a text value MUST admit all UTF-8 strings except those which contain ASCII control characters (U+0000 through U+001F, and U+007F), not including HTAB (U+0009) and LF (U+000A) which MUST be admitted.


### Time

The time type is defined by RFC 5545 §3.3.12. It represents a time of day in terms of hours, minutes, and seconds.

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

### UTC Offset

The UTC offset type is defined by RFC 5545 §3.3.14. It represents a signed offset from UTC, used to describe time zone offsets.

r[model.prim.utc-offset.sign]
A type representing a UTC offset MUST include a sign (positive or negative).

r[model.prim.utc-offset.hour]
A type representing a UTC offset MUST admit hour values in the range 0 to 23 inclusive.

r[model.prim.utc-offset.minute]
A type representing a UTC offset MUST admit minute values in the range 0 to 59 inclusive.

r[model.prim.utc-offset.second]
A type representing a UTC offset MUST admit second values in the range 0 to 59 inclusive. Leap seconds are not permitted in UTC offsets.

r[model.prim.utc-offset.negative-zero]
A type representing a UTC offset MUST reject negative zero (i.e. an offset of `-000000`). The zero offset MUST only be represented with a positive sign.

## Implied String Types
Although RFC 5545 does not explicitly define string subtypes other than [Text](#model-types--primitive-values--text), it is still useful to explicitly define the string types which are implied by various grammar rules in the document.

### Names

As described by the `name` and `param-name` rules in RFC 5545 §3.1, a name is a non-empty string whose characters are restricted to the ASCII alphanumeric characters and the hyphen (U+002D).

r[string.name.domain]
A name is a non-empty string containing only ASCII alphanumeric characters and the hyphen (U+002D).

There are several places in RFC 5545 where a grammar element admits only the `iana-token` and `x-name` rules (e.g. the `iana-comp` and `x-comp` rules in §3.6); in such cases it is usually correct to represent the corresponding strings with a name.

## The iCalendar Object

Whereas primitive values sit at the lowest level of the iCalendar data model, the iCalendar object is the principal and highest-level value; it is defined by RFC 5545 §3.4. Although the document treats it separately, the iCalendar object is essentially a [component](#model-types--components).

r[model.icalendar-object.kind]
An iCalendar object is a component.

When 0 or more iCalendar objects are placed in a sequence, the resulting value is called an iCalendar stream.

r[model.icalendar-object.stream]
A type representing an iCalendar stream MUST admit arbitrary sequences of iCalendar objects.

## Components

Components are defined by RFC 5545 §3.6, and consist of a name (e.g. `VEVENT`), a list of [properties](#model-types--properties), and a list of subcomponents. These constituent parts may be restricted based on the name of the component.

r[model.component.name]
A component MUST have a name.

r[model.component.name.domain]
The domain of a component's name MUST be the same as [the implied name string type](#model-types--implied-string-types--names).

It's actually quite difficult to articulate precise requirements about the properties of a component, for a few different reasons. You might expect that the properties of a component just correspond to record fields, but properties are in general allowed to occur multiple times, and unless a property is explicitly disallowed it may always occur.

r[model.component.properties]
A component MUST have a collection of properties.

r[model.component.properties.domain]
Unless a restriction is given for a specific property, a component MUST admit any property to occur an arbitrary number of times.

Subcomponents are a little simpler, although these requirements are still inferred rather than explicitly given by RFC 5545.

r[model.component.subcomponents]
A component MUST have a collection of subcomponents.

### Calendar

A calendar component (identified by the name `VCALENDAR`) represents an [iCalendar object](#model-types--the-icalendar-object).

r[model.component.calendar.name]
The name of a component representing an iCalendar object MUST be `VCALENDAR`.

This component type has two mandatory properties.

r[model.component.calendar.prodid]
A component representing an iCalendar object MUST admit the `PRODID` property exactly once.

r[model.component.calendar.version]
A component representing an iCalendar object MUST admit the `VERSION` property exactly once.

And two optional properties.

r[model.component.calendar.calscale]
A component representing an iCalendar object MUST admit the `CALSCALE` property at most once.

r[model.component.calendar.method]
A component representing an iCalendar object MUST admit the `METHOD` property at most once.

The subcomponents of this component type may be event, todo, jounal, free-busy, time zone, or extension components.

r[model.component.calendar.subcomponents]
A component representing an iCalendar object MUST admit the VEVENT, VTODO, VJOURNAL, VFREEBUSY, and VTIMEZONE components as subcomponents, as well as any extension component.

### Event

### Todo

### Journal

### Free-Busy

### Time Zone

### Extension Components

## Properties

# Parsing

# Rendering
