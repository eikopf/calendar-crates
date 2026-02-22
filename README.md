A collection of crates for working with calendar data formats, primarily iCalendar (RFC 5545) and JSCalendar (RFC 8984).

## Crates

### `calendar-types`

Common types not belonging to a single calendaring RFC. These include

- Date, time, and duration types based on RFC 3339 (e.g. Year, Hour, Duration, DateTime).
- String types used by multiple RFCs (e.g. Uid, Uri, LanguageTag).
- CSS3 color names and IANA token types (e.g. LinkRelation, LocationType).
- Other common primitives (e.g. Sign).

### `rfc5545-types`

Types specific to iCalendar (RFC 5545). These include

- Recurrence rule types and representations (e.g. RRule, WeekdayNum, MonthDaySet).
- Efficient bitset types for time components (e.g. SecondSet, MinuteSet, HourSet).
- Property value types (e.g. status, trigger, period, attachment).
- String types used by RFC 5545 (e.g. ParamText, Text, Name).

### `calico`

A parser, printer, and data model for iCalendar. Supports RFC 5545, RFC 5546, and RFC 7986.

### `jscalendar`

A parser-agnostic implementation of JSCalendar (RFC 8984) with optional serde_json support.
