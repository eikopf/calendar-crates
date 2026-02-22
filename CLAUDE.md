# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates
cargo build

# Run all tests (use --all-features to include serde_json tests)
cargo test --all-features

# Run tests for a specific crate
cargo test -p calendar-types
cargo test -p rfc5545-types
cargo test -p jscalendar
cargo test -p calico

# Run a specific test
cargo test -p rfc5545-types behavior_with_table

# Check without building
cargo check
```

## Crate Structure

This workspace contains four crates for working with calendar data formats:

- **calendar-types**: Foundation crate with date/time primitives (Year, Hour, Duration, DateTime), string types (Uid, Uri, LanguageTag), CSS3 color names, and IANA token types (LinkRelation, LocationType). Based on RFC 3339.

- **rfc5545-types**: iCalendar (RFC 5545) types. Contains recurrence rule (`RRule`) implementation with frequency-dependent BYxxx rules, efficient bitset types for time components (SecondSet, MinuteSet, HourSet, MonthSet, MonthDaySet, WeekNoSet), and property value types (status, trigger, period, etc.).

- **calico**: iCalendar parser, printer, and data model. Full support for RFC 5545, RFC 5546, and RFC 7986. Uses winnow for parsing and structible for component/parameter types. Re-exports rrule from rfc5545-types and css from calendar-types.

- **jscalendar**: JSCalendar (RFC 8984) implementation. Parser-agnostic design with traits (`DestructibleJsonValue`, `ConstructibleJsonValue`) abstracting over JSON libraries. Optional serde_json support via the `serde_json` feature flag.

## Architecture Notes

### Dependency Flow
```
calendar-types
     ↓
rfc5545-types
   ↓     ↓
calico  jscalendar
```

### Key Design Patterns

**Bitsets for recurrence rules**: Time component sets (SecondSet, MinuteSet, etc.) use NonZero integer types with a sentinel bit to guarantee non-zero representation. The highest bit is always set as a sentinel.

**Incremental parsing**: The parser in `jscalendar/src/parser.rs` uses `&mut &str` pattern for incremental parsing with explicit checkpointing for error recovery. Parsers must not modify input unless they succeed. The calico crate uses winnow for its iCalendar parser.

**Structible macro**: Both jscalendar and calico use the `structible` macro for builder-pattern construction with optional fields. JSCalendar uses it for object types (Event, Task, Group, Link); calico uses it for component types (Calendar, Event, Todo, etc.) and the Params type.

**Type markers**: DateTime uses marker types (Utc, Local, ()) to distinguish timezone context at compile time.

**Enum parsing**: IANA token enums use strum's `EnumString` with `ascii_case_insensitive` for case-insensitive parsing.

### Recurrence Rule Structure

RRule separates frequency-dependent rules (FreqByRules) from core rules (CoreByRules) that apply regardless of frequency. The `ByRuleName::behavior_with()` method implements the RFC 5545 table (page 44) defining which BYxxx rules expand vs limit occurrences for each frequency.
