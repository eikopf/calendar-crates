//! Round-trip serialization tests: JSON → Rust → JSON → Rust → assert_eq!

#![cfg(feature = "serde_json")]

use jscalendar::json::{IntoJson, TryFromJson};
use jscalendar::model::object::{Event, Group, Task};
use serde_json::{json, Value};

/// Parse → clone → serialize → re-parse → compare the two Rust structs.
fn assert_event_round_trips(input: Value) {
    let parsed: Event<Value> = Event::try_from_json(input).expect("initial parse failed");
    let cloned = parsed.clone();
    let json_out: Value = cloned.into_json();
    let reparsed: Event<Value> = Event::try_from_json(json_out).expect("re-parse failed");
    assert_eq!(parsed, reparsed);
}

fn assert_task_round_trips(input: Value) {
    let parsed: Task<Value> = Task::try_from_json(input).expect("initial parse failed");
    let cloned = parsed.clone();
    let json_out: Value = cloned.into_json();
    let reparsed: Task<Value> = Task::try_from_json(json_out).expect("re-parse failed");
    assert_eq!(parsed, reparsed);
}

fn assert_group_round_trips(input: Value) {
    let parsed: Group<Value> = Group::try_from_json(input).expect("initial parse failed");
    let cloned = parsed.clone();
    let json_out: Value = cloned.into_json();
    let reparsed: Group<Value> = Group::try_from_json(json_out).expect("re-parse failed");
    assert_eq!(parsed, reparsed);
}

#[test]
fn round_trip_simple_event() {
    assert_event_round_trips(json!({
        "@type": "Event",
        "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
        "updated": "2020-01-02T18:23:04Z",
        "title": "Some event",
        "start": "2020-01-15T13:00:00",
        "timeZone": "America/New_York",
        "duration": "PT1H"
    }));
}

#[test]
fn round_trip_event_with_optional_fields() {
    assert_event_round_trips(json!({
        "@type": "Event",
        "uid": "b1234567-0000-0000-0000-000000000001",
        "start": "2024-06-15T10:00:00",
        "timeZone": "Europe/Berlin",
        "duration": "PT2H30M",
        "title": "Conference talk",
        "description": "A detailed description of the talk",
        "updated": "2024-05-01T12:00:00Z",
        "color": "steelblue",
        "keywords": {
            "conference": true,
            "tech": true
        },
        "categories": {
            "work": true
        },
        "links": {
            "link-1": {
                "@type": "Link",
                "href": "https://example.com/slides.pdf",
                "title": "Slides",
                "rel": "enclosure"
            }
        },
        "locations": {
            "loc-1": {
                "@type": "Location",
                "name": "Main Hall",
                "description": "Building A, Floor 2"
            }
        }
    }));
}

#[test]
fn round_trip_recurring_event_with_participants() {
    assert_event_round_trips(json!({
        "@type": "Event",
        "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
        "title": "FooBar team meeting",
        "start": "2020-01-08T09:00:00",
        "timeZone": "Africa/Johannesburg",
        "duration": "PT1H",
        "virtualLocations": {
            "0": {
                "@type": "VirtualLocation",
                "name": "ChatMe meeting room",
                "uri": "https://chatme.example.com?id=1234567&pw=a8a24627b63d"
            }
        },
        "recurrenceRules": [{
            "@type": "RecurrenceRule",
            "frequency": "weekly"
        }],
        "replyTo": {
            "imip": "mailto:f245f875-7f63-4a5e-a2c8@schedule.example.com"
        },
        "participants": {
            "dG9tQGZvb2Jhci5xlLmNvbQ": {
                "@type": "Participant",
                "name": "Tom Tool",
                "email": "tom@foobar.example.com",
                "sendTo": {
                    "imip": "mailto:tom@calendar.example.com"
                },
                "participationStatus": "accepted",
                "roles": {
                    "attendee": true
                }
            },
            "em9lQGZvb2GFtcGxlLmNvbQ": {
                "@type": "Participant",
                "name": "Zoe Zelda",
                "email": "zoe@foobar.example.com",
                "sendTo": {
                    "imip": "mailto:zoe@foobar.example.com"
                },
                "participationStatus": "accepted",
                "roles": {
                    "owner": true,
                    "attendee": true,
                    "chair": true
                }
            }
        },
        "recurrenceOverrides": {
            "2020-03-04T09:00:00": {
                "participants/dG9tQGZvb2Jhci5xlLmNvbQ/participationStatus": "declined"
            }
        }
    }));
}

#[test]
fn round_trip_all_day_event() {
    assert_event_round_trips(json!({
        "@type": "Event",
        "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
        "title": "April Fool's Day",
        "showWithoutTime": true,
        "start": "1900-04-01T00:00:00",
        "duration": "P1D",
        "recurrenceRules": [{
            "@type": "RecurrenceRule",
            "frequency": "yearly"
        }]
    }));
}

#[test]
fn round_trip_simple_task() {
    assert_task_round_trips(json!({
        "@type": "Task",
        "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
        "updated": "2020-01-09T14:32:01Z",
        "title": "Do something"
    }));
}

#[test]
fn round_trip_task_with_due_date() {
    assert_task_round_trips(json!({
        "@type": "Task",
        "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
        "title": "Buy groceries",
        "due": "2020-01-19T18:00:00",
        "timeZone": "Europe/Vienna",
        "estimatedDuration": "PT1H"
    }));
}

#[test]
fn round_trip_simple_group() {
    assert_group_round_trips(json!({
        "@type": "Group",
        "uid": "bf0ac22b-4989-4caf-9ebd-54301b4ee51a",
        "updated": "2020-01-15T18:00:00Z",
        "title": "A simple group",
        "entries": [{
            "@type": "Event",
            "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
            "updated": "2020-01-02T18:23:04Z",
            "title": "Some event",
            "start": "2020-01-15T13:00:00",
            "timeZone": "America/New_York",
            "duration": "PT1H"
        },
        {
            "@type": "Task",
            "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
            "updated": "2020-01-09T14:32:01Z",
            "title": "Do something"
        }]
    }));
}

#[test]
fn round_trip_vendor_properties() {
    assert_event_round_trips(json!({
        "@type": "Event",
        "uid": "vendor-prop-test-uid",
        "start": "2024-01-01T00:00:00",
        "title": "Vendor property test",
        "example.com:custom": "hello world",
        "example.com:nested": { "key": "value" }
    }));
}
