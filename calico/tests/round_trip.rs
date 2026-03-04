//! Round-trip serialization tests: parse → serialize → parse again.

use calico::model::component::Calendar;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn walkdir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path));
            } else {
                result.push(path);
            }
        }
    }
    result
}

fn collect_ics_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    for entry in walkdir(dir) {
        if entry.extension().is_some_and(|e| e == "ics") {
            files.push(entry);
        }
    }
    files.sort();
    files
}

/// Parses, serializes, then re-parses a single .ics file.
/// Returns Ok if re-parse succeeds with the same number of calendars.
fn round_trip_file(path: &std::path::Path) -> Result<(), String> {
    let input = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let calendars = Calendar::parse(&input).map_err(|e| format!("parse1: {e}"))?;
    if calendars.is_empty() {
        return Err("parse1 returned 0 calendars".into());
    }

    // Serialize each calendar
    let mut serialized = String::new();
    for cal in &calendars {
        cal.write_ical_to(&mut serialized).map_err(|e| format!("serialize: {e}"))?;
    }

    // Re-parse the serialized output
    let calendars2 = Calendar::parse(&serialized).map_err(|e| format!("parse2: {e}"))?;

    if calendars.len() != calendars2.len() {
        return Err(format!(
            "calendar count mismatch: {} vs {}",
            calendars.len(),
            calendars2.len()
        ));
    }

    Ok(())
}

/// Runs round-trip on all parseable corpus files.
#[test]
#[ignore]
fn round_trip_corpus() {
    let fixtures = fixtures_dir();
    let files = collect_ics_files(&fixtures);

    if files.is_empty() {
        eprintln!("No .ics fixtures found. Run `just fetch-fixtures` to download them.");
        return;
    }

    let mut total = 0;
    let mut parse_ok = 0;
    let mut round_trip_ok = 0;
    let mut round_trip_failures: Vec<(String, String)> = Vec::new();

    for file in &files {
        let rel = file.strip_prefix(&fixtures).unwrap_or(file);
        let name = rel.display().to_string();

        let input = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Only test files that parse successfully in the first place
        match Calendar::parse(&input) {
            Ok(c) if !c.is_empty() => c,
            _ => continue,
        };

        total += 1;
        parse_ok += 1;

        match round_trip_file(file) {
            Ok(()) => round_trip_ok += 1,
            Err(e) => round_trip_failures.push((name, e)),
        }
    }

    eprintln!("\n=== Round-Trip Test Results ===");
    eprintln!("Parseable files: {parse_ok}  Round-trip OK: {round_trip_ok}  Failed: {}", round_trip_failures.len());

    if !round_trip_failures.is_empty() {
        eprintln!("\n--- Round-Trip Failures ---");
        for (name, err) in &round_trip_failures {
            eprintln!("  {name}: {err}");
        }
    }

    eprintln!("\nRound-trip rate: {:.1}%", if total > 0 { 100.0 * (round_trip_ok as f64 / total as f64) } else { 0.0 });
}

/// A known-good small calendar that must round-trip.
#[test]
fn round_trip_simple_event() {
    let input = "BEGIN:VCALENDAR\r\n\
                  VERSION:2.0\r\n\
                  PRODID:-//Test//Test//EN\r\n\
                  BEGIN:VEVENT\r\n\
                  UID:test-1@example.com\r\n\
                  DTSTAMP:20070423T123432Z\r\n\
                  DTSTART:20070628T090000Z\r\n\
                  DTEND:20070628T100000Z\r\n\
                  SUMMARY:Test Event\r\n\
                  END:VEVENT\r\n\
                  END:VCALENDAR\r\n";

    let cals = Calendar::parse(input).expect("parse1");
    assert_eq!(cals.len(), 1);

    let serialized = cals[0].to_ical();
    let cals2 = Calendar::parse(&serialized).expect("parse2");
    assert_eq!(cals2.len(), 1);

    // Verify key property values survived the round-trip
    let event = match &cals2[0].components()[0] {
        calico::model::component::CalendarComponent::Event(e) => e,
        other => panic!("expected Event, got {:?}", std::mem::discriminant(other)),
    };
    assert_eq!(event.summary().unwrap().value.as_str(), "Test Event");
    assert_eq!(event.uid().unwrap().value.as_str(), "test-1@example.com");
}

/// Round-trip a calendar with a VTIMEZONE.
#[test]
fn round_trip_timezone() {
    let input = "BEGIN:VCALENDAR\r\n\
                  VERSION:2.0\r\n\
                  PRODID:-//Test//Test//EN\r\n\
                  BEGIN:VTIMEZONE\r\n\
                  TZID:America/New_York\r\n\
                  BEGIN:STANDARD\r\n\
                  DTSTART:19701101T020000\r\n\
                  RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n\
                  TZOFFSETFROM:-0400\r\n\
                  TZOFFSETTO:-0500\r\n\
                  TZNAME:EST\r\n\
                  END:STANDARD\r\n\
                  BEGIN:DAYLIGHT\r\n\
                  DTSTART:19700308T020000\r\n\
                  RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n\
                  TZOFFSETFROM:-0500\r\n\
                  TZOFFSETTO:-0400\r\n\
                  TZNAME:EDT\r\n\
                  END:DAYLIGHT\r\n\
                  END:VTIMEZONE\r\n\
                  END:VCALENDAR\r\n";

    let cals = Calendar::parse(input).expect("parse1");
    assert_eq!(cals.len(), 1);

    let serialized = cals[0].to_ical();
    let cals2 = Calendar::parse(&serialized).expect("parse2");
    assert_eq!(cals2.len(), 1);

    // Verify timezone survived
    match &cals2[0].components()[0] {
        calico::model::component::CalendarComponent::TimeZone(tz) => {
            assert_eq!(tz.tz_id().value.as_str(), "America/New_York");
            assert_eq!(tz.rules().len(), 2);
        }
        other => panic!("expected TimeZone, got {:?}", std::mem::discriminant(other)),
    }
}

/// Round-trip a VTODO component.
#[test]
fn round_trip_todo() {
    let input = "BEGIN:VCALENDAR\r\n\
                  VERSION:2.0\r\n\
                  PRODID:-//Test//Test//EN\r\n\
                  BEGIN:VTODO\r\n\
                  UID:todo-1@example.com\r\n\
                  DTSTAMP:20070423T123432Z\r\n\
                  SUMMARY:Buy groceries\r\n\
                  STATUS:NEEDS-ACTION\r\n\
                  END:VTODO\r\n\
                  END:VCALENDAR\r\n";

    let cals = Calendar::parse(input).expect("parse1");
    let serialized = cals[0].to_ical();
    let cals2 = Calendar::parse(&serialized).expect("parse2");
    assert_eq!(cals2.len(), 1);

    match &cals2[0].components()[0] {
        calico::model::component::CalendarComponent::Todo(t) => {
            assert_eq!(t.summary().unwrap().value.as_str(), "Buy groceries");
        }
        other => panic!("expected Todo, got {:?}", std::mem::discriminant(other)),
    }
}

/// Round-trip a DATE-only DTSTART (VALUE=DATE).
#[test]
fn round_trip_date_value() {
    let input = "BEGIN:VCALENDAR\r\n\
                  VERSION:2.0\r\n\
                  PRODID:-//Test//Test//EN\r\n\
                  BEGIN:VEVENT\r\n\
                  UID:date-1@example.com\r\n\
                  DTSTAMP:20070423T123432Z\r\n\
                  DTSTART;VALUE=DATE:20071102\r\n\
                  SUMMARY:All Day Event\r\n\
                  END:VEVENT\r\n\
                  END:VCALENDAR\r\n";

    let cals = Calendar::parse(input).expect("parse1");
    let serialized = cals[0].to_ical();

    // Verify the serialized output includes VALUE=DATE
    assert!(
        serialized.contains("VALUE=DATE"),
        "Serialized output should contain VALUE=DATE: {serialized}"
    );

    let cals2 = Calendar::parse(&serialized).expect("parse2");
    assert_eq!(cals2.len(), 1);
}

/// Round-trip a VALARM with DISPLAY action.
#[test]
fn round_trip_alarm() {
    let input = "BEGIN:VCALENDAR\r\n\
                  VERSION:2.0\r\n\
                  PRODID:-//Test//Test//EN\r\n\
                  BEGIN:VEVENT\r\n\
                  UID:alarm-1@example.com\r\n\
                  DTSTAMP:20070423T123432Z\r\n\
                  DTSTART:20070628T090000Z\r\n\
                  SUMMARY:Meeting\r\n\
                  BEGIN:VALARM\r\n\
                  ACTION:DISPLAY\r\n\
                  TRIGGER:-PT15M\r\n\
                  DESCRIPTION:Event reminder\r\n\
                  END:VALARM\r\n\
                  END:VEVENT\r\n\
                  END:VCALENDAR\r\n";

    let cals = Calendar::parse(input).expect("parse1");
    let serialized = cals[0].to_ical();
    let cals2 = Calendar::parse(&serialized).expect("parse2");
    assert_eq!(cals2.len(), 1);

    // Verify alarm was preserved
    match &cals2[0].components()[0] {
        calico::model::component::CalendarComponent::Event(e) => {
            assert_eq!(e.alarms().len(), 1);
        }
        other => panic!("expected Event, got {:?}", std::mem::discriminant(other)),
    }
}

/// Round-trip a corpus file that is known to parse correctly.
#[test]
fn round_trip_rfc5545_sec3_6_1() {
    let path = fixtures_dir().join("ical4j/rfc5545-sec3.6.1.ics");
    if !path.exists() {
        eprintln!("Skipping: fixture not found. Run `just fetch-fixtures`.");
        return;
    }
    round_trip_file(&path).unwrap();
}
