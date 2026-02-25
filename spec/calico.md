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

The date type is defined by RFC 5545 §3.3.4. 

### Date-Time

### Duration

### Float

### Integer

### Period of Time

### Recurrence Rule

### Text

### Time

### URI

### UTC Offset

## Components

# Parsing

# Rendering
