A collection of crates for working with calendar data formats, primarily iCalendar (RFC 5545) and JSCalendar (RFC 8984).

## Crates

### `calendar-types`

Common types not belonging to a single calendaring RFC. These include

- Date, time, and duration types based on RFC 3339 (e.g. Year, Hour, Duration).
- String types used by multiple RFCs (e.g. Uid and Uri).
- Other common primitives (e.g. Sign).

### `rfc5545-types`

Types specific to iCalendar (RFC 5545). These include

- Data model types.
- Recurrence rule types and representations (e.g. RRule, WeekdayNum, MonthDaySet).
- String types used by RFC 5545 (e.g. ParamText).

### `jscalendar`

A parser-agnostic implementation of JSCalendar (RFC 8984).
