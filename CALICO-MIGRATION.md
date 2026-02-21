# Plan: Migrate calico into the calendar-crates workspace

## Context

The [calico](https://github.com/eikopf/calico) crate (v0.2.0) is an RFC 5545 iCalendar parser and data model built on `winnow`. It needs to be brought into the calendar-crates workspace alongside `calendar-types`, `rfc5545-types`, and `jscalendar`. The goal is to eliminate type duplication, use the workspace's shared macro infrastructure (`structible`, `dizzy`), and establish a clean dependency graph.

### Target dependency graph

```
calendar-types
     |
rfc5545-types
   /       \
calico    jscalendar
```

Both `calico` and `jscalendar` depend on `rfc5545-types` (and transitively `calendar-types`) but **not** on each other.

---

## 1. Types replaced by existing workspace equivalents

These calico types have direct counterparts and should be **deleted from calico** in favor of imports:

| calico type | replace with | crate |
|---|---|---|
| `Date`, `DateTime<F>`, `Time<F>` | `Date`, `DateTime<M>`, `Time` | calendar-types |
| `Weekday` | `Weekday` | calendar-types |
| `IsoWeek` | `IsoWeek` | calendar-types |
| `Month` (1-12) | `Month` | calendar-types |
| `Sign` | `Sign` | calendar-types |
| `Css3Color` | `Css3Color` | calendar-types |
| `Uri` | `Uri`/`UriBuf` | calendar-types |
| `Uid` | `Uid`/`UidBuf` | calendar-types |
| `Duration`, `DurationKind`, `DurationTime` | `Duration`/`NominalDuration`/`ExactDuration`/`SignedDuration` | calendar-types |
| `RRule`, `Freq`, `FreqByRules`, `CoreByRules` | `RRule`, `Freq`, `FreqByRules`, `CoreByRules` | rfc5545-types |
| `SecondSet`, `MinuteSet`, `HourSet`, `MonthSet`, `MonthDaySet`, `WeekNoSet` | same names | rfc5545-types |
| `WeekdayNum`, `WeekdayNumSet`, `YearDayNum` | same names | rfc5545-types |
| `Interval`, `Termination` | same names | rfc5545-types |
| `DateTimeOrDate<F>` | `DateTimeOrDate<M>` | rfc5545-types |
| `UtcOffset` | `UtcOffset` | rfc5545-types |
| `Method` | `Method` | rfc5545-types |
| `Priority`, `PriorityClass` | `Priority`, `PriorityClass` | rfc5545-types |
| `RequestStatus`, `StatusCode` | `RequestStatus`, `StatusCode` | rfc5545-types |
| `CompletionPercentage` | `Percent` | rfc5545-types |
| `Integer`/`Float` type aliases | `primitive::Integer`/`Float` | rfc5545-types |

**Duration note:** calico decomposes duration as `DurationKind<T>` (Date/Time/Week) with `DurationTime<T>` (HMS/HM/MS/H/M/S). The workspace uses `NominalDuration` (weeks+days+optional exact) and `ExactDuration` (h/m/s/frac). These model the same RFC 5545 §3.3.6 grammar but with different decompositions. Calico's parsers will need adapter logic or the workspace types may need minor additions to support the exact same set of valid forms calico currently accepts.

## 2. Types to move into rfc5545-types

These are RFC 5545 concepts that calico defines but that should be shared types in `rfc5545-types`:

| calico type | RFC reference | notes |
|---|---|---|
| `Text`/`TextBuf` | §3.3.11 | string newtype (use dizzy) |
| `CaselessStr` | §3.1 | case-insensitive string newtype |
| `Name`/`NameKind` | §3.1 | content line name (IANA vs X-name) |
| `Encoding` (Bit8/Base64) | §3.3.2 | value encoding |
| `Geo` (lat/lon f64 pair) | §3.8.1.6 | geographic position |
| `Version` (V2_0) | §3.7.4 | iCalendar version |
| `Gregorian` | §3.7.1 | CALSCALE value |
| `TimeTransparency` | §3.8.2.7 | TRANSP (Opaque/Transparent) |
| `EventStatus`, `TodoStatus`, `JournalStatus` | §3.8.1.11 | component-specific statuses |
| `ClassValue<S>` | §3.8.1.3 | CLASS (Public/Private/Confidential/Other) |
| `FreeBusyType<S>` | §3.2.9 | FBTYPE parameter |
| `RelationshipType<S>` | §3.2.15 | RELTYPE parameter |
| `CalendarUserType<S>` | §3.2.3 | CUTYPE parameter |
| `ParticipationRole<S>` | §3.2.16 | ROLE parameter |
| `ParticipationStatus<S>` | §3.2.12 | PARTSTAT parameter |
| `AlarmAction<S>` + subtypes | §3.8.6.1 | ACTION property values |
| `TriggerValue`, `TriggerRelation` | §3.8.6.3 | TRIGGER property |
| `ValueType<S>` | §3.2.20 | VALUE parameter |
| `ThisAndFuture` | §3.2.13 | RANGE parameter |
| `Attachment` (Uri/Binary) | §3.8.1.1 | ATTACH value |
| `FormatType` (MIME) | §3.2.8 | FMTTYPE parameter |

**Design note on `<S>` generics:** calico uses a generic `S` parameter on extensible enums for the "Other" variant (e.g. `CalendarUserType<S>` has `Other(S)`). The workspace currently uses the `Token<T, S>` pattern from calendar-types for this (Known/Unknown discrimination). The rfc5545-types versions should follow whichever pattern the workspace already uses — likely `Token<KnownCalendarUserType, S>` or a similar approach with `#[non_exhaustive]` closed enums + `Token`.

## 3. Types to move into calendar-types

General-purpose types that aren't RFC-5545-specific:

| calico type | notes |
|---|---|
| `Language` (RFC 5646 tag) | jscalendar already has `LanguageTag` — unify into calendar-types so both can use it |

**LanguageTag extraction:** jscalendar currently defines `LanguageTag` in its own `model/string.rs` wrapping `language_tags::LanguageTag`. Calico needs language tags for the `LANGUAGE` parameter. This should be extracted to calendar-types so both crates can import it. The `language-tags` dependency moves from jscalendar to calendar-types.

## 4. Potential extraction from jscalendar

Beyond `LanguageTag`, no jscalendar-specific functionality needs extraction. The concepts that overlap (participant roles, statuses, etc.) are **semantically different** between RFC 5545 and RFC 8984:

| concept | RFC 5545 (calico) | RFC 8984 (jscalendar) |
|---|---|---|
| Participant roles | Chair, ReqParticipant, OptParticipant, NonParticipant | Owner, Attendee, Optional, Informational, Chair, Contact |
| Participation statuses | NeedsAction, Accepted, Declined, Tentative, Delegated, Completed, InProcess | NeedsAction, Accepted, Declined, Tentative, Delegated |
| Event statuses | Tentative, Confirmed, Cancelled | Confirmed, Cancelled, Tentative |
| Free/busy | Free, Busy, BusyUnavailable, BusyTentative | Free, Busy |

These are distinct enum types from distinct RFCs and should remain separate — RFC 5545 versions in rfc5545-types, RFC 8984 versions in jscalendar.

## 5. Macro replacements

### dizzy (`DstNewtype`) replaces `unsized_newtype!`

calico's `unsized_newtype!` macro generates transparent string newtypes with cloning/conversion. dizzy's `DstNewtype` derive does the same thing with validation hooks. Candidates:

- `CaselessStr` → `DstNewtype` with case-insensitive comparison overrides
- `TzId` → `DstNewtype`
- `Text`/`TextBuf` → `DstNewtype` with RFC 5545 text validation
- `Name` → `DstNewtype` with alphanumeric+hyphen validation
- `ParamValue` → `DstNewtype` (already have `ParamText`/`ParamTextBuf` in rfc5545-types)

### structible replaces `define_component_newtype!` and `define_parameter_type!`

calico's component types (Calendar, Event, Todo, Journal, FreeBusy, TimeZone, Alarm) are essentially structs with:
- Many optional typed property fields
- A catch-all map for unknown/extension properties
- Generated accessor methods

This maps directly to structible's design:
- Optional fields → `Option<T>` fields
- Catch-all → `#[structible(key = Box<str>)]` on a HashMap field
- Accessors → generated by structible

Similarly, `Params` (the parameter table) has ~23 optional RFC 5545 parameter fields + unknown parameter storage — also a good structible candidate.

**Caveat:** structible currently generates builder-style constructors for JSCalendar's JSON-centric objects. Calico's components are parsed from text content lines, not JSON. We need to verify that structible's generated API is flexible enough for calico's construction patterns (building up property-by-property during parsing), or whether calico needs its own construction approach.

## 6. What stays in calico

The following are iCalendar-text-format-specific and remain in calico:

- **Parser** (`winnow`-based): All of `parser/` — component, property, parameter, primitive, rrule, escaped, config, error modules
- **Component model**: `Calendar`, `Event`, `Todo`, `Journal`, `FreeBusy`, `TimeZone`, `TzRule`, `AudioAlarm`, `DisplayAlarm`, `EmailAlarm` (structible structs), `CalendarComponent` and `Alarm` dispatch enums, `OtherComponent`/`OtherAlarm` for unknown components
- **Property model**: `Prop<V,P>`, `StaticProp`, termination/trigger ref enums, `StructuredDataProp`
- **Parameter model**: `Params` struct (structible), `StructuredDataParams` (structible), `KnownParam`, `StaticParam`, `ParamName<S>`
- **Content line infrastructure**: Line folding, escape sequences, the `InputStream` trait
- **Validation/error types**: `MalformedCalendarError`, `MalformedEventError`, etc. (bitflag-based validation)
- **Value<S>**: Runtime-discriminated property values (for the text format's untyped value parsing)
- **Component dispatch**: `ComponentName<S>`, `AlarmKind`, `TzRuleKind`

## 7. Key decisions needing input

1. **Duration model reconciliation**: calico's `DurationKind<T>`/`DurationTime<T>` vs workspace's `NominalDuration`/`ExactDuration`. Should calico adapt to the workspace model, or does the workspace model need to be extended?

2. **Extensible enum pattern**: calico uses `EnumType<S>` with an `Other(S)` variant. The workspace uses `Token<T, S>` (Known/Unknown). Should calico's enums be migrated to `Token`, or should rfc5545-types adopt the `<S>` generic pattern for new types?

3. **winnow dependency**: calico uses winnow for parsing. The workspace's jscalendar parser uses `&mut &str` incremental parsing without winnow. Is it acceptable for calico to keep winnow, or should it be rewritten to match the workspace parser style?

4. **language-tags dependency**: Move to calendar-types (making it a dependency of the foundation crate) or to rfc5545-types?

5. **Naming**: Should the crate keep the name `calico` or be renamed to something like `icalendar` or `rfc5545`?

## 8. Remaining work

The structural migration is **complete**: all calico types have been replaced with workspace re-exports (see §10), macros have been replaced (dizzy, structible, strum), and the object model uses workspace types throughout. What remains is **new feature work**, not migration:

1. **Implement component/property parsers**: All parser entry points currently contain `todo!()` stubs. These need to be implemented to construct the new structible-based component types from parsed content lines.

2. **CI integration**: Add calico to workspace CI and ensure `cargo test --all-features` passes across the full workspace.

## 9. Migration steps (high-level)

1. ~~Add calico source to workspace, update workspace `Cargo.toml`~~ ✅
2. ~~Extract `LanguageTag` from jscalendar into calendar-types~~ ✅
3. ~~Add new RFC 5545 types to rfc5545-types (§2 above)~~ ✅
4. ~~Replace calico's duplicated types with imports from calendar-types and rfc5545-types~~ ✅
5. ~~Replace `unsized_newtype!` usages with dizzy `DstNewtype`~~ ✅
6. ~~Apply structible to Params and component types~~ ✅ (see §11)
7. ~~Replace hashify with strum EnumString~~ ✅ (see §12)
8. ~~Replace remaining calico-local types with workspace equivalents~~ ✅ (all types now re-exported, see §10)
9. Implement component/property parsers (new feature work, not migration)
10. Add calico to workspace CI, ensure `cargo test --all-features` passes

## 10. Completed type replacements

### Types replaced (calico now imports from workspace)

**From calendar-types:**
- `Sign`, `Token` (primitive)
- `Weekday`, `IsoWeek`, `Date`, `DateTime`, `Day`, `Hour`, `Local`, `Minute`, `Month`, `NonLeapSecond`, `Second`, `Time`, `TimeFormat`, `Utc`, `Year` (time)
- `Duration`, `ExactDuration`, `NominalDuration`, `SignedDuration` (duration)
- `LanguageTag` (as `Language`)
- `Css3Color` (entire css module re-exported)

**From rfc5545-types:**
- `Float`, `Integer`, `PositiveInteger` (primitive)
- `RequestStatus`, `StatusCode` (as `RequestStatusCode`), `Class` (request_status)
- `Gregorian`, `Version`, `Encoding`, `TimeTransparency`, `EventStatus`, `TodoStatus`, `JournalStatus`, `Priority`, `PriorityClass`, `TriggerRelation`, `ThisAndFuture`, `AlarmAction`, `CalendarUserType`, `ClassValue`, `FreeBusyType`, `Method`, `ParticipationRole`, `ParticipationStatus`, `RelationshipType`, `ValueType`, `Percent` (as `CompletionPercentage`) (set)
- `DateTimeOrDate`, `ExDateSeq`, `Period`, `RDate`, `RDateSeq`, `TriggerValue`, `UtcOffset` (time)
- `Geo`, `Attachment`, `FormatType`, `FormatTypeBuf` (value)
- `CaselessStr`, `Text`, `TextBuf`, `Name`, `NameKind`, `InvalidCharError`, `InvalidTextError`, `InvalidNameError` (string)
- Entire `rrule` module re-exported (RRule, Freq, FreqByRules, CoreByRules, bitset types)

### unsized_newtype! → DstNewtype migration

The `unsized_newtype!` macro and `CloneUnsized` helper trait have been completely removed. All remaining calico-local string DST newtypes now use `dizzy::DstNewtype`:

| type | invariant | notes |
|---|---|---|
| `TzId(str)` | `dizzy::trivial` | Any string is a valid TZID |
| `Uri(str)` | `dizzy::trivial` | Permissive — no scheme validation (unlike `calendar_types::string::Uri`) |
| `Uid(str)` | `dizzy::trivial` | Any string is a valid UID |
| `ParamValue(str)` | char validation | Rejects ASCII control (except HTAB) and double-quote |

### Workspace API additions for migration

- `rfc5545_types::string::TextBuf::from_string()` — construct from `String` with validation
- `rfc5545_types::string::TextBuf::from_string_unchecked()` — construct from pre-validated `String`
- `rfc5545_types::string::TextBuf::into_string()` — consume and return inner `String`
- `rfc5545_types::string::Text::char_is_valid()` — public character validation
- `rfc5545_types::time::DateTimeOrDate::map_marker()` — convert timezone marker type

### All type replacements complete

All types previously listed as "not yet replaced" have been migrated to workspace re-exports. The structural differences noted earlier (variant names, discriminant layouts, decomposition models, extensible enum patterns) were resolved during the migration. Key mappings:

- `Sign` → `calendar_types::primitive::Sign`
- `Month`, `Date`, `DateTime<F>`, `Time<F>` → `calendar_types::time::*`
- `Duration`/`DurationKind`/`DurationTime` → `calendar_types::duration::{Duration, ExactDuration, NominalDuration, SignedDuration}`
- `UtcOffset`, `DateTimeOrDate<F>` → `rfc5545_types::time::*`
- `CompletionPercentage` → `rfc5545_types::set::Percent` (re-exported as `CompletionPercentage`)
- `RequestStatus`/`RequestStatusCode` → `rfc5545_types::request_status::*`
- RRule and bitset types → `rfc5545_types::rrule::*` (entire module re-exported)
- Extensible `<S>` enums → `calendar_types::set::Token<ClosedEnum, S>` pattern

See `calico/src/model/primitive.rs` and `calico/src/model.rs` for the full re-export listing.

### LanguageTag extracted

`LanguageTag` moved from jscalendar to calendar-types. Both calico and jscalendar can now import from the same source. `language-tags` dependency moved from jscalendar to calendar-types.

### New types added to rfc5545-types

- **string.rs:** `Text`/`TextBuf`, `CaselessStr`, `Name`/`NameKind`, `InvalidCharError`, `InvalidTextError`, `InvalidNameError`
- **set.rs:** `Version`, `Gregorian`, `Encoding`, `TimeTransparency`, `EventStatus`, `TodoStatus`, `JournalStatus`, `ClassValue`, `CalendarUserType`, `ParticipationRole`, `ParticipationStatus`, `FreeBusyType`, `RelationshipType`, `AlarmAction`, `TriggerRelation`, `ValueType`, `ThisAndFuture`
- **value.rs (new):** `Geo`, `Attachment`, `FormatType`/`FormatTypeBuf`

## 11. structible migration

### Params → structible

Replaced `define_parameter_type!`/`define_methods!` macros, `ParameterTable`, `AnyParamValue`/`AnyParamValueInner`, and `impl_any_param_value_conversions!` with two `#[structible]` structs:

- **`Params`**: 25 optional named fields + catch-all `#[structible(key = Box<CaselessStr>)] unknown_param`
- **`StructuredDataParams`**: 2 required fields (`format_type`, `schema`) + same optional fields as `Params`
- Added `Params::insert_known(KnownParam)` dispatch method
- Added `Params::contains_known(StaticParam) -> bool` dispatch method
- `TryFrom<Params> for StructuredDataParams` uses `into_fields()` + `take_*` pattern

Deleted: `ParameterTable`, `AnyParamValue`/`AnyParamValueInner`, `NeverStr`, all associated macros.

### Component types → structible

Replaced `define_component_newtype!` macro and generic `Component` property bag with 14 individual `#[structible]` structs:

| Struct | Required constructor fields |
|--------|---------------------------|
| `Calendar` | `prod_id`, `version` |
| `Event` | `dtstamp`, `uid` |
| `Todo` | `dtstamp`, `uid` |
| `Journal` | `dtstamp`, `uid` |
| `FreeBusy` | `dtstamp`, `uid` |
| `TimeZone` | `tz_id` |
| `TzRule` | `kind`, `dtstart`, `tz_offset_to`, `tz_offset_from` |
| `AudioAlarm` | `trigger` |
| `DisplayAlarm` | `trigger`, `description` |
| `EmailAlarm` | `trigger`, `description`, `summary` |
| `LocationComponent` | `uid` |
| `ResourceComponent` | `uid` |
| `Participant` | `uid`, `participant_type` |

Each struct has typed property fields (`Prop<V, Params>`), multi-valued fields as `Vec`, subcomponents as `Vec`, and a catch-all `x_property` field for unknown/extension properties.

Added dispatch enums: `CalendarComponent` (Event/Todo/Journal/FreeBusy/TimeZone/Other) and `Alarm` (Audio/Display/Email/Other).

Deleted: `Component` struct, `ComponentTag` enum, `PropKey<S>`, `PropertySeq`, all `Equivalent` impls.

## 12. hashify → strum EnumString

Removed `hashify` dependency. All IANA token enums already derive `strum::EnumString` with `#[strum(ascii_case_insensitive)]` in rfc5545-types and calendar-types, so the `hashify::map_ignore_case!` perfect-hash maps were redundant.

- Simplified `match_iana_token!` macro to use `str::parse::<T>()` (14 call sites)
- Replaced direct `hashify::map_ignore_case!` in `status` and `color` parsers with `str::parse()`
- Net reduction of ~280 lines of duplicated variant mappings

## Verification

- ✅ `cargo check` and `cargo test --all-features` across the entire workspace
- ✅ Calico's existing test suite passes with workspace types substituted
- ✅ No circular dependencies between crates
- ✅ jscalendar's existing tests still pass after LanguageTag extraction
