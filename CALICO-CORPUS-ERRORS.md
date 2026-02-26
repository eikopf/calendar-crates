# Calico corpus test failures

Results from `just ics-corpus-test`.

**262 total, 222 passed (84.7%), 40 failed.**

## A. Non-UTC datetime for UTC-only properties (11 files)

The parser requires a `Z` suffix for DTSTAMP, COMPLETED, CREATED, and LAST-MODIFIED.
Real-world files frequently emit local datetimes or TZID-qualified datetimes for these.

| File | Property |
|------|----------|
| `allenporter-ical/datetime_local.ics` | `DTSTAMP:19980115T110500` |
| `allenporter-ical/datetime_vtimezone.ics` | `DTSTAMP:20230312T181210` |
| `allenporter-ical/datetime_vtimezone_plus.ics` | `DTSTAMP:20230312T181210` |
| `allenporter-ical/rrule-exdate-mismatch.ics` | `DTSTAMP:20231210T163427` |
| `ical.net/Calendar1.ics` | `COMPLETED:20060728T141612` |
| `ical.net/Categories1.ics` | `EXDATE:20250914` (see also B) |
| `ical.net/Parameter2.ics` | `LAST-MODIFIED:20090920T000000` |
| `ical.net/Todo5.ics` | `COMPLETED;TZID=US-Eastern:20060730T090000` |
| `ical.net/Todo6.ics` | same |
| `ical.net/Todo7.ics` | same |
| `ical.net/Todo8.ics` | same |
| `ical.net/Todo9.ics` | same |

## B. EXDATE date-only without `VALUE=DATE` (2 files)

EXDATE defaults to DateTime parsing when no `VALUE` param is present. Date-only values
fail the datetime parser.

| File | Property |
|------|----------|
| `ical.net/Categories1.ics` | `EXDATE:20250914` |
| `allenporter-ical/rrule-exdate.ics` | `EXDATE:20220901,20221001` |

## C. `VALUE=DATE` parameter with datetime value (3 files)

`DTSTART;VALUE=DATE:20090117T200000` tells the parser to parse as date-only, but the
value includes a time component. The parser consumes `20090117` and chokes on `T200000`.

| File |
|------|
| `ical.net/CaseInsensitive4.ics` |
| `ical.net/EmptyLines4.ics` |
| `ical.net/Property1.ics` |

## D. Missing PRODID property (5 files)

PRODID is required by RFC 5545 and the parser now enforces this.

| File |
|------|
| `ical.net/CaseInsensitive1.ics` |
| `ical.net/CaseInsensitive2.ics` |
| `ical.net/CaseInsensitive3.ics` |
| `ical.net/XProperty1.ics` |
| `ical4j/bitfire1.ics` |

## E. Missing VERSION property (2 files)

| File | Detail |
|------|--------|
| `ical.net/CalendarParameters2.ics` | No VERSION |
| `ical.net/Bug2959692.ics` | `PRODID:... VERSION:2.0` on same line |

## F. `ENCODING=BASE64` on known typed properties (2 files)

The parser dispatches known property names to type-specific parsers (e.g. SEQUENCE to
integer). When `ENCODING=BASE64` is present, the value is base64-encoded but the parser
tries to parse it as the native type.

| File | Property |
|------|----------|
| `ical.net/Encoding1.ics` | `SEQUENCE;ENCODING=BASE64:MQ==` |
| `ical.net/Encoding3.ics` | same |

## G. `ENCODING=QUOTED-PRINTABLE` not supported (1 file)

| File | Property |
|------|----------|
| `ical.net/Parse1.ics` | `DESCRIPTION;ENCODING=QUOTED-PRINTABLE:...` |

## H. VALARM missing mandatory ACTION property (1 file)

| File | Detail |
|------|--------|
| `ical.net/Event3.ics` | Last VALARM has only TRIGGER, no ACTION |

## I. DisplayAlarm missing mandatory DESCRIPTION property (1 file)

| File | Detail |
|------|--------|
| `ical4j/evolution2.ics` | `BEGIN:VALARM` / `ACTION:DISPLAY` / `TRIGGER:-P1D` / `END:VALARM` (no DESCRIPTION) |

## J. Duplicate parameter names (1 file)

| File | Detail |
|------|--------|
| `ical.net/Parameter1.ics` | `DTSTART;VALUE=DATE;VALUE=OTHER:...` |

## K. Custom/non-standard VALUE type in parameter (1 file)

| File | Detail |
|------|--------|
| `allenporter-ical/unknown_value_custom.ics` | `SUMMARY;VALUE=X-CUSTOM-TYPE:...` |

## L. Duplicate MEMBER parameter (1 file)

| File | Detail |
|------|--------|
| `ical.net/Attendee2.ics` | `ATTENDEE;MEMBER="...","...";MEMBER="..."` |

## M. Invalid datetime format (1 file)

| File | Detail |
|------|--------|
| `ical.net/PARSE17.ics` | `DTSTART:1234` (4-digit value) |

## N. Non-standard X-property name (1 file)

| File | Detail |
|------|--------|
| `ical.net/Bug2033495.ics` | `X-LOTUS-CHILD_UID:XXX` (underscore in name) |

## O. Non-UTF-8 encoding (4 files)

Fail at `read_to_string()` before the parser is invoked.

| File |
|------|
| `ical.net/Bug2148092.ics` |
| `ical.net/Bug2912657.ics` |
| `ical.net/ProdID2.ics` |
| `ical4j/1106817412.ics` |

## P. TRIGGER with absolute datetime but no `VALUE=DATE-TIME` (1 file)

| File | Detail |
|------|--------|
| `allenporter-ical/todo_valarm.ics` | `TRIGGER:19980403T120000Z` |

## Q. Semicolon in property value treated as parameter separator (1 file)

| File | Detail |
|------|--------|
| `ical4j/stacksize.ics` | `X-ALT-DESC` value contains `&lt;a` where `;` is parsed as a parameter separator |

## R. Property name/value split across fold boundary (1 file)

| File | Detail |
|------|--------|
| `ical.net/Language2.ics` | Properties like `UID\n :value` with fold between name and colon |
