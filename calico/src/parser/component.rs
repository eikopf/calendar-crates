//! Parsers for the components of an iCalendar object.

use std::{collections::HashMap, hash::Hash};

use winnow::{
    Parser,
    ascii::Caseless,
    combinator::{alt, empty, eof, peek, preceded, repeat, terminated},
    error::{FromExternalError, ParserError},
    stream::{AsBStr, AsChar, Compare, SliceLen, Stream, StreamIsPartial},
};

use crate::{
    model::component::{
        Alarm, AudioAlarm, Calendar, CalendarComponent, DisplayAlarm,
        EmailAlarm, Event, FreeBusy, Journal, LocationComponent, OtherAlarm,
        OtherComponent, Participant, ResourceComponent, TimeZone, Todo, TzRule, TzRuleKind,
    },
    model::parameter::Params,
    model::primitive::{
        Attachment, ClassValue, CompletionPercentage, DateTime, DateTimeOrDate, ExDateSeq,
        Geo, Gregorian, Integer, Method, ParticipantType, Period, Priority, RDateSeq,
        RequestStatus, ResourceType, SignedDuration, Status, StyledDescriptionValue,
        TimeTransparency, Token, TriggerValue, Utc, UtcOffset, Value, Version,
    },
    model::property::{Prop, StaticProp, StructuredDataProp},
    model::rrule::RRule,
    model::string::{CaselessStr, TzId, Uid, Uri},
    model::css::Css3Color,
    parser::{
        InputStream,
        error::{CalendarParseError, ComponentKind},
        property::{ParsedProp, KnownProp, PropValue, UnknownProp, PropName, property},
    },
};

// ============================================================================
// Macro helpers for property parsing inside components
// ============================================================================

/// Parses properties in a loop, breaking when BEGIN or END is encountered.
/// The body of the match is provided by the caller.
macro_rules! parse_props {
    ($input:ident, $parsed:ident, $body:block) => {
        loop {
            let checkpoint = $input.checkpoint();
            if alt((begin(empty::<I, E>), end(empty::<I, E>))).parse_next($input).is_ok() {
                $input.reset(&checkpoint);
                break;
            }
            $input.reset(&checkpoint);

            let $parsed: ParsedProp<I::Slice> = terminated(property, crlf).parse_next($input)?;
            let result: Result<(), CalendarParseError<I::Slice>> = (|| $body)();
            result.map_err(|e| E::from_external_error($input, e))?;
        }
    };
}

/// Checks that a once-only property hasn't been set yet.
macro_rules! once {
    ($opt:expr, $prop:expr, $component:expr, $val:expr) => {
        if $opt.is_some() {
            return Err(CalendarParseError::MoreThanOneProp {
                prop: PropName::Known($prop),
                component: $component,
            });
        }
        $opt = Some($val);
    };
}

/// Handles unknown properties by inserting into the x_props map.
macro_rules! handle_unknown {
    ($x_props:ident, $name:expr, $params:expr, $value:expr) => {{
        let name_str: Box<CaselessStr> = $name.into();
        $x_props
            .entry(name_str)
            .or_insert_with(Vec::new)
            .push(Prop {
                value: $value,
                params: $params,
            });
    }};
}

// ============================================================================
// Calendar parser (RFC 5545 §3.4)
// ============================================================================

pub fn calendar<I, E>(input: &mut I) -> Result<Calendar, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + Hash + AsRef<[u8]>,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VCALENDAR")), crlf).parse_next(input)?;

    // Once-only properties
    let mut prod_id: Option<Prop<String, Params>> = None;
    let mut version: Option<Prop<Version, Params>> = None;
    let mut cal_scale: Option<Prop<Token<Gregorian, String>, Params>> = None;
    let mut method: Option<Prop<Token<Method, String>, Params>> = None;
    // RFC 7986 optional
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    let mut refresh_interval: Option<Prop<SignedDuration, Params>> = None;
    let mut source: Option<Prop<Box<Uri>, Params>> = None;
    let mut color: Option<Prop<Css3Color, Params>> = None;
    // Multi-valued
    let mut name: Vec<Prop<String, Params>> = Vec::new();
    let mut description: Vec<Prop<String, Params>> = Vec::new();
    let mut categories: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut image: Vec<Prop<Attachment, Params>> = Vec::new();
    // Unknown
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    // Parse properties and subcomponents in any order (real-world .ics files
    // freely interleave them, even though RFC 5545 grammar suggests props-first).
    let mut components: Vec<CalendarComponent> = Vec::new();

    loop {
        // Check for END:VCALENDAR — we're done
        let checkpoint = input.checkpoint();
        if end(empty::<I, E>).parse_next(input).is_ok() {
            input.reset(&checkpoint);
            break;
        }
        input.reset(&checkpoint);

        // Try to parse a subcomponent (BEGIN:...)
        let checkpoint = input.checkpoint();
        if begin(empty::<I, E>).parse_next(input).is_ok() {
            input.reset(&checkpoint);
            components.push(calendar_component(input)?);
            continue;
        }
        input.reset(&checkpoint);

        // Otherwise parse a property
        let parsed: ParsedProp<I::Slice> = terminated(property, crlf).parse_next(input)?;
        let result: Result<(), CalendarParseError<I::Slice>> = (|| {
            match parsed {
                ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                    match (prop_name, value) {
                        (StaticProp::ProdId, PropValue::Text(p)) => {
                            once!(prod_id, StaticProp::ProdId, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Version, PropValue::Version(p)) => {
                            once!(version, StaticProp::Version, ComponentKind::Calendar, p);
                        }
                        (StaticProp::CalScale, PropValue::Gregorian(p)) => {
                            once!(cal_scale, StaticProp::CalScale, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Method, PropValue::Method(p)) => {
                            once!(method, StaticProp::Method, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Uid, PropValue::Uid(p)) => {
                            once!(uid, StaticProp::Uid, ComponentKind::Calendar, p);
                        }
                        (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                            once!(last_modified, StaticProp::LastModified, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Url, PropValue::Uri(p)) => {
                            once!(url, StaticProp::Url, ComponentKind::Calendar, p);
                        }
                        (StaticProp::RefreshInterval, PropValue::Duration(p)) => {
                            once!(refresh_interval, StaticProp::RefreshInterval, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Source, PropValue::Uri(p)) => {
                            once!(source, StaticProp::Source, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Color, PropValue::Color(p)) => {
                            once!(color, StaticProp::Color, ComponentKind::Calendar, p);
                        }
                        (StaticProp::Name, PropValue::Text(p)) => {
                            name.push(p);
                        }
                        (StaticProp::Description, PropValue::Text(p)) => {
                            description.push(p);
                        }
                        (StaticProp::Categories, PropValue::TextSeq(p)) => {
                            categories.push(p);
                        }
                        (StaticProp::Image, PropValue::Attachment(p)) => {
                            image.push(p);
                        }
                        _ => { /* ignore - property parser guarantees correct variant */ }
                    }
                }
                ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                    let name_string: String = I::try_into_string(&uname)?;
                    handle_unknown!(x_props, name_string, params, value);
                }
            }
            Ok(())
        })();
        result.map_err(|e| E::from_external_error(input, e))?;
    }

    terminated(end(Caseless("VCALENDAR")), alt((crlf, eof))).parse_next(input)?;

    // Check mandatory fields
    let version = version.ok_or_else(|| {
        E::from_external_error(
            input,
            CalendarParseError::MissingProp {
                prop: PropName::Known(StaticProp::Version),
                component: ComponentKind::Calendar,
            },
        )
    })?;

    let mut cal = Calendar::new(version, components);
    if let Some(v) = prod_id { cal.set_prod_id(v); }
    if let Some(v) = cal_scale { cal.set_cal_scale(v); }
    if let Some(v) = method { cal.set_method(v); }
    if let Some(v) = uid { cal.set_uid(v); }
    if let Some(v) = last_modified { cal.set_last_modified(v); }
    if let Some(v) = url { cal.set_url(v); }
    if let Some(v) = refresh_interval { cal.set_refresh_interval(v); }
    if let Some(v) = source { cal.set_source(v); }
    if let Some(v) = color { cal.set_color(v); }
    if !name.is_empty() { cal.set_name(name); }
    if !description.is_empty() { cal.set_description(description); }
    if !categories.is_empty() { cal.set_categories(categories); }
    if !image.is_empty() { cal.set_image(image); }
    for (k, v) in x_props {
        cal.insert_x_property(k, v);
    }

    Ok(cal)
}

// ============================================================================
// CalendarComponent dispatcher
// ============================================================================

/// Parses a [`CalendarComponent`].
pub fn calendar_component<I, E>(input: &mut I) -> Result<CalendarComponent, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + Hash + AsRef<[u8]>,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    // Peek at the component name by trying each known BEGIN:<name>
    macro_rules! try_component {
        ($name:literal, $parser:expr, $variant:expr) => {{
            let checkpoint = input.checkpoint();
            let matched: Result<I::Slice, E> = begin(Caseless($name)).parse_next(input);
            input.reset(&checkpoint);
            if matched.is_ok() {
                return $parser.map($variant).parse_next(input);
            }
        }};
    }

    try_component!("VEVENT", event, CalendarComponent::Event);
    try_component!("VTODO", todo_comp, CalendarComponent::Todo);
    try_component!("VJOURNAL", journal, CalendarComponent::Journal);
    try_component!("VFREEBUSY", free_busy, CalendarComponent::FreeBusy);
    try_component!("VTIMEZONE", timezone, CalendarComponent::TimeZone);

    // Anything else (including VALARM at calendar level) → other
    other_with_name.map(CalendarComponent::Other).parse_next(input)
}

// ============================================================================
// Event parser (RFC 5545 §3.6.1)
// ============================================================================

/// Parses a [`Event`].
fn event<I, E>(input: &mut I) -> Result<Event, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VEVENT")), crlf).parse_next(input)?;

    // Once-only properties
    let mut dtstamp: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut dtstart: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut class: Option<Prop<Token<ClassValue, String>, Params>> = None;
    let mut created: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut geo: Option<Prop<Geo, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut loc_prop: Option<Prop<String, Params>> = None;
    let mut organizer: Option<Prop<Box<Uri>, Params>> = None;
    let mut priority: Option<Prop<Priority, Params>> = None;
    let mut sequence: Option<Prop<Integer, Params>> = None;
    let mut status: Option<Prop<Status, Params>> = None;
    let mut summary: Option<Prop<String, Params>> = None;
    let mut transp: Option<Prop<TimeTransparency, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    let mut recurrence_id: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut dtend: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut duration: Option<Prop<SignedDuration, Params>> = None;
    let mut color: Option<Prop<Css3Color, Params>> = None;
    // Multi-valued
    let mut attach: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut attendee: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut categories: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut contact: Vec<Prop<String, Params>> = Vec::new();
    let mut exdate: Vec<Prop<DateTimeOrDate, Params>> = Vec::new();
    let mut request_status: Vec<Prop<RequestStatus, Params>> = Vec::new();
    let mut related_to: Vec<Prop<Box<Uid>, Params>> = Vec::new();
    let mut resources: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut rdate: Vec<Prop<RDateSeq, Params>> = Vec::new();
    let mut rrule: Vec<Prop<RRule, Params>> = Vec::new();
    let mut image: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut conference: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut styled_description: Vec<Prop<StyledDescriptionValue, Params>> = Vec::new();
    let mut structured_data: Vec<StructuredDataProp> = Vec::new();
    // Unknown
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::DtStamp, PropValue::DateTimeUtc(p)) => {
                        once!(dtstamp, StaticProp::DtStamp, ComponentKind::Event, p);
                    }
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Event, p);
                    }
                    (StaticProp::DtStart, PropValue::DateTimeOrDate(p)) => {
                        once!(dtstart, StaticProp::DtStart, ComponentKind::Event, p);
                    }
                    (StaticProp::Class, PropValue::ClassValue(p)) => {
                        once!(class, StaticProp::Class, ComponentKind::Event, p);
                    }
                    (StaticProp::Created, PropValue::DateTimeUtc(p)) => {
                        once!(created, StaticProp::Created, ComponentKind::Event, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Event, p);
                    }
                    (StaticProp::Geo, PropValue::Geo(p)) => {
                        once!(geo, StaticProp::Geo, ComponentKind::Event, p);
                    }
                    (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                        once!(last_modified, StaticProp::LastModified, ComponentKind::Event, p);
                    }
                    (StaticProp::Location, PropValue::Text(p)) => {
                        once!(loc_prop, StaticProp::Location, ComponentKind::Event, p);
                    }
                    (StaticProp::Organizer, PropValue::Uri(p)) => {
                        once!(organizer, StaticProp::Organizer, ComponentKind::Event, p);
                    }
                    (StaticProp::Priority, PropValue::Priority(p)) => {
                        once!(priority, StaticProp::Priority, ComponentKind::Event, p);
                    }
                    (StaticProp::Sequence, PropValue::Integer(p)) => {
                        once!(sequence, StaticProp::Sequence, ComponentKind::Event, p);
                    }
                    (StaticProp::Status, PropValue::Status(p)) => {
                        if status.is_some() {
                            return Err(CalendarParseError::MoreThanOneProp {
                                prop: PropName::Known(StaticProp::Status),
                                component: ComponentKind::Event,
                            });
                        }
                        match p.value {
                            Status::Tentative | Status::Confirmed | Status::Cancelled => {}
                            s => return Err(CalendarParseError::InvalidEventStatus(s)),
                        }
                        status = Some(p);
                    }
                    (StaticProp::Summary, PropValue::Text(p)) => {
                        once!(summary, StaticProp::Summary, ComponentKind::Event, p);
                    }
                    (StaticProp::Transp, PropValue::TimeTransparency(p)) => {
                        once!(transp, StaticProp::Transp, ComponentKind::Event, p);
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::Event, p);
                    }
                    (StaticProp::RecurId, PropValue::DateTimeOrDate(p)) => {
                        once!(recurrence_id, StaticProp::RecurId, ComponentKind::Event, p);
                    }
                    (StaticProp::DtEnd, PropValue::DateTimeOrDate(p)) => {
                        if duration.is_some() {
                            return Err(CalendarParseError::EventTerminationCollision);
                        }
                        once!(dtend, StaticProp::DtEnd, ComponentKind::Event, p);
                    }
                    (StaticProp::Duration, PropValue::Duration(p)) => {
                        if dtend.is_some() {
                            return Err(CalendarParseError::EventTerminationCollision);
                        }
                        once!(duration, StaticProp::Duration, ComponentKind::Event, p);
                    }
                    (StaticProp::Color, PropValue::Color(p)) => {
                        once!(color, StaticProp::Color, ComponentKind::Event, p);
                    }
                    // Multi-valued
                    (StaticProp::Attach, PropValue::Attachment(p)) => {
                        attach.push(p);
                    }
                    (StaticProp::Attendee, PropValue::Uri(p)) => {
                        attendee.push(p);
                    }
                    (StaticProp::Categories, PropValue::TextSeq(p)) => {
                        categories.push(p);
                    }
                    (StaticProp::Comment, PropValue::Text(p)) => {
                        comment.push(p);
                    }
                    (StaticProp::Contact, PropValue::Text(p)) => {
                        contact.push(p);
                    }
                    (StaticProp::ExDate, PropValue::ExDateSeq(seq, params)) => {
                        match seq {
                            ExDateSeq::DateTime(dates) => {
                                for dt in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::DateTime(dt), params: params.clone() });
                                }
                            }
                            ExDateSeq::Date(dates) => {
                                for d in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::Date(d), params: params.clone() });
                                }
                            }
                        }
                    }
                    (StaticProp::RequestStatus, PropValue::RequestStatus(p)) => {
                        request_status.push(p);
                    }
                    (StaticProp::RelatedTo, PropValue::Uid(p)) => {
                        related_to.push(p);
                    }
                    (StaticProp::Resources, PropValue::TextSeq(p)) => {
                        resources.push(p);
                    }
                    (StaticProp::RDate, PropValue::RDateSeq(p)) => {
                        rdate.push(p);
                    }
                    (StaticProp::RRule, PropValue::RRule(p)) => {
                        rrule.push(p);
                    }
                    (StaticProp::Image, PropValue::Attachment(p)) => {
                        image.push(p);
                    }
                    (StaticProp::Conference, PropValue::Uri(p)) => {
                        conference.push(p);
                    }
                    (StaticProp::StyledDescription, PropValue::StyledDescription(p)) => {
                        styled_description.push(p);
                    }
                    (StaticProp::StructuredData, PropValue::StructuredData(p)) => {
                        structured_data.push(p);
                    }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse subcomponents
    let alarms: Vec<Alarm> = repeat(0.., preceded(peek(begin(Caseless("VALARM"))), alarm)).parse_next(input)?;
    let participants: Vec<Participant> = repeat(0.., preceded(peek(begin(Caseless("PARTICIPANT"))), participant)).parse_next(input)?;
    let locations: Vec<LocationComponent> = repeat(0.., preceded(peek(begin(Caseless("VLOCATION"))), location)).parse_next(input)?;
    let resource_components: Vec<ResourceComponent> = repeat(0.., preceded(peek(begin(Caseless("VRESOURCE"))), resource)).parse_next(input)?;

    terminated(end(Caseless("VEVENT")), crlf).parse_next(input)?;

    // Check mandatory fields
    let dtstamp = dtstamp.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::DtStamp),
            component: ComponentKind::Event,
        })
    })?;
    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Event,
        })
    })?;

    let mut ev = Event::new(dtstamp, uid, alarms, participants, locations, resource_components);
    if let Some(v) = dtstart { ev.set_dtstart(v); }
    if let Some(v) = class { ev.set_class(v); }
    if let Some(v) = created { ev.set_created(v); }
    if let Some(v) = description { ev.set_description(v); }
    if let Some(v) = geo { ev.set_geo(v); }
    if let Some(v) = last_modified { ev.set_last_modified(v); }
    if let Some(v) = loc_prop { ev.set_location(v); }
    if let Some(v) = organizer { ev.set_organizer(v); }
    if let Some(v) = priority { ev.set_priority(v); }
    if let Some(v) = sequence { ev.set_sequence(v); }
    if let Some(v) = status { ev.set_status(v); }
    if let Some(v) = summary { ev.set_summary(v); }
    if let Some(v) = transp { ev.set_transp(v); }
    if let Some(v) = url { ev.set_url(v); }
    if let Some(v) = recurrence_id { ev.set_recurrence_id(v); }
    if let Some(v) = dtend { ev.set_dtend(v); }
    if let Some(v) = duration { ev.set_duration(v); }
    if let Some(v) = color { ev.set_color(v); }
    if !attach.is_empty() { ev.set_attach(attach); }
    if !attendee.is_empty() { ev.set_attendee(attendee); }
    if !categories.is_empty() { ev.set_categories(categories); }
    if !comment.is_empty() { ev.set_comment(comment); }
    if !contact.is_empty() { ev.set_contact(contact); }
    if !exdate.is_empty() { ev.set_exdate(exdate); }
    if !request_status.is_empty() { ev.set_request_status(request_status); }
    if !related_to.is_empty() { ev.set_related_to(related_to); }
    if !resources.is_empty() { ev.set_resources(resources); }
    if !rdate.is_empty() { ev.set_rdate(rdate); }
    if !rrule.is_empty() { ev.set_rrule(rrule); }
    if !image.is_empty() { ev.set_image(image); }
    if !conference.is_empty() { ev.set_conference(conference); }
    if !styled_description.is_empty() { ev.set_styled_description(styled_description); }
    if !structured_data.is_empty() { ev.set_structured_data(structured_data); }
    for (k, v) in x_props {
        ev.insert_x_property(k, v);
    }

    Ok(ev)
}

// ============================================================================
// Todo parser (RFC 5545 §3.6.2)
// ============================================================================

/// Parses a [`Todo`].
fn todo_comp<I, E>(input: &mut I) -> Result<Todo, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VTODO")), crlf).parse_next(input)?;

    // Once-only properties
    let mut dtstamp: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut dtstart: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut class: Option<Prop<Token<ClassValue, String>, Params>> = None;
    let mut completed: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut created: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut geo: Option<Prop<Geo, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut loc_prop: Option<Prop<String, Params>> = None;
    let mut organizer: Option<Prop<Box<Uri>, Params>> = None;
    let mut percent_complete: Option<Prop<CompletionPercentage, Params>> = None;
    let mut priority: Option<Prop<Priority, Params>> = None;
    let mut recurrence_id: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut sequence: Option<Prop<Integer, Params>> = None;
    let mut status: Option<Prop<Status, Params>> = None;
    let mut summary: Option<Prop<String, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    let mut due: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut duration: Option<Prop<SignedDuration, Params>> = None;
    let mut color: Option<Prop<Css3Color, Params>> = None;
    // Multi-valued
    let mut attach: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut attendee: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut categories: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut contact: Vec<Prop<String, Params>> = Vec::new();
    let mut exdate: Vec<Prop<DateTimeOrDate, Params>> = Vec::new();
    let mut request_status: Vec<Prop<RequestStatus, Params>> = Vec::new();
    let mut related_to: Vec<Prop<Box<Uid>, Params>> = Vec::new();
    let mut resources: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut rdate: Vec<Prop<RDateSeq, Params>> = Vec::new();
    let mut rrule: Vec<Prop<RRule, Params>> = Vec::new();
    let mut image: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut conference: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut styled_description: Vec<Prop<StyledDescriptionValue, Params>> = Vec::new();
    let mut structured_data: Vec<StructuredDataProp> = Vec::new();
    // Unknown
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::DtStamp, PropValue::DateTimeUtc(p)) => {
                        once!(dtstamp, StaticProp::DtStamp, ComponentKind::Todo, p);
                    }
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Todo, p);
                    }
                    (StaticProp::DtStart, PropValue::DateTimeOrDate(p)) => {
                        once!(dtstart, StaticProp::DtStart, ComponentKind::Todo, p);
                    }
                    (StaticProp::Class, PropValue::ClassValue(p)) => {
                        once!(class, StaticProp::Class, ComponentKind::Todo, p);
                    }
                    (StaticProp::DtCompleted, PropValue::DateTimeUtc(p)) => {
                        once!(completed, StaticProp::DtCompleted, ComponentKind::Todo, p);
                    }
                    (StaticProp::Created, PropValue::DateTimeUtc(p)) => {
                        once!(created, StaticProp::Created, ComponentKind::Todo, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Todo, p);
                    }
                    (StaticProp::Geo, PropValue::Geo(p)) => {
                        once!(geo, StaticProp::Geo, ComponentKind::Todo, p);
                    }
                    (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                        once!(last_modified, StaticProp::LastModified, ComponentKind::Todo, p);
                    }
                    (StaticProp::Location, PropValue::Text(p)) => {
                        once!(loc_prop, StaticProp::Location, ComponentKind::Todo, p);
                    }
                    (StaticProp::Organizer, PropValue::Uri(p)) => {
                        once!(organizer, StaticProp::Organizer, ComponentKind::Todo, p);
                    }
                    (StaticProp::PercentComplete, PropValue::CompletionPercentage(p)) => {
                        once!(percent_complete, StaticProp::PercentComplete, ComponentKind::Todo, p);
                    }
                    (StaticProp::Priority, PropValue::Priority(p)) => {
                        once!(priority, StaticProp::Priority, ComponentKind::Todo, p);
                    }
                    (StaticProp::RecurId, PropValue::DateTimeOrDate(p)) => {
                        once!(recurrence_id, StaticProp::RecurId, ComponentKind::Todo, p);
                    }
                    (StaticProp::Sequence, PropValue::Integer(p)) => {
                        once!(sequence, StaticProp::Sequence, ComponentKind::Todo, p);
                    }
                    (StaticProp::Status, PropValue::Status(p)) => {
                        if status.is_some() {
                            return Err(CalendarParseError::MoreThanOneProp {
                                prop: PropName::Known(StaticProp::Status),
                                component: ComponentKind::Todo,
                            });
                        }
                        match p.value {
                            Status::NeedsAction | Status::Completed | Status::InProcess | Status::Cancelled => {}
                            s => return Err(CalendarParseError::InvalidTodoStatus(s)),
                        }
                        status = Some(p);
                    }
                    (StaticProp::Summary, PropValue::Text(p)) => {
                        once!(summary, StaticProp::Summary, ComponentKind::Todo, p);
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::Todo, p);
                    }
                    (StaticProp::DtDue, PropValue::DateTimeOrDate(p)) => {
                        if duration.is_some() {
                            return Err(CalendarParseError::TodoTerminationCollision);
                        }
                        once!(due, StaticProp::DtDue, ComponentKind::Todo, p);
                    }
                    (StaticProp::Duration, PropValue::Duration(p)) => {
                        if due.is_some() {
                            return Err(CalendarParseError::TodoTerminationCollision);
                        }
                        once!(duration, StaticProp::Duration, ComponentKind::Todo, p);
                    }
                    (StaticProp::Color, PropValue::Color(p)) => {
                        once!(color, StaticProp::Color, ComponentKind::Todo, p);
                    }
                    // Multi-valued
                    (StaticProp::Attach, PropValue::Attachment(p)) => { attach.push(p); }
                    (StaticProp::Attendee, PropValue::Uri(p)) => { attendee.push(p); }
                    (StaticProp::Categories, PropValue::TextSeq(p)) => { categories.push(p); }
                    (StaticProp::Comment, PropValue::Text(p)) => { comment.push(p); }
                    (StaticProp::Contact, PropValue::Text(p)) => { contact.push(p); }
                    (StaticProp::ExDate, PropValue::ExDateSeq(seq, params)) => {
                        match seq {
                            ExDateSeq::DateTime(dates) => {
                                for dt in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::DateTime(dt), params: params.clone() });
                                }
                            }
                            ExDateSeq::Date(dates) => {
                                for d in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::Date(d), params: params.clone() });
                                }
                            }
                        }
                    }
                    (StaticProp::RequestStatus, PropValue::RequestStatus(p)) => { request_status.push(p); }
                    (StaticProp::RelatedTo, PropValue::Uid(p)) => { related_to.push(p); }
                    (StaticProp::Resources, PropValue::TextSeq(p)) => { resources.push(p); }
                    (StaticProp::RDate, PropValue::RDateSeq(p)) => { rdate.push(p); }
                    (StaticProp::RRule, PropValue::RRule(p)) => { rrule.push(p); }
                    (StaticProp::Image, PropValue::Attachment(p)) => { image.push(p); }
                    (StaticProp::Conference, PropValue::Uri(p)) => { conference.push(p); }
                    (StaticProp::StyledDescription, PropValue::StyledDescription(p)) => { styled_description.push(p); }
                    (StaticProp::StructuredData, PropValue::StructuredData(p)) => { structured_data.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse subcomponents
    let alarms: Vec<Alarm> = repeat(0.., preceded(peek(begin(Caseless("VALARM"))), alarm)).parse_next(input)?;
    let participants: Vec<Participant> = repeat(0.., preceded(peek(begin(Caseless("PARTICIPANT"))), participant)).parse_next(input)?;
    let locations: Vec<LocationComponent> = repeat(0.., preceded(peek(begin(Caseless("VLOCATION"))), location)).parse_next(input)?;
    let resource_components: Vec<ResourceComponent> = repeat(0.., preceded(peek(begin(Caseless("VRESOURCE"))), resource)).parse_next(input)?;

    terminated(end(Caseless("VTODO")), crlf).parse_next(input)?;

    let dtstamp = dtstamp.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::DtStamp),
            component: ComponentKind::Todo,
        })
    })?;
    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Todo,
        })
    })?;

    let mut td = Todo::new(dtstamp, uid, alarms, participants, locations, resource_components);
    if let Some(v) = dtstart { td.set_dtstart(v); }
    if let Some(v) = class { td.set_class(v); }
    if let Some(v) = completed { td.set_completed(v); }
    if let Some(v) = created { td.set_created(v); }
    if let Some(v) = description { td.set_description(v); }
    if let Some(v) = geo { td.set_geo(v); }
    if let Some(v) = last_modified { td.set_last_modified(v); }
    if let Some(v) = loc_prop { td.set_location(v); }
    if let Some(v) = organizer { td.set_organizer(v); }
    if let Some(v) = percent_complete { td.set_percent_complete(v); }
    if let Some(v) = priority { td.set_priority(v); }
    if let Some(v) = recurrence_id { td.set_recurrence_id(v); }
    if let Some(v) = sequence { td.set_sequence(v); }
    if let Some(v) = status { td.set_status(v); }
    if let Some(v) = summary { td.set_summary(v); }
    if let Some(v) = url { td.set_url(v); }
    if let Some(v) = due { td.set_due(v); }
    if let Some(v) = duration { td.set_duration(v); }
    if let Some(v) = color { td.set_color(v); }
    if !attach.is_empty() { td.set_attach(attach); }
    if !attendee.is_empty() { td.set_attendee(attendee); }
    if !categories.is_empty() { td.set_categories(categories); }
    if !comment.is_empty() { td.set_comment(comment); }
    if !contact.is_empty() { td.set_contact(contact); }
    if !exdate.is_empty() { td.set_exdate(exdate); }
    if !request_status.is_empty() { td.set_request_status(request_status); }
    if !related_to.is_empty() { td.set_related_to(related_to); }
    if !resources.is_empty() { td.set_resources(resources); }
    if !rdate.is_empty() { td.set_rdate(rdate); }
    if !rrule.is_empty() { td.set_rrule(rrule); }
    if !image.is_empty() { td.set_image(image); }
    if !conference.is_empty() { td.set_conference(conference); }
    if !styled_description.is_empty() { td.set_styled_description(styled_description); }
    if !structured_data.is_empty() { td.set_structured_data(structured_data); }
    for (k, v) in x_props {
        td.insert_x_property(k, v);
    }

    Ok(td)
}

// ============================================================================
// Journal parser (RFC 5545 §3.6.3)
// ============================================================================

/// Parses a [`Journal`].
fn journal<I, E>(input: &mut I) -> Result<Journal, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VJOURNAL")), crlf).parse_next(input)?;

    let mut dtstamp: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut dtstart: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut class: Option<Prop<Token<ClassValue, String>, Params>> = None;
    let mut created: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut organizer: Option<Prop<Box<Uri>, Params>> = None;
    let mut recurrence_id: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut sequence: Option<Prop<Integer, Params>> = None;
    let mut status: Option<Prop<Status, Params>> = None;
    let mut summary: Option<Prop<String, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    // Multi-valued
    let mut attach: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut attendee: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut categories: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut contact: Vec<Prop<String, Params>> = Vec::new();
    let mut description: Vec<Prop<String, Params>> = Vec::new();
    let mut exdate: Vec<Prop<DateTimeOrDate, Params>> = Vec::new();
    let mut related_to: Vec<Prop<Box<Uid>, Params>> = Vec::new();
    let mut rdate: Vec<Prop<RDateSeq, Params>> = Vec::new();
    let mut rrule: Vec<Prop<RRule, Params>> = Vec::new();
    let mut request_status: Vec<Prop<RequestStatus, Params>> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::DtStamp, PropValue::DateTimeUtc(p)) => {
                        once!(dtstamp, StaticProp::DtStamp, ComponentKind::Journal, p);
                    }
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Journal, p);
                    }
                    (StaticProp::DtStart, PropValue::DateTimeOrDate(p)) => {
                        once!(dtstart, StaticProp::DtStart, ComponentKind::Journal, p);
                    }
                    (StaticProp::Class, PropValue::ClassValue(p)) => {
                        once!(class, StaticProp::Class, ComponentKind::Journal, p);
                    }
                    (StaticProp::Created, PropValue::DateTimeUtc(p)) => {
                        once!(created, StaticProp::Created, ComponentKind::Journal, p);
                    }
                    (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                        once!(last_modified, StaticProp::LastModified, ComponentKind::Journal, p);
                    }
                    (StaticProp::Organizer, PropValue::Uri(p)) => {
                        once!(organizer, StaticProp::Organizer, ComponentKind::Journal, p);
                    }
                    (StaticProp::RecurId, PropValue::DateTimeOrDate(p)) => {
                        once!(recurrence_id, StaticProp::RecurId, ComponentKind::Journal, p);
                    }
                    (StaticProp::Sequence, PropValue::Integer(p)) => {
                        once!(sequence, StaticProp::Sequence, ComponentKind::Journal, p);
                    }
                    (StaticProp::Status, PropValue::Status(p)) => {
                        if status.is_some() {
                            return Err(CalendarParseError::MoreThanOneProp {
                                prop: PropName::Known(StaticProp::Status),
                                component: ComponentKind::Journal,
                            });
                        }
                        match p.value {
                            Status::Draft | Status::Final | Status::Cancelled => {}
                            s => return Err(CalendarParseError::InvalidJournalStatus(s)),
                        }
                        status = Some(p);
                    }
                    (StaticProp::Summary, PropValue::Text(p)) => {
                        once!(summary, StaticProp::Summary, ComponentKind::Journal, p);
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::Journal, p);
                    }
                    // Multi-valued
                    (StaticProp::Attach, PropValue::Attachment(p)) => { attach.push(p); }
                    (StaticProp::Attendee, PropValue::Uri(p)) => { attendee.push(p); }
                    (StaticProp::Categories, PropValue::TextSeq(p)) => { categories.push(p); }
                    (StaticProp::Comment, PropValue::Text(p)) => { comment.push(p); }
                    (StaticProp::Contact, PropValue::Text(p)) => { contact.push(p); }
                    (StaticProp::Description, PropValue::Text(p)) => { description.push(p); }
                    (StaticProp::ExDate, PropValue::ExDateSeq(seq, params)) => {
                        match seq {
                            ExDateSeq::DateTime(dates) => {
                                for dt in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::DateTime(dt), params: params.clone() });
                                }
                            }
                            ExDateSeq::Date(dates) => {
                                for d in dates {
                                    exdate.push(Prop { value: DateTimeOrDate::Date(d), params: params.clone() });
                                }
                            }
                        }
                    }
                    (StaticProp::RelatedTo, PropValue::Uid(p)) => { related_to.push(p); }
                    (StaticProp::RDate, PropValue::RDateSeq(p)) => { rdate.push(p); }
                    (StaticProp::RRule, PropValue::RRule(p)) => { rrule.push(p); }
                    (StaticProp::RequestStatus, PropValue::RequestStatus(p)) => { request_status.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse subcomponents
    let participants: Vec<Participant> = repeat(0.., preceded(peek(begin(Caseless("PARTICIPANT"))), participant)).parse_next(input)?;
    let locations: Vec<LocationComponent> = repeat(0.., preceded(peek(begin(Caseless("VLOCATION"))), location)).parse_next(input)?;
    let resource_components: Vec<ResourceComponent> = repeat(0.., preceded(peek(begin(Caseless("VRESOURCE"))), resource)).parse_next(input)?;

    terminated(end(Caseless("VJOURNAL")), crlf).parse_next(input)?;

    let dtstamp = dtstamp.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::DtStamp),
            component: ComponentKind::Journal,
        })
    })?;
    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Journal,
        })
    })?;

    let mut jn = Journal::new(dtstamp, uid, participants, locations, resource_components);
    if let Some(v) = dtstart { jn.set_dtstart(v); }
    if let Some(v) = class { jn.set_class(v); }
    if let Some(v) = created { jn.set_created(v); }
    if let Some(v) = last_modified { jn.set_last_modified(v); }
    if let Some(v) = organizer { jn.set_organizer(v); }
    if let Some(v) = recurrence_id { jn.set_recurrence_id(v); }
    if let Some(v) = sequence { jn.set_sequence(v); }
    if let Some(v) = status { jn.set_status(v); }
    if let Some(v) = summary { jn.set_summary(v); }
    if let Some(v) = url { jn.set_url(v); }
    if !attach.is_empty() { jn.set_attach(attach); }
    if !attendee.is_empty() { jn.set_attendee(attendee); }
    if !categories.is_empty() { jn.set_categories(categories); }
    if !comment.is_empty() { jn.set_comment(comment); }
    if !contact.is_empty() { jn.set_contact(contact); }
    if !description.is_empty() { jn.set_description(description); }
    if !exdate.is_empty() { jn.set_exdate(exdate); }
    if !related_to.is_empty() { jn.set_related_to(related_to); }
    if !rdate.is_empty() { jn.set_rdate(rdate); }
    if !rrule.is_empty() { jn.set_rrule(rrule); }
    if !request_status.is_empty() { jn.set_request_status(request_status); }
    for (k, v) in x_props {
        jn.insert_x_property(k, v);
    }

    Ok(jn)
}

// ============================================================================
// FreeBusy parser (RFC 5545 §3.6.4)
// ============================================================================

/// Parses a [`FreeBusy`].
fn free_busy<I, E>(input: &mut I) -> Result<FreeBusy, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VFREEBUSY")), crlf).parse_next(input)?;

    let mut dtstamp: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut contact: Option<Prop<String, Params>> = None;
    let mut dtstart: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut dtend: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut organizer: Option<Prop<Box<Uri>, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    // Multi-valued
    let mut attendee: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut freebusy: Vec<Prop<Vec<Period>, Params>> = Vec::new();
    let mut request_status: Vec<Prop<RequestStatus, Params>> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::DtStamp, PropValue::DateTimeUtc(p)) => {
                        once!(dtstamp, StaticProp::DtStamp, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::Contact, PropValue::Text(p)) => {
                        once!(contact, StaticProp::Contact, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::DtStart, PropValue::DateTimeOrDate(p)) => {
                        once!(dtstart, StaticProp::DtStart, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::DtEnd, PropValue::DateTimeOrDate(p)) => {
                        once!(dtend, StaticProp::DtEnd, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::Organizer, PropValue::Uri(p)) => {
                        once!(organizer, StaticProp::Organizer, ComponentKind::FreeBusy, p);
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::FreeBusy, p);
                    }
                    // Multi-valued
                    (StaticProp::Attendee, PropValue::Uri(p)) => { attendee.push(p); }
                    (StaticProp::Comment, PropValue::Text(p)) => { comment.push(p); }
                    (StaticProp::FreeBusy, PropValue::FreeBusyPeriods(p)) => { freebusy.push(p); }
                    (StaticProp::RequestStatus, PropValue::RequestStatus(p)) => { request_status.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse subcomponents
    let participants: Vec<Participant> = repeat(0.., preceded(peek(begin(Caseless("PARTICIPANT"))), participant)).parse_next(input)?;
    let locations: Vec<LocationComponent> = repeat(0.., preceded(peek(begin(Caseless("VLOCATION"))), location)).parse_next(input)?;
    let resource_components: Vec<ResourceComponent> = repeat(0.., preceded(peek(begin(Caseless("VRESOURCE"))), resource)).parse_next(input)?;

    terminated(end(Caseless("VFREEBUSY")), crlf).parse_next(input)?;

    let dtstamp = dtstamp.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::DtStamp),
            component: ComponentKind::FreeBusy,
        })
    })?;
    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::FreeBusy,
        })
    })?;

    let mut fb = FreeBusy::new(dtstamp, uid, participants, locations, resource_components);
    if let Some(v) = contact { fb.set_contact(v); }
    if let Some(v) = dtstart { fb.set_dtstart(v); }
    if let Some(v) = dtend { fb.set_dtend(v); }
    if let Some(v) = organizer { fb.set_organizer(v); }
    if let Some(v) = url { fb.set_url(v); }
    if !attendee.is_empty() { fb.set_attendee(attendee); }
    if !comment.is_empty() { fb.set_comment(comment); }
    if !freebusy.is_empty() { fb.set_freebusy(freebusy); }
    if !request_status.is_empty() { fb.set_request_status(request_status); }
    for (k, v) in x_props {
        fb.insert_x_property(k, v);
    }

    Ok(fb)
}

// ============================================================================
// TimeZone parser (RFC 5545 §3.6.5)
// ============================================================================

/// Parses a [`TimeZone`].
fn timezone<I, E>(input: &mut I) -> Result<TimeZone, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VTIMEZONE")), crlf).parse_next(input)?;

    let mut tz_id: Option<Prop<Box<TzId>, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut tz_url: Option<Prop<Box<Uri>, Params>> = None;
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::TzId, PropValue::TzId(p)) => {
                        once!(tz_id, StaticProp::TzId, ComponentKind::TimeZone, p);
                    }
                    (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                        once!(last_modified, StaticProp::LastModified, ComponentKind::TimeZone, p);
                    }
                    (StaticProp::TzUrl, PropValue::Uri(p)) => {
                        once!(tz_url, StaticProp::TzUrl, ComponentKind::TimeZone, p);
                    }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse STANDARD/DAYLIGHT subcomponents (at least one required)
    let rules: Vec<TzRule> = repeat(1.., tz_rule).parse_next(input)?;

    terminated(end(Caseless("VTIMEZONE")), crlf).parse_next(input)?;

    let tz_id = tz_id.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::TzId),
            component: ComponentKind::TimeZone,
        })
    })?;

    let mut tz = TimeZone::new(tz_id, rules);
    if let Some(v) = last_modified { tz.set_last_modified(v); }
    if let Some(v) = tz_url { tz.set_tz_url(v); }
    for (k, v) in x_props {
        tz.insert_x_property(k, v);
    }

    Ok(tz)
}

/// Parses a STANDARD or DAYLIGHT subcomponent of a VTIMEZONE.
fn tz_rule<I, E>(input: &mut I) -> Result<TzRule, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let kind: TzRuleKind = terminated(
        begin(alt((
            Caseless("STANDARD").value(TzRuleKind::Standard),
            Caseless("DAYLIGHT").value(TzRuleKind::Daylight),
        ))),
        crlf,
    )
    .parse_next(input)?;

    let mut dtstart: Option<Prop<DateTimeOrDate, Params>> = None;
    let mut tz_offset_to: Option<Prop<UtcOffset, Params>> = None;
    let mut tz_offset_from: Option<Prop<UtcOffset, Params>> = None;
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut rdate: Vec<Prop<RDateSeq, Params>> = Vec::new();
    let mut rrule: Vec<Prop<RRule, Params>> = Vec::new();
    let mut tz_name: Vec<Prop<String, Params>> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::DtStart, PropValue::DateTimeOrDate(p)) => {
                        once!(dtstart, StaticProp::DtStart, ComponentKind::StandardOrDaylight, p);
                    }
                    (StaticProp::TzOffsetTo, PropValue::UtcOffset(p)) => {
                        once!(tz_offset_to, StaticProp::TzOffsetTo, ComponentKind::StandardOrDaylight, p);
                    }
                    (StaticProp::TzOffsetFrom, PropValue::UtcOffset(p)) => {
                        once!(tz_offset_from, StaticProp::TzOffsetFrom, ComponentKind::StandardOrDaylight, p);
                    }
                    (StaticProp::Comment, PropValue::Text(p)) => { comment.push(p); }
                    (StaticProp::RDate, PropValue::RDateSeq(p)) => { rdate.push(p); }
                    (StaticProp::RRule, PropValue::RRule(p)) => { rrule.push(p); }
                    (StaticProp::TzName, PropValue::Text(p)) => { tz_name.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // End the rule with the matching END line
    let end_kind: TzRuleKind = terminated(
        end(alt((
            Caseless("STANDARD").value(TzRuleKind::Standard),
            Caseless("DAYLIGHT").value(TzRuleKind::Daylight),
        ))),
        crlf,
    )
    .parse_next(input)?;

    if end_kind != kind {
        return Err(E::from_external_error(
            input,
            CalendarParseError::MissingProp {
                prop: PropName::Known(StaticProp::DtStart),
                component: ComponentKind::StandardOrDaylight,
            },
        ));
    }

    let dtstart = dtstart.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::DtStart),
            component: ComponentKind::StandardOrDaylight,
        })
    })?;
    let tz_offset_to = tz_offset_to.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::TzOffsetTo),
            component: ComponentKind::StandardOrDaylight,
        })
    })?;
    let tz_offset_from = tz_offset_from.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::TzOffsetFrom),
            component: ComponentKind::StandardOrDaylight,
        })
    })?;

    let mut rule = TzRule::new(kind, dtstart, tz_offset_to, tz_offset_from);
    if !comment.is_empty() { rule.set_comment(comment); }
    if !rdate.is_empty() { rule.set_rdate(rdate); }
    if !rrule.is_empty() { rule.set_rrule(rrule); }
    if !tz_name.is_empty() { rule.set_tz_name(tz_name); }
    for (k, v) in x_props {
        rule.insert_x_property(k, v);
    }

    Ok(rule)
}

// ============================================================================
// Alarm parser (RFC 5545 §3.6.6)
// ============================================================================

fn alarm<I, E>(input: &mut I) -> Result<Alarm, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    use crate::model::primitive::AlarmAction;

    terminated(begin(Caseless("VALARM")), crlf).parse_next(input)?;

    // Shared alarm properties
    let mut action: Option<Prop<Token<AlarmAction, String>, Params>> = None;
    let mut trigger: Option<Prop<TriggerValue, Params>> = None;
    let mut duration: Option<Prop<SignedDuration, Params>> = None;
    let mut repeat: Option<Prop<Integer, Params>> = None;
    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut acknowledged: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut summary: Option<Prop<String, Params>> = None;
    // Multi-valued
    let mut attach: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut attendee: Vec<Prop<Box<Uri>, Params>> = Vec::new();
    let mut related_to: Vec<Prop<Box<Uid>, Params>> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();

    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::Action, PropValue::AlarmAction(p)) => {
                        once!(action, StaticProp::Action, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Trigger, PropValue::Trigger(p)) => {
                        once!(trigger, StaticProp::Trigger, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Duration, PropValue::Duration(p)) => {
                        once!(duration, StaticProp::Duration, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Repeat, PropValue::Integer(p)) => {
                        once!(repeat, StaticProp::Repeat, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Acknowledged, PropValue::DateTimeUtc(p)) => {
                        once!(acknowledged, StaticProp::Acknowledged, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Summary, PropValue::Text(p)) => {
                        once!(summary, StaticProp::Summary, ComponentKind::Alarm, p);
                    }
                    (StaticProp::Attach, PropValue::Attachment(p)) => { attach.push(p); }
                    (StaticProp::Attendee, PropValue::Uri(p)) => { attendee.push(p); }
                    (StaticProp::RelatedTo, PropValue::Uid(p)) => { related_to.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    terminated(end(Caseless("VALARM")), crlf).parse_next(input)?;

    // Check mandatory: trigger
    let trigger = trigger.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Trigger),
            component: ComponentKind::Alarm,
        })
    })?;

    // Check mandatory: action
    let action = action.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Action),
            component: ComponentKind::Alarm,
        })
    })?;

    // Dispatch based on action
    match action.value {
        Token::Known(AlarmAction::Audio) => {
            // Audio: at most one attachment
            if attach.len() > 1 {
                return Err(E::from_external_error(
                    input,
                    CalendarParseError::TooManyAttachmentsOnAudioAlarm,
                ));
            }
            let mut a = AudioAlarm::new(trigger);
            if let Some(att) = attach.into_iter().next() { a.set_attach(att); }
            if let Some(v) = uid { a.set_uid(v); }
            if let Some(v) = duration { a.set_duration(v); }
            if let Some(v) = repeat { a.set_repeat(v); }
            if let Some(v) = acknowledged { a.set_acknowledged(v); }
            for (k, v) in x_props { a.insert_x_property(k, v); }
            Ok(Alarm::Audio(a))
        }
        Token::Known(AlarmAction::Display) => {
            let description = description.ok_or_else(|| {
                E::from_external_error(input, CalendarParseError::MissingProp {
                    prop: PropName::Known(StaticProp::Description),
                    component: ComponentKind::DisplayAlarm,
                })
            })?;
            let mut a = DisplayAlarm::new(trigger, description);
            if let Some(v) = uid { a.set_uid(v); }
            if let Some(v) = duration { a.set_duration(v); }
            if let Some(v) = repeat { a.set_repeat(v); }
            if let Some(v) = acknowledged { a.set_acknowledged(v); }
            for (k, v) in x_props { a.insert_x_property(k, v); }
            Ok(Alarm::Display(a))
        }
        Token::Known(AlarmAction::Email) => {
            let description = description.ok_or_else(|| {
                E::from_external_error(input, CalendarParseError::MissingProp {
                    prop: PropName::Known(StaticProp::Description),
                    component: ComponentKind::EmailAlarm,
                })
            })?;
            let summary = summary.ok_or_else(|| {
                E::from_external_error(input, CalendarParseError::MissingProp {
                    prop: PropName::Known(StaticProp::Summary),
                    component: ComponentKind::EmailAlarm,
                })
            })?;
            let mut a = EmailAlarm::new(trigger, description, summary);
            if let Some(v) = uid { a.set_uid(v); }
            if let Some(v) = duration { a.set_duration(v); }
            if let Some(v) = repeat { a.set_repeat(v); }
            if let Some(v) = acknowledged { a.set_acknowledged(v); }
            if !attendee.is_empty() { a.set_attendee(attendee); }
            if !attach.is_empty() { a.set_attach(attach); }
            for (k, v) in x_props { a.insert_x_property(k, v); }
            Ok(Alarm::Email(a))
        }
        Token::Known(_) | Token::Unknown(_) => {
            // Other/unknown action
            let action_str = match action.value {
                Token::Unknown(s) => s,
                _ => String::new(),
            };
            let action_prop = Prop { value: action_str, params: action.params };
            let mut a = OtherAlarm::new(trigger, action_prop);
            if let Some(v) = description { a.set_description(v); }
            if let Some(v) = summary { a.set_summary(v); }
            if let Some(v) = uid { a.set_uid(v); }
            if let Some(v) = duration { a.set_duration(v); }
            if let Some(v) = repeat { a.set_repeat(v); }
            if let Some(v) = acknowledged { a.set_acknowledged(v); }
            if !attendee.is_empty() { a.set_attendee(attendee); }
            if !attach.is_empty() { a.set_attach(attach); }
            for (k, v) in x_props { a.insert_x_property(k, v); }
            Ok(Alarm::Other(a))
        }
    }
}

// ============================================================================
// Participant parser (RFC 9073 §7.1)
// ============================================================================

fn participant<I, E>(input: &mut I) -> Result<Participant, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("PARTICIPANT")), crlf).parse_next(input)?;

    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut participant_type: Option<Prop<Token<ParticipantType, String>, Params>> = None;
    let mut calendar_address: Option<Prop<Box<Uri>, Params>> = None;
    let mut created: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut dtstamp: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut geo: Option<Prop<Geo, Params>> = None;
    let mut last_modified: Option<Prop<DateTime<Utc>, Params>> = None;
    let mut priority: Option<Prop<Priority, Params>> = None;
    let mut sequence: Option<Prop<Integer, Params>> = None;
    let mut status: Option<Prop<Status, Params>> = None;
    let mut summary: Option<Prop<String, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    // Multi-valued
    let mut attach: Vec<Prop<Attachment, Params>> = Vec::new();
    let mut categories: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut comment: Vec<Prop<String, Params>> = Vec::new();
    let mut contact: Vec<Prop<String, Params>> = Vec::new();
    let mut location_prop: Vec<Prop<String, Params>> = Vec::new();
    let mut request_status: Vec<Prop<RequestStatus, Params>> = Vec::new();
    let mut related_to: Vec<Prop<Box<Uid>, Params>> = Vec::new();
    let mut resources: Vec<Prop<Vec<String>, Params>> = Vec::new();
    let mut styled_description: Vec<Prop<StyledDescriptionValue, Params>> = Vec::new();
    let mut structured_data: Vec<StructuredDataProp> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();


    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Unknown, p);
                    }
                    (StaticProp::ParticipantType, PropValue::ParticipantType(p)) => {
                        once!(participant_type, StaticProp::ParticipantType, ComponentKind::Unknown, p);
                    }
                    (StaticProp::CalendarAddress, PropValue::Uri(p)) => {
                        once!(calendar_address, StaticProp::CalendarAddress, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Created, PropValue::DateTimeUtc(p)) => {
                        once!(created, StaticProp::Created, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Unknown, p);
                    }
                    (StaticProp::DtStamp, PropValue::DateTimeUtc(p)) => {
                        once!(dtstamp, StaticProp::DtStamp, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Geo, PropValue::Geo(p)) => {
                        once!(geo, StaticProp::Geo, ComponentKind::Unknown, p);
                    }
                    (StaticProp::LastModified, PropValue::DateTimeUtc(p)) => {
                        once!(last_modified, StaticProp::LastModified, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Priority, PropValue::Priority(p)) => {
                        once!(priority, StaticProp::Priority, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Sequence, PropValue::Integer(p)) => {
                        once!(sequence, StaticProp::Sequence, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Status, PropValue::Status(p)) => {
                        once!(status, StaticProp::Status, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Summary, PropValue::Text(p)) => {
                        once!(summary, StaticProp::Summary, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::Unknown, p);
                    }
                    // Multi-valued
                    (StaticProp::Attach, PropValue::Attachment(p)) => { attach.push(p); }
                    (StaticProp::Categories, PropValue::TextSeq(p)) => { categories.push(p); }
                    (StaticProp::Comment, PropValue::Text(p)) => { comment.push(p); }
                    (StaticProp::Contact, PropValue::Text(p)) => { contact.push(p); }
                    (StaticProp::Location, PropValue::Text(p)) => { location_prop.push(p); }
                    (StaticProp::RequestStatus, PropValue::RequestStatus(p)) => { request_status.push(p); }
                    (StaticProp::RelatedTo, PropValue::Uid(p)) => { related_to.push(p); }
                    (StaticProp::Resources, PropValue::TextSeq(p)) => { resources.push(p); }
                    (StaticProp::StyledDescription, PropValue::StyledDescription(p)) => { styled_description.push(p); }
                    (StaticProp::StructuredData, PropValue::StructuredData(p)) => { structured_data.push(p); }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    // Parse subcomponents
    let locations: Vec<LocationComponent> = repeat(0.., preceded(peek(begin(Caseless("VLOCATION"))), location)).parse_next(input)?;
    let resource_components: Vec<ResourceComponent> = repeat(0.., preceded(peek(begin(Caseless("VRESOURCE"))), resource)).parse_next(input)?;

    terminated(end(Caseless("PARTICIPANT")), crlf).parse_next(input)?;

    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Unknown,
        })
    })?;
    let participant_type = participant_type.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::ParticipantType),
            component: ComponentKind::Unknown,
        })
    })?;

    let mut p = Participant::new(uid, participant_type, locations, resource_components);
    if let Some(v) = calendar_address { p.set_calendar_address(v); }
    if let Some(v) = created { p.set_created(v); }
    if let Some(v) = description { p.set_description(v); }
    if let Some(v) = dtstamp { p.set_dtstamp(v); }
    if let Some(v) = geo { p.set_geo(v); }
    if let Some(v) = last_modified { p.set_last_modified(v); }
    if let Some(v) = priority { p.set_priority(v); }
    if let Some(v) = sequence { p.set_sequence(v); }
    if let Some(v) = status { p.set_status(v); }
    if let Some(v) = summary { p.set_summary(v); }
    if let Some(v) = url { p.set_url(v); }
    if !attach.is_empty() { p.set_attach(attach); }
    if !categories.is_empty() { p.set_categories(categories); }
    if !comment.is_empty() { p.set_comment(comment); }
    if !contact.is_empty() { p.set_contact(contact); }
    if !location_prop.is_empty() { p.set_location_prop(location_prop); }
    if !request_status.is_empty() { p.set_request_status(request_status); }
    if !related_to.is_empty() { p.set_related_to(related_to); }
    if !resources.is_empty() { p.set_resources(resources); }
    if !styled_description.is_empty() { p.set_styled_description(styled_description); }
    if !structured_data.is_empty() { p.set_structured_data(structured_data); }
    for (k, v) in x_props {
        p.insert_x_property(k, v);
    }

    Ok(p)
}

// ============================================================================
// Location parser (RFC 9073 §7.2)
// ============================================================================

fn location<I, E>(input: &mut I) -> Result<LocationComponent, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VLOCATION")), crlf).parse_next(input)?;

    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut geo: Option<Prop<Geo, Params>> = None;
    let mut name: Option<Prop<String, Params>> = None;
    let mut location_type: Option<Prop<String, Params>> = None;
    let mut url: Option<Prop<Box<Uri>, Params>> = None;
    let mut structured_data: Vec<StructuredDataProp> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();


    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Geo, PropValue::Geo(p)) => {
                        once!(geo, StaticProp::Geo, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Name, PropValue::Text(p)) => {
                        once!(name, StaticProp::Name, ComponentKind::Unknown, p);
                    }
                    (StaticProp::LocationType, PropValue::TextSeq(p)) => {
                        // LocationType is parsed as TextSeq but model stores as String
                        // Join back into a single comma-separated string
                        let joined = p.value.join(",");
                        once!(location_type, StaticProp::LocationType, ComponentKind::Unknown, Prop { value: joined, params: p.params });
                    }
                    (StaticProp::Url, PropValue::Uri(p)) => {
                        once!(url, StaticProp::Url, ComponentKind::Unknown, p);
                    }
                    (StaticProp::StructuredData, PropValue::StructuredData(p)) => {
                        structured_data.push(p);
                    }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    terminated(end(Caseless("VLOCATION")), crlf).parse_next(input)?;

    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Unknown,
        })
    })?;

    let mut loc = LocationComponent::new(uid);
    if let Some(v) = description { loc.set_description(v); }
    if let Some(v) = geo { loc.set_geo(v); }
    if let Some(v) = name { loc.set_name(v); }
    if let Some(v) = location_type { loc.set_location_type(v); }
    if let Some(v) = url { loc.set_url(v); }
    if !structured_data.is_empty() { loc.set_structured_data(structured_data); }
    for (k, v) in x_props {
        loc.insert_x_property(k, v);
    }

    Ok(loc)
}

// ============================================================================
// Resource parser (RFC 9073 §7.3)
// ============================================================================

/// Parses a [`ResourceComponent`].
fn resource<I, E>(input: &mut I) -> Result<ResourceComponent, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    terminated(begin(Caseless("VRESOURCE")), crlf).parse_next(input)?;

    let mut uid: Option<Prop<Box<Uid>, Params>> = None;
    let mut description: Option<Prop<String, Params>> = None;
    let mut geo: Option<Prop<Geo, Params>> = None;
    let mut name: Option<Prop<String, Params>> = None;
    let mut resource_type: Option<Prop<Token<ResourceType, String>, Params>> = None;
    let mut structured_data: Vec<StructuredDataProp> = Vec::new();
    let mut x_props: HashMap<Box<CaselessStr>, Vec<Prop<Value<String>, Params>>> = HashMap::new();


    parse_props!(input, parsed, {
        match parsed {
            ParsedProp::Known(KnownProp { name: prop_name, value }) => {
                match (prop_name, value) {
                    (StaticProp::Uid, PropValue::Uid(p)) => {
                        once!(uid, StaticProp::Uid, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Description, PropValue::Text(p)) => {
                        once!(description, StaticProp::Description, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Geo, PropValue::Geo(p)) => {
                        once!(geo, StaticProp::Geo, ComponentKind::Unknown, p);
                    }
                    (StaticProp::Name, PropValue::Text(p)) => {
                        once!(name, StaticProp::Name, ComponentKind::Unknown, p);
                    }
                    (StaticProp::ResourceType, PropValue::ResourceType(p)) => {
                        once!(resource_type, StaticProp::ResourceType, ComponentKind::Unknown, p);
                    }
                    (StaticProp::StructuredData, PropValue::StructuredData(p)) => {
                        structured_data.push(p);
                    }
                    _ => { /* ignore - property parser guarantees correct variant */ }
                }
            }
            ParsedProp::Unknown(UnknownProp { name: uname, params, value, .. }) => {
                let name_string: String = I::try_into_string(&uname)?;
                handle_unknown!(x_props, name_string, params, value);
            }
        }
        Ok(())
    });

    terminated(end(Caseless("VRESOURCE")), crlf).parse_next(input)?;

    let uid = uid.ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::MissingProp {
            prop: PropName::Known(StaticProp::Uid),
            component: ComponentKind::Unknown,
        })
    })?;

    let mut res = ResourceComponent::new(uid);
    if let Some(v) = description { res.set_description(v); }
    if let Some(v) = geo { res.set_geo(v); }
    if let Some(v) = name { res.set_name(v); }
    if let Some(v) = resource_type { res.set_resource_type(v); }
    if !structured_data.is_empty() { res.set_structured_data(structured_data); }
    for (k, v) in x_props {
        res.insert_x_property(k, v);
    }

    Ok(res)
}

// ============================================================================
// OtherComponent parser
// ============================================================================

/// Parses an arbitrary component with BEGIN and END lines.
fn other_with_name<I, E>(input: &mut I) -> Result<OtherComponent, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    I::Slice: AsBStr + Clone + PartialEq + Eq + SliceLen + Stream + AsRef<[u8]> + Hash,
    <<I as Stream>::Slice as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn is_name_char<T: AsChar>(c: T) -> bool {
        let c = c.as_char();
        c.is_ascii_alphanumeric() || c == '-'
    }

    // Parse BEGIN:<name>
    let name_slice = terminated(
        begin(winnow::token::take_while(1.., is_name_char)),
        crlf,
    )
    .parse_next(input)?;

    let name_string = I::try_into_string(&name_slice)
        .map_err(|e| E::from_external_error(input, e))?;

    // Skip all properties (just consume them)
    loop {
        let checkpoint = input.checkpoint();
        if alt((begin(empty::<I, E>), end(empty::<I, E>))).parse_next(input).is_ok() {
            input.reset(&checkpoint);
            break;
        }
        input.reset(&checkpoint);

        // Consume the property line (we don't need to interpret it)
        let _: ParsedProp<I::Slice> = terminated(property, crlf).parse_next(input)?;
    }

    // Parse nested subcomponents recursively
    let subcomponents: Vec<OtherComponent> = repeat(0.., other_with_name).parse_next(input)?;

    // Parse END:<name>
    let end_name_slice = terminated(
        end(winnow::token::take_while(1.., is_name_char)),
        crlf,
    )
    .parse_next(input)?;

    let end_name_string = I::try_into_string(&end_name_slice)
        .map_err(|e| E::from_external_error(input, e))?;

    // Verify BEGIN and END names match (case-insensitive)
    if !name_string.eq_ignore_ascii_case(&end_name_string) {
        return Err(E::from_external_error(
            input,
            CalendarParseError::MissingProp {
                prop: PropName::Known(StaticProp::Uid),
                component: ComponentKind::Unknown,
            },
        ));
    }

    Ok(OtherComponent {
        name: name_string.into_boxed_str(),
        subcomponents,
    })
}

// ============================================================================
// Helpers
// ============================================================================

/// Parses the `BEGIN:<name>` sequence at the start of a component.
pub fn begin<I, O, E>(name: impl Parser<I, O, E>) -> impl Parser<I, O, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    preceded(Caseless("BEGIN:"), name)
}

/// Parses the `END:<name>` sequence at the end of a component.
pub fn end<I, O, E>(name: impl Parser<I, O, E>) -> impl Parser<I, O, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    preceded(Caseless("END:"), name)
}

/// A version of [`winnow::ascii::crlf`] bounded by `Compare<char>` instead
/// of `Compare<&'static str>`.
pub fn crlf<I, E>(input: &mut I) -> Result<I::Slice, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    E: ParserError<I>,
{
    ('\r', '\n').take().parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{date, time, utc_offset};
    use crate::model::primitive::{
        ClassValue, DateTimeOrDate, Sign, TimeFormat, Token, TriggerValue, Version,
    };
    use crate::parser::escaped::AsEscaped;
    use calendar_types::duration::Duration;

    macro_rules! concat_crlf {
        ($($l:literal),* $(,)?) => {
            concat!(
                $($l, "\r\n",)*
            )
        };
    }

    // ======================================================================
    // 1. begin parser
    // ======================================================================

    #[test]
    fn begin_parser() {
        let input = "BEGIN:VTODO\r\n";
        let result: Result<(&str, &str), ()> =
            begin(Caseless("VTODO").take()).parse_peek(input);
        assert!(result.is_ok());
        let (remaining, matched) = result.unwrap();
        assert_eq!(matched, "VTODO");
        assert_eq!(remaining, "\r\n");
    }

    // ======================================================================
    // 2. end parser
    // ======================================================================

    #[test]
    fn end_parser() {
        let input = "END:VALARM\r\n";
        let result: Result<(&str, &str), ()> =
            end(Caseless("VALARM").take()).parse_peek(input);
        assert!(result.is_ok());
        let (remaining, matched) = result.unwrap();
        assert_eq!(matched, "VALARM");
        assert_eq!(remaining, "\r\n");
    }

    // ======================================================================
    // 3. parse_minimal_event
    // ======================================================================

    #[test]
    fn parse_minimal_event() {
        let input = concat_crlf!(
            "BEGIN:VEVENT",
            "DTSTAMP:19970901T130000Z",
            "UID:uid1@example.com",
            "END:VEVENT",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty(), "remaining input: {:?}", std::str::from_utf8(remaining.0));

        match comp {
            CalendarComponent::Event(ev) => {
                assert_eq!(
                    ev.dtstamp().value,
                    DateTime {
                        date: date!(1997;9;1),
                        time: time!(13;0;0),
                        marker: Utc,
                    }
                );
                assert_eq!(ev.uid().value.as_str(), "uid1@example.com");
                assert!(ev.dtstart().is_none());
                assert!(ev.summary().is_none());
            }
            other => panic!("expected Event, got {:?}", other),
        }
    }

    // ======================================================================
    // 4. parse_event_with_properties
    // ======================================================================

    #[test]
    fn parse_event_with_properties() {
        let input = concat_crlf!(
            "BEGIN:VEVENT",
            "DTSTAMP:19970901T130000Z",
            "UID:uid2@example.com",
            "DTSTART:19970903T163000Z",
            "DTEND:19970903T190000Z",
            "SUMMARY:Annual Employee Review",
            "CLASS:PRIVATE",
            "END:VEVENT",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (_, comp) = result.unwrap();

        match comp {
            CalendarComponent::Event(ev) => {
                // Check uid
                assert_eq!(ev.uid().value.as_str(), "uid2@example.com");

                // Check dtstart
                let dtstart = ev.dtstart().expect("dtstart should be present");
                assert_eq!(
                    dtstart.value,
                    DateTimeOrDate::DateTime(DateTime {
                        date: date!(1997;9;3),
                        time: time!(16;30;0),
                        marker: TimeFormat::Utc,
                    })
                );

                // Check dtend
                let dtend = ev.dtend().expect("dtend should be present");
                assert_eq!(
                    dtend.value,
                    DateTimeOrDate::DateTime(DateTime {
                        date: date!(1997;9;3),
                        time: time!(19;0;0),
                        marker: TimeFormat::Utc,
                    })
                );

                // Check summary
                let summary = ev.summary().expect("summary should be present");
                assert_eq!(summary.value, "Annual Employee Review");

                // Check class
                let class = ev.class().expect("class should be present");
                assert_eq!(class.value, Token::Known(ClassValue::Private));
            }
            other => panic!("expected Event, got {:?}", other),
        }
    }

    // ======================================================================
    // 5. parse_minimal_todo
    // ======================================================================

    #[test]
    fn parse_minimal_todo() {
        let input = concat_crlf!(
            "BEGIN:VTODO",
            "DTSTAMP:19980130T134500Z",
            "UID:todo1@example.com",
            "END:VTODO",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty());

        match comp {
            CalendarComponent::Todo(td) => {
                assert_eq!(
                    td.dtstamp().value,
                    DateTime {
                        date: date!(1998;1;30),
                        time: time!(13;45;0),
                        marker: Utc,
                    }
                );
                assert_eq!(td.uid().value.as_str(), "todo1@example.com");
            }
            other => panic!("expected Todo, got {:?}", other),
        }
    }

    // ======================================================================
    // 6. parse_minimal_journal
    // ======================================================================

    #[test]
    fn parse_minimal_journal() {
        let input = concat_crlf!(
            "BEGIN:VJOURNAL",
            "DTSTAMP:19970901T130000Z",
            "UID:journal1@example.com",
            "END:VJOURNAL",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty());

        match comp {
            CalendarComponent::Journal(jn) => {
                assert_eq!(
                    jn.dtstamp().value,
                    DateTime {
                        date: date!(1997;9;1),
                        time: time!(13;0;0),
                        marker: Utc,
                    }
                );
                assert_eq!(jn.uid().value.as_str(), "journal1@example.com");
            }
            other => panic!("expected Journal, got {:?}", other),
        }
    }

    // ======================================================================
    // 7. parse_minimal_freebusy
    // ======================================================================

    #[test]
    fn parse_minimal_freebusy() {
        let input = concat_crlf!(
            "BEGIN:VFREEBUSY",
            "DTSTAMP:19970901T120000Z",
            "UID:fb1@example.com",
            "END:VFREEBUSY",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty());

        match comp {
            CalendarComponent::FreeBusy(fb) => {
                assert_eq!(fb.uid().value.as_str(), "fb1@example.com");
                assert_eq!(
                    fb.dtstamp().value,
                    DateTime {
                        date: date!(1997;9;1),
                        time: time!(12;0;0),
                        marker: Utc,
                    }
                );
            }
            other => panic!("expected FreeBusy, got {:?}", other),
        }
    }

    // ======================================================================
    // 8. parse_timezone
    // ======================================================================

    #[test]
    fn parse_timezone() {
        let input = concat_crlf!(
            "BEGIN:VTIMEZONE",
            "TZID:America/New_York",
            "BEGIN:STANDARD",
            "DTSTART:19971026T020000",
            "TZOFFSETFROM:-0400",
            "TZOFFSETTO:-0500",
            "TZNAME:EST",
            "END:STANDARD",
            "BEGIN:DAYLIGHT",
            "DTSTART:19980301T020000",
            "TZOFFSETFROM:-0500",
            "TZOFFSETTO:-0400",
            "TZNAME:EDT",
            "END:DAYLIGHT",
            "END:VTIMEZONE",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty());

        match comp {
            CalendarComponent::TimeZone(tz) => {
                assert_eq!(tz.tz_id().value.as_str(), "America/New_York");
                assert_eq!(tz.rules().len(), 2);

                // First rule: STANDARD
                let standard = &tz.rules()[0];
                assert_eq!(standard.kind(), &TzRuleKind::Standard);
                assert_eq!(
                    standard.tz_offset_from().value,
                    utc_offset!(-4;0)
                );
                assert_eq!(
                    standard.tz_offset_to().value,
                    utc_offset!(-5;0)
                );
                let tz_names = standard.tz_name().expect("tz_name should be present");
                assert_eq!(tz_names[0].value, "EST");

                // Second rule: DAYLIGHT
                let daylight = &tz.rules()[1];
                assert_eq!(daylight.kind(), &TzRuleKind::Daylight);
                assert_eq!(
                    daylight.tz_offset_from().value,
                    utc_offset!(-5;0)
                );
                assert_eq!(
                    daylight.tz_offset_to().value,
                    utc_offset!(-4;0)
                );
                let tz_names = daylight.tz_name().expect("tz_name should be present");
                assert_eq!(tz_names[0].value, "EDT");
            }
            other => panic!("expected TimeZone, got {:?}", other),
        }
    }

    // ======================================================================
    // 9. parse_display_alarm
    // ======================================================================

    #[test]
    fn parse_display_alarm() {
        // An alarm as a subcomponent of an event
        let input = concat_crlf!(
            "BEGIN:VEVENT",
            "DTSTAMP:19970901T130000Z",
            "UID:alarm-test@example.com",
            "BEGIN:VALARM",
            "ACTION:DISPLAY",
            "DESCRIPTION:Breakfast meeting",
            "TRIGGER:-PT15M",
            "END:VALARM",
            "END:VEVENT",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (_, comp) = result.unwrap();

        match comp {
            CalendarComponent::Event(ev) => {
                assert_eq!(ev.alarms().len(), 1);
                match &ev.alarms()[0] {
                    Alarm::Display(da) => {
                        assert_eq!(da.description().value, "Breakfast meeting");
                        // TRIGGER:-PT15M is a negative duration of 15 minutes
                        match &da.trigger().value {
                            TriggerValue::Duration(sd) => {
                                assert_eq!(sd.sign, Sign::Neg);
                                match sd.duration {
                                    Duration::Exact(exact) => {
                                        assert_eq!(exact.minutes, 15);
                                        assert_eq!(exact.hours, 0);
                                        assert_eq!(exact.seconds, 0);
                                    }
                                    other => panic!("expected Exact duration, got {:?}", other),
                                }
                            }
                            other => panic!("expected Duration trigger, got {:?}", other),
                        }
                    }
                    other => panic!("expected Display alarm, got {:?}", other),
                }
            }
            other => panic!("expected Event, got {:?}", other),
        }
    }

    // ======================================================================
    // 10. parse_full_calendar
    // ======================================================================

    #[test]
    fn parse_full_calendar() {
        let input = concat_crlf!(
            "BEGIN:VCALENDAR",
            "PRODID:-//Test//Test//EN",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "DTSTAMP:19970901T130000Z",
            "UID:cal-event1@example.com",
            "SUMMARY:Bastille Day Party",
            "END:VEVENT",
            "END:VCALENDAR",
        );

        let result = calendar::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, cal) = result.unwrap();
        assert!(remaining.is_empty());

        assert_eq!(cal.prod_id().unwrap().value, "-//Test//Test//EN");
        assert_eq!(cal.version().value, Version::V2_0);
        assert_eq!(cal.components().len(), 1);

        match &cal.components()[0] {
            CalendarComponent::Event(ev) => {
                assert_eq!(ev.uid().value.as_str(), "cal-event1@example.com");
                let summary = ev.summary().expect("summary should be present");
                assert_eq!(summary.value, "Bastille Day Party");
            }
            other => panic!("expected Event component, got {:?}", other),
        }
    }

    // ======================================================================
    // 11. parse_other_component
    // ======================================================================

    #[test]
    fn parse_other_component() {
        let input = concat_crlf!(
            "BEGIN:X-CUSTOM",
            "X-FOO:bar",
            "END:X-CUSTOM",
        );

        let result = calendar_component::<_, ()>
            .parse_peek(input.as_escaped());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let (remaining, comp) = result.unwrap();
        assert!(remaining.is_empty());

        match comp {
            CalendarComponent::Other(other) => {
                assert_eq!(&*other.name, "X-CUSTOM");
                assert!(other.subcomponents.is_empty());
            }
            other => panic!("expected Other component, got {:?}", other),
        }
    }
}
