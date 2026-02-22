# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

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

- **calendar-types**: Foundation crate with date/time primitives (Year, Hour, Duration), string types (Uid, Uri), and common primitives (Sign). Based on RFC 3339.

- **rfc5545-types**: iCalendar (RFC 5545) types. Contains recurrence rule (`RRule`) implementation with frequency-dependent BYxxx rules, and efficient bitset types for time components (SecondSet, MinuteSet, HourSet, MonthSet, MonthDaySet, WeekNoSet).

- **calico**: iCalendar (RFC 5545) parser, printer, and data model. Uses winnow for parsing and the `structible` macro for builder-pattern construction.

- **jscalendar**: JSCalendar (RFC 8984) implementation. Parser-agnostic design with optional serde support via feature flags.

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

**Incremental parsing**: The parser in `jscalendar/src/parser.rs` uses `&mut &str` pattern for incremental parsing with explicit checkpointing for error recovery. Parsers must not modify input unless they succeed.

**Structible macro**: JSCalendar object types (Event, Task, Group, Link) and calico component/parameter types use the `structible` macro for builder-pattern construction with optional fields.

**Type markers**: DateTime uses marker types (Utc, Local, ()) to distinguish timezone context at compile time.

### Recurrence Rule Structure

RRule separates frequency-dependent rules (FreqByRules) from core rules (CoreByRules) that apply regardless of frequency. The `ByRuleName::behavior_with()` method implements the RFC 5545 table (page 44) defining which BYxxx rules expand vs limit occurrences for each frequency.
