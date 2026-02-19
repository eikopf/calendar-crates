//! Tests against the examples given in RFC 8984 §6.

use jscalendar::{
    json::TryFromJson,
    model::{
        object::{Event, Group, Task},
        rrule::{FreqByRules, YearlyByRules},
        time::{
            Date, DateTime, Day, Duration, ExactDuration, Hour, Local, Minute, Month,
            NominalDuration, Second, Time, Utc, Year,
        },
    },
};

#[cfg(feature = "serde_json")]
#[test]
fn simple_event() {
    use serde_json::json;

    let input = json!({
      "@type": "Event",
      "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
      "updated": "2020-01-02T18:23:04Z",
      "title": "Some event",
      "start": "2020-01-15T13:00:00",
      "timeZone": "America/New_York",
      "duration": "PT1H",
    });

    let event = Event::try_from_json(input).unwrap();
    assert_eq!(event.uid().as_str(), "a8df6573-0474-496d-8496-033ad45d7fea");
    assert_eq!(
        event.updated(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D02).unwrap(),
            time: Time::new(Hour::H18, Minute::M23, Second::S04, None).unwrap(),
            marker: Utc
        })
    );
    assert_eq!(event.title(), Some(&String::from("Some event")));
    assert_eq!(
        event.start(),
        &DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D15).unwrap(),
            time: Time::new(Hour::H13, Minute::M00, Second::S00, None).unwrap(),
            marker: Local
        }
    );
    assert_eq!(event.time_zone(), Some(&String::from("America/New_York")));
    assert_eq!(
        event.duration(),
        Some(&Duration::Exact(ExactDuration {
            hours: 1,
            ..Default::default()
        }))
    );
}

#[cfg(feature = "serde_json")]
#[test]
fn simple_task() {
    use serde_json::json;

    let input = json!({
      "@type": "Task",
      "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
      "updated": "2020-01-09T14:32:01Z",
      "title": "Do something",
    });

    let task = Task::try_from_json(input).unwrap();
    assert_eq!(task.uid().as_str(), "2a358cee-6489-4f14-a57f-c104db4dc2f2");
    assert_eq!(
        task.updated(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D09).unwrap(),
            time: Time::new(Hour::H14, Minute::M32, Second::S01, None).unwrap(),
            marker: Utc
        })
    );
    assert_eq!(task.title(), Some(&String::from("Do something")));
}

#[cfg(feature = "serde_json")]
#[test]
fn simple_group() {
    use serde_json::json;

    let input = json!({
      "@type": "Group",
      "uid": "bf0ac22b-4989-4caf-9ebd-54301b4ee51a",
      "updated": "2020-01-15T18:00:00Z",
      "title": "A simple group", // Erratum 8028
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
    });

    let group = Group::try_from_json(input).unwrap();
    assert_eq!(group.uid().as_str(), "bf0ac22b-4989-4caf-9ebd-54301b4ee51a");
    assert_eq!(
        group.updated(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D15).unwrap(),
            time: Time::new(Hour::H18, Minute::M00, Second::S00, None).unwrap(),
            marker: Utc
        })
    );
    assert_eq!(group.title(), Some(&String::from("A simple group"))); // Erratum 8028

    let event = group.entries()[0].as_event().unwrap();
    assert_eq!(event.uid().as_str(), "a8df6573-0474-496d-8496-033ad45d7fea");
    assert_eq!(
        event.updated(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D02).unwrap(),
            time: Time::new(Hour::H18, Minute::M23, Second::S04, None).unwrap(),
            marker: Utc
        })
    );
    assert_eq!(event.title(), Some(&String::from("Some event")));
    assert_eq!(
        event.start(),
        &DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D15).unwrap(),
            time: Time::new(Hour::H13, Minute::M00, Second::S00, None).unwrap(),
            marker: Local
        }
    );
    assert_eq!(event.time_zone(), Some(&String::from("America/New_York")));
    assert_eq!(
        event.duration(),
        Some(&Duration::Exact(ExactDuration {
            hours: 1,
            ..Default::default()
        }))
    );

    let task = group.entries()[1].as_task().unwrap();
    assert_eq!(task.uid().as_str(), "2a358cee-6489-4f14-a57f-c104db4dc2f2");
    assert_eq!(
        task.updated(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D09).unwrap(),
            time: Time::new(Hour::H14, Minute::M32, Second::S01, None).unwrap(),
            marker: Utc
        })
    );
    assert_eq!(task.title(), Some(&String::from("Do something")));
}

#[cfg(feature = "serde_json")]
#[test]
fn all_day_event() {
    use serde_json::json;

    let input = json!({
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
    });

    let event = Event::try_from_json(input).unwrap();
    assert_eq!(event.uid().as_str(), "a8df6573-0474-496d-8496-033ad45d7fea");
    assert_eq!(event.title(), Some(&String::from("April Fool's Day")));
    assert_eq!(event.show_without_time(), Some(&true));
    assert_eq!(
        event.start(),
        &DateTime {
            date: Date::new(Year::new(1900).unwrap(), Month::Apr, Day::D01).unwrap(),
            time: Time::new(Hour::H00, Minute::M00, Second::S00, None).unwrap(),
            marker: Local
        }
    );
    assert_eq!(
        event.duration(),
        Some(&Duration::Nominal(NominalDuration {
            weeks: 0,
            days: 1,
            exact: None
        }))
    );

    let rrule = &event.recurrence_rules().unwrap()[0];
    assert_eq!(rrule.freq, FreqByRules::Yearly(YearlyByRules::default()));
}

#[cfg(feature = "serde_json")]
#[test]
fn task_with_a_due_date() {
    use serde_json::json;

    let input = json!({
        "@type": "Task",
        "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
        "title": "Buy groceries",
        "due": "2020-01-19T18:00:00",
        "timeZone": "Europe/Vienna",
        "estimatedDuration": "PT1H"
    });

    let task = Task::try_from_json(input).unwrap();
    assert_eq!(task.uid().as_str(), "2a358cee-6489-4f14-a57f-c104db4dc2f2");
    assert_eq!(task.title(), Some(&String::from("Buy groceries")));
    assert_eq!(
        task.due(),
        Some(&DateTime {
            date: Date::new(Year::new(2020).unwrap(), Month::Jan, Day::D19).unwrap(),
            time: Time::new(Hour::H18, Minute::M00, Second::S00, None).unwrap(),
            marker: Local
        })
    );
    assert_eq!(task.time_zone(), Some(&String::from("Europe/Vienna")));
    assert_eq!(
        task.estimated_duration(),
        Some(&Duration::Exact(ExactDuration {
            hours: 1,
            ..Default::default()
        }))
    );
}

#[cfg(feature = "serde_json")]
#[test]
fn recurring_event_with_participants() {
    use serde_json::json;

    let input = json!({
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
    });

    let event = Event::try_from_json(input).unwrap();
    dbg![event];

    // TODO: test individual fields
}
