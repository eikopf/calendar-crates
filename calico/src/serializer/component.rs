//! `WriteIcal` implementations for iCalendar components.

use std::fmt;

use super::{
    WriteIcal, FoldingWriter, write_crlf,
    property::*,
};
use crate::model::{
    component::*,
    parameter::Params,
};

// ============================================================================
// Calendar
// ============================================================================

impl WriteIcal for Calendar {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VCALENDAR\r\n")?;
        write_prop("VERSION", self.version(), w)?;
        write_prop("PRODID", self.prod_id(), w)?;
        write_opt_prop("CALSCALE", self.cal_scale(), w)?;
        write_opt_prop("METHOD", self.method(), w)?;

        // RFC 7986
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("URL", self.url(), w)?;
        write_opt_prop("REFRESH-INTERVAL", self.refresh_interval(), w)?;
        write_opt_prop("SOURCE", self.source(), w)?;
        write_opt_prop("COLOR", self.color(), w)?;
        write_vec_prop("NAME", self.name(), w)?;
        write_vec_prop("DESCRIPTION", self.description(), w)?;
        write_vec_prop("CATEGORIES", self.categories(), w)?;
        write_attach_vec("IMAGE", self.image(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for comp in self.components() {
            comp.write_ical(w)?;
        }

        w.write_str("END:VCALENDAR\r\n")
    }
}

impl Calendar {
    /// Serializes this calendar to an iCalendar string with RFC 5545 line folding.
    pub fn to_ical(&self) -> String {
        let mut fw = FoldingWriter::new(String::new());
        self.write_ical(&mut fw).expect("writing to String cannot fail");
        fw.into_inner()
    }

    /// Writes this calendar in iCalendar format to the given writer with line folding.
    pub fn write_ical_to(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        let mut fw = FoldingWriter::new(WriteAdapter(w));
        self.write_ical(&mut fw)
    }
}

/// Adapter to allow `FoldingWriter<&mut dyn Write>` since we need ownership.
struct WriteAdapter<'a>(&'a mut dyn fmt::Write);

impl fmt::Write for WriteAdapter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(s)
    }
}

// ============================================================================
// CalendarComponent
// ============================================================================

impl WriteIcal for CalendarComponent {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            CalendarComponent::Event(e) => e.write_ical(w),
            CalendarComponent::Todo(t) => t.write_ical(w),
            CalendarComponent::Journal(j) => j.write_ical(w),
            CalendarComponent::FreeBusy(fb) => fb.write_ical(w),
            CalendarComponent::TimeZone(tz) => tz.write_ical(w),
            CalendarComponent::Other(o) => o.write_ical(w),
        }
    }
}

// ============================================================================
// Event
// ============================================================================

impl WriteIcal for Event {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VEVENT\r\n")?;

        write_opt_prop("DTSTAMP", self.dtstamp(), w)?;
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_dtod_prop("DTSTART", self.dtstart(), w)?;
        write_opt_prop("CLASS", self.class(), w)?;
        write_opt_prop("CREATED", self.created(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("GEO", self.geo(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("LOCATION", self.location(), w)?;
        write_opt_prop("ORGANIZER", self.organizer(), w)?;
        write_opt_prop("PRIORITY", self.priority(), w)?;
        write_opt_prop("SEQUENCE", self.sequence(), w)?;
        write_opt_prop("STATUS", self.status(), w)?;
        write_opt_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("TRANSP", self.transp(), w)?;
        write_opt_prop("URL", self.url(), w)?;
        write_opt_dtod_prop("RECURRENCE-ID", self.recurrence_id(), w)?;
        write_opt_dtod_prop("DTEND", self.dtend(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("COLOR", self.color(), w)?;

        // Multi-valued
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_vec_prop("CATEGORIES", self.categories(), w)?;
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_vec_prop("CONTACT", self.contact(), w)?;
        write_exdate_vec(self.exdate(), w)?;
        write_vec_prop("REQUEST-STATUS", self.request_status(), w)?;
        write_vec_prop("RELATED-TO", self.related_to(), w)?;
        write_vec_prop("RESOURCES", self.resources(), w)?;
        write_rdate_vec(self.rdate(), w)?;
        write_vec_prop("RRULE", self.rrule(), w)?;
        write_attach_vec("IMAGE", self.image(), w)?;
        write_vec_prop("CONFERENCE", self.conference(), w)?;
        write_styled_description_vec(self.styled_description(), w)?;
        write_structured_data_props(self.structured_data(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for alarm in self.alarms() {
            alarm.write_ical(w)?;
        }
        for p in self.participants() {
            p.write_ical(w)?;
        }
        for l in self.locations() {
            l.write_ical(w)?;
        }
        for r in self.resource_components() {
            r.write_ical(w)?;
        }

        w.write_str("END:VEVENT\r\n")
    }
}

// ============================================================================
// Todo
// ============================================================================

impl WriteIcal for Todo {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VTODO\r\n")?;

        write_opt_prop("DTSTAMP", self.dtstamp(), w)?;
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_dtod_prop("DTSTART", self.dtstart(), w)?;
        write_opt_prop("CLASS", self.class(), w)?;
        write_opt_prop("COMPLETED", self.completed(), w)?;
        write_opt_prop("CREATED", self.created(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("GEO", self.geo(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("LOCATION", self.location(), w)?;
        write_opt_prop("ORGANIZER", self.organizer(), w)?;
        write_opt_prop("PERCENT-COMPLETE", self.percent_complete(), w)?;
        write_opt_prop("PRIORITY", self.priority(), w)?;
        write_opt_dtod_prop("RECURRENCE-ID", self.recurrence_id(), w)?;
        write_opt_prop("SEQUENCE", self.sequence(), w)?;
        write_opt_prop("STATUS", self.status(), w)?;
        write_opt_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("URL", self.url(), w)?;
        write_opt_dtod_prop("DUE", self.due(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("COLOR", self.color(), w)?;

        // Multi-valued
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_vec_prop("CATEGORIES", self.categories(), w)?;
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_vec_prop("CONTACT", self.contact(), w)?;
        write_exdate_vec(self.exdate(), w)?;
        write_vec_prop("REQUEST-STATUS", self.request_status(), w)?;
        write_vec_prop("RELATED-TO", self.related_to(), w)?;
        write_vec_prop("RESOURCES", self.resources(), w)?;
        write_rdate_vec(self.rdate(), w)?;
        write_vec_prop("RRULE", self.rrule(), w)?;
        write_attach_vec("IMAGE", self.image(), w)?;
        write_vec_prop("CONFERENCE", self.conference(), w)?;
        write_styled_description_vec(self.styled_description(), w)?;
        write_structured_data_props(self.structured_data(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for alarm in self.alarms() {
            alarm.write_ical(w)?;
        }
        for p in self.participants() {
            p.write_ical(w)?;
        }
        for l in self.locations() {
            l.write_ical(w)?;
        }
        for r in self.resource_components() {
            r.write_ical(w)?;
        }

        w.write_str("END:VTODO\r\n")
    }
}

// ============================================================================
// Journal
// ============================================================================

impl WriteIcal for Journal {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VJOURNAL\r\n")?;

        write_prop("DTSTAMP", self.dtstamp(), w)?;
        write_prop("UID", self.uid(), w)?;
        write_opt_dtod_prop("DTSTART", self.dtstart(), w)?;
        write_opt_prop("CLASS", self.class(), w)?;
        write_opt_prop("CREATED", self.created(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("ORGANIZER", self.organizer(), w)?;
        write_opt_dtod_prop("RECURRENCE-ID", self.recurrence_id(), w)?;
        write_opt_prop("SEQUENCE", self.sequence(), w)?;
        write_opt_prop("STATUS", self.status(), w)?;
        write_opt_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("URL", self.url(), w)?;

        // Multi-valued
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_vec_prop("CATEGORIES", self.categories(), w)?;
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_vec_prop("CONTACT", self.contact(), w)?;
        write_vec_prop("DESCRIPTION", self.description(), w)?;
        write_exdate_vec(self.exdate(), w)?;
        write_vec_prop("RELATED-TO", self.related_to(), w)?;
        write_rdate_vec(self.rdate(), w)?;
        write_vec_prop("RRULE", self.rrule(), w)?;
        write_vec_prop("REQUEST-STATUS", self.request_status(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for p in self.participants() {
            p.write_ical(w)?;
        }
        for l in self.locations() {
            l.write_ical(w)?;
        }
        for r in self.resource_components() {
            r.write_ical(w)?;
        }

        w.write_str("END:VJOURNAL\r\n")
    }
}

// ============================================================================
// FreeBusy
// ============================================================================

impl WriteIcal for FreeBusy {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VFREEBUSY\r\n")?;

        write_prop("DTSTAMP", self.dtstamp(), w)?;
        write_prop("UID", self.uid(), w)?;
        write_opt_prop("CONTACT", self.contact(), w)?;
        write_opt_dtod_prop("DTSTART", self.dtstart(), w)?;
        write_opt_dtod_prop("DTEND", self.dtend(), w)?;
        write_opt_prop("ORGANIZER", self.organizer(), w)?;
        write_opt_prop("URL", self.url(), w)?;

        // Multi-valued
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_vec_prop("FREEBUSY", self.freebusy(), w)?;
        write_vec_prop("REQUEST-STATUS", self.request_status(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for p in self.participants() {
            p.write_ical(w)?;
        }
        for l in self.locations() {
            l.write_ical(w)?;
        }
        for r in self.resource_components() {
            r.write_ical(w)?;
        }

        w.write_str("END:VFREEBUSY\r\n")
    }
}

// ============================================================================
// TimeZone
// ============================================================================

impl WriteIcal for TimeZone {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VTIMEZONE\r\n")?;

        write_prop("TZID", self.tz_id(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("TZURL", self.tz_url(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for rule in self.rules() {
            rule.write_ical(w)?;
        }

        w.write_str("END:VTIMEZONE\r\n")
    }
}

// ============================================================================
// TzRule
// ============================================================================

impl WriteIcal for TzRule {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        let name = match self.kind() {
            &TzRuleKind::Standard => "STANDARD",
            &TzRuleKind::Daylight => "DAYLIGHT",
        };
        w.write_str("BEGIN:")?;
        w.write_str(name)?;
        write_crlf(w)?;

        write_dtod_prop("DTSTART", self.dtstart(), w)?;
        write_prop("TZOFFSETTO", self.tz_offset_to(), w)?;
        write_prop("TZOFFSETFROM", self.tz_offset_from(), w)?;

        // Multi-valued
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_rdate_vec(self.rdate(), w)?;
        write_vec_prop("RRULE", self.rrule(), w)?;
        write_vec_prop("TZNAME", self.tz_name(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        w.write_str("END:")?;
        w.write_str(name)?;
        write_crlf(w)
    }
}

// ============================================================================
// Alarm
// ============================================================================

impl WriteIcal for Alarm {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            Alarm::Audio(a) => a.write_ical(w),
            Alarm::Display(a) => a.write_ical(w),
            Alarm::Email(a) => a.write_ical(w),
            Alarm::Other(a) => a.write_ical(w),
        }
    }
}

impl WriteIcal for AudioAlarm {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VALARM\r\n")?;
        write_action("AUDIO", w)?;
        write_trigger_prop(self.trigger(), w)?;
        if let Some(a) = self.attach() {
            write_attach_prop("ATTACH", a, w)?;
        }
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("REPEAT", self.repeat(), w)?;
        write_opt_prop("ACKNOWLEDGED", self.acknowledged(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VALARM\r\n")
    }
}

impl WriteIcal for DisplayAlarm {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VALARM\r\n")?;
        write_action("DISPLAY", w)?;
        write_trigger_prop(self.trigger(), w)?;
        write_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("REPEAT", self.repeat(), w)?;
        write_opt_prop("ACKNOWLEDGED", self.acknowledged(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VALARM\r\n")
    }
}

impl WriteIcal for EmailAlarm {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VALARM\r\n")?;
        write_action("EMAIL", w)?;
        write_trigger_prop(self.trigger(), w)?;
        write_prop("DESCRIPTION", self.description(), w)?;
        write_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("REPEAT", self.repeat(), w)?;
        write_opt_prop("ACKNOWLEDGED", self.acknowledged(), w)?;
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VALARM\r\n")
    }
}

impl WriteIcal for OtherAlarm {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VALARM\r\n")?;
        write_prop("ACTION", self.action(), w)?;
        write_trigger_prop(self.trigger(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("UID", self.uid(), w)?;
        write_opt_prop("DURATION", self.duration(), w)?;
        write_opt_prop("REPEAT", self.repeat(), w)?;
        write_opt_prop("ACKNOWLEDGED", self.acknowledged(), w)?;
        write_vec_prop("ATTENDEE", self.attendee(), w)?;
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VALARM\r\n")
    }
}

// ============================================================================
// RFC 9073 Components
// ============================================================================

impl WriteIcal for LocationComponent {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VLOCATION\r\n")?;
        write_prop("UID", self.uid(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("GEO", self.geo(), w)?;
        write_opt_prop("NAME", self.name(), w)?;
        write_opt_prop("LOCATION-TYPE", self.location_type(), w)?;
        write_opt_prop("URL", self.url(), w)?;
        write_structured_data_props(self.structured_data(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VLOCATION\r\n")
    }
}

impl WriteIcal for ResourceComponent {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:VRESOURCE\r\n")?;
        write_prop("UID", self.uid(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("GEO", self.geo(), w)?;
        write_opt_prop("NAME", self.name(), w)?;
        write_opt_prop("RESOURCE-TYPE", self.resource_type(), w)?;
        write_structured_data_props(self.structured_data(), w)?;
        write_x_property_iter(self.x_property_iter(), w)?;
        w.write_str("END:VRESOURCE\r\n")
    }
}

impl WriteIcal for Participant {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:PARTICIPANT\r\n")?;
        write_prop("UID", self.uid(), w)?;
        write_prop("PARTICIPANT-TYPE", self.participant_type(), w)?;
        write_opt_prop("CALENDAR-ADDRESS", self.calendar_address(), w)?;
        write_opt_prop("CREATED", self.created(), w)?;
        write_opt_prop("DESCRIPTION", self.description(), w)?;
        write_opt_prop("DTSTAMP", self.dtstamp(), w)?;
        write_opt_prop("GEO", self.geo(), w)?;
        write_opt_prop("LAST-MODIFIED", self.last_modified(), w)?;
        write_opt_prop("PRIORITY", self.priority(), w)?;
        write_opt_prop("SEQUENCE", self.sequence(), w)?;
        write_opt_prop("STATUS", self.status(), w)?;
        write_opt_prop("SUMMARY", self.summary(), w)?;
        write_opt_prop("URL", self.url(), w)?;

        // Multi-valued
        write_attach_vec("ATTACH", self.attach(), w)?;
        write_vec_prop("CATEGORIES", self.categories(), w)?;
        write_vec_prop("COMMENT", self.comment(), w)?;
        write_vec_prop("CONTACT", self.contact(), w)?;
        write_vec_prop("LOCATION", self.location_prop(), w)?;
        write_vec_prop("REQUEST-STATUS", self.request_status(), w)?;
        write_vec_prop("RELATED-TO", self.related_to(), w)?;
        write_vec_prop("RESOURCES", self.resources(), w)?;
        write_styled_description_vec(self.styled_description(), w)?;
        write_structured_data_props(self.structured_data(), w)?;

        // X-properties
        write_x_property_iter(self.x_property_iter(), w)?;

        // Subcomponents
        for l in self.locations() {
            l.write_ical(w)?;
        }
        for r in self.resource_components() {
            r.write_ical(w)?;
        }

        w.write_str("END:PARTICIPANT\r\n")
    }
}

// ============================================================================
// OtherComponent
// ============================================================================

impl WriteIcal for OtherComponent {
    fn write_ical(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("BEGIN:")?;
        w.write_str(&self.name)?;
        write_crlf(w)?;
        for sub in &self.subcomponents {
            sub.write_ical(w)?;
        }
        w.write_str("END:")?;
        w.write_str(&self.name)?;
        write_crlf(w)
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn write_action(action: &str, w: &mut dyn fmt::Write) -> fmt::Result {
    w.write_str("ACTION:")?;
    w.write_str(action)?;
    write_crlf(w)
}

fn write_attach_vec(
    name: &str,
    props: Option<&Vec<crate::model::property::Prop<rfc5545_types::value::Attachment, Params>>>,
    w: &mut dyn fmt::Write,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_attach_prop(name, p, w)?;
        }
    }
    Ok(())
}

fn write_rdate_vec(
    props: Option<&Vec<crate::model::property::Prop<rfc5545_types::time::RDateSeq, Params>>>,
    w: &mut dyn fmt::Write,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_rdate_seq_prop("RDATE", p, w)?;
        }
    }
    Ok(())
}

fn write_exdate_vec(
    props: Option<&Vec<crate::model::property::Prop<rfc5545_types::time::DateTimeOrDate, Params>>>,
    w: &mut dyn fmt::Write,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_exdate_prop(p, w)?;
        }
    }
    Ok(())
}

fn write_styled_description_vec(
    props: Option<&Vec<crate::model::property::Prop<rfc5545_types::value::StyledDescriptionValue, Params>>>,
    w: &mut dyn fmt::Write,
) -> fmt::Result {
    if let Some(ps) = props {
        for p in ps {
            write_styled_description_prop(p, w)?;
        }
    }
    Ok(())
}
