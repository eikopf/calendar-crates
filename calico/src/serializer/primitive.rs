//! `WriteIcal` implementations for primitive types.

use std::fmt;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use calendar_types::{
    duration::{Duration, SignedDuration},
    primitive::Sign,
    time::{Date, DateTime, Local, Time, TimeFormat, Utc},
};
use rfc5545_types::{
    request_status::RequestStatus,
    rrule::{
        ByMonthDayRule, ByPeriodDayRules, CoreByRules, Freq, FreqByRules, HourSet, MinuteSet,
        MonthDaySet, MonthDaySetIndex, MonthDay, MonthSet, RRule, SecondSet, WeekNoSet,
        WeekNoSetIndex, WeekdayNum, YearlyByRules,
    },
    set::{Percent, Priority, ThisAndFuture, Version},
    time::{DateTimeOrDate, ExDateSeq, Period, RDateSeq, TriggerValue, UtcOffset},
    value::{Attachment, Geo, StyledDescriptionValue},
};

use super::WriteIcal;
use crate::model::{
    primitive::Value,
    string::{TzId, Uid, Uri},
};

// ============================================================================
// Date / Time / DateTime
// ============================================================================

impl WriteIcal for Date {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{:04}{:02}{:02}", self.year().get(), self.month() as u8, self.day() as u8)
    }
}

impl WriteIcal for Time {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "{:02}{:02}{:02}",
            self.hour() as u8,
            self.minute() as u8,
            self.second() as u8,
        )?;
        if let Some(frac) = self.frac() {
            let nanos = frac.get().get();
            let mut s = format!("{nanos:09}");
            let trimmed = s.trim_end_matches('0');
            s.truncate(trimmed.len());
            write!(w, ".{s}")?;
        }
        Ok(())
    }
}

impl WriteIcal for DateTime<Utc> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        self.date.write_ical(w)?;
        w.write_char('T')?;
        self.time.write_ical(w)?;
        w.write_char('Z')
    }
}

impl WriteIcal for DateTime<Local> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        self.date.write_ical(w)?;
        w.write_char('T')?;
        self.time.write_ical(w)
    }
}

impl WriteIcal for DateTime<TimeFormat> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        self.date.write_ical(w)?;
        w.write_char('T')?;
        self.time.write_ical(w)?;
        match self.marker {
            TimeFormat::Utc => w.write_char('Z'),
            TimeFormat::Local => Ok(()),
        }
    }
}

// ============================================================================
// UtcOffset
// ============================================================================

impl WriteIcal for UtcOffset {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "{}{:02}{:02}",
            self.sign.as_char(),
            self.hour as u8,
            self.minute as u8,
        )?;
        let sec = self.second as u8;
        if sec != 0 {
            write!(w, "{sec:02}")?;
        }
        Ok(())
    }
}

// ============================================================================
// DateTimeOrDate
// ============================================================================

impl WriteIcal for DateTimeOrDate<Utc> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            DateTimeOrDate::DateTime(dt) => dt.write_ical(w),
            DateTimeOrDate::Date(d) => d.write_ical(w),
        }
    }
}

impl WriteIcal for DateTimeOrDate<TimeFormat> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            DateTimeOrDate::DateTime(dt) => dt.write_ical(w),
            DateTimeOrDate::Date(d) => d.write_ical(w),
        }
    }
}

// ============================================================================
// Period
// ============================================================================

impl WriteIcal for Period<TimeFormat> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            Period::Explicit { start, end } => {
                start.write_ical(w)?;
                w.write_char('/')?;
                end.write_ical(w)
            }
            Period::Start { start, duration } => {
                start.write_ical(w)?;
                w.write_char('/')?;
                duration.write_ical(w)
            }
        }
    }
}

// ============================================================================
// Duration (delegates to Display — ISO 8601 format is correct for iCalendar)
// ============================================================================

impl WriteIcal for Duration {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

impl WriteIcal for SignedDuration {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

// ============================================================================
// Geo
// ============================================================================

impl WriteIcal for Geo {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{};{}", self.lat, self.lon)
    }
}

// ============================================================================
// RDateSeq / ExDateSeq
// ============================================================================

impl WriteIcal for RDateSeq {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            RDateSeq::DateTime(dts) => write_comma_separated(dts, w),
            RDateSeq::Date(ds) => write_comma_separated(ds, w),
            RDateSeq::Period(ps) => write_comma_separated(ps, w),
        }
    }
}

impl WriteIcal for ExDateSeq {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            ExDateSeq::DateTime(dts) => write_comma_separated(dts, w),
            ExDateSeq::Date(ds) => write_comma_separated(ds, w),
        }
    }
}

// ============================================================================
// TriggerValue
// ============================================================================

impl WriteIcal for TriggerValue {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            TriggerValue::Duration(d) => d.write_ical(w),
            TriggerValue::DateTime(dt) => dt.write_ical(w),
        }
    }
}

// ============================================================================
// RequestStatus (Display format is correct for iCalendar)
// ============================================================================

impl WriteIcal for RequestStatus {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        // RequestStatus Display writes: code;description[;exception_data]
        // but description/exception_data TEXT values need escaping
        write!(w, "{}", self.code)?;
        w.write_char(';')?;
        super::escape_text(&self.description, w)?;
        if let Some(data) = &self.exception_data {
            w.write_char(';')?;
            super::escape_text(data, w)?;
        }
        Ok(())
    }
}

// ============================================================================
// Attachment
// ============================================================================

impl WriteIcal for Attachment {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            Attachment::Uri(uri) => w.write_str(uri.as_str()),
            Attachment::Binary(data) => {
                let encoded = BASE64.encode(data);
                w.write_str(&encoded)
            }
        }
    }
}

// ============================================================================
// StyledDescriptionValue
// ============================================================================

impl WriteIcal for StyledDescriptionValue {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            StyledDescriptionValue::Text(s) => super::escape_text(s, w),
            StyledDescriptionValue::Uri(uri) => w.write_str(uri.as_str()),
            StyledDescriptionValue::Iana { value, .. } => super::escape_text(value, w),
        }
    }
}

// ============================================================================
// Value<S>
// ============================================================================

impl<S: AsRef<str>> WriteIcal for Value<S> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            Value::Binary(data) => {
                let encoded = BASE64.encode(data);
                w.write_str(&encoded)
            }
            Value::Boolean(b) => w.write_str(if *b { "TRUE" } else { "FALSE" }),
            Value::CalAddress(uri) => w.write_str(uri.as_str()),
            Value::Date(d) => d.write_ical(w),
            Value::DateTime(dt) => dt.write_ical(w),
            Value::Duration(d) => d.write_ical(w),
            Value::Float(f) => write!(w, "{f}"),
            Value::Integer(i) => write!(w, "{i}"),
            Value::Period(p) => p.write_ical(w),
            Value::Recur(r) => r.write_ical(w),
            Value::Text(s) => super::escape_text(s.as_ref(), w),
            Value::Time(t, tf) => {
                t.write_ical(w)?;
                match tf {
                    TimeFormat::Utc => w.write_char('Z'),
                    TimeFormat::Local => Ok(()),
                }
            }
            Value::Uri(uri) => w.write_str(uri.as_str()),
            Value::UtcOffset(o) => o.write_ical(w),
            Value::Other { value, .. } => super::escape_text(value.as_ref(), w),
        }
    }
}

// ============================================================================
// Display-delegating types
// ============================================================================

/// Implements `WriteIcal` by delegating to `Display`.
macro_rules! impl_write_ical_via_display {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl WriteIcal for $ty {
                fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
                    write!(w, "{self}")
                }
            }
        )+
    };
}

impl_write_ical_via_display!(
    rfc5545_types::set::Method,
    rfc5545_types::set::Encoding,
    rfc5545_types::set::TimeTransparency,
    rfc5545_types::set::EventStatus,
    rfc5545_types::set::TodoStatus,
    rfc5545_types::set::JournalStatus,
    rfc5545_types::set::ClassValue,
    rfc5545_types::set::CalendarUserType,
    rfc5545_types::set::ParticipationRole,
    rfc5545_types::set::ParticipationStatus,
    rfc5545_types::set::FreeBusyType,
    rfc5545_types::set::RelationshipType,
    rfc5545_types::set::AlarmAction,
    rfc5545_types::set::TriggerRelation,
    rfc5545_types::set::ValueType,
    rfc5545_types::set::DisplayType,
    rfc5545_types::set::FeatureType,
    rfc5545_types::set::ResourceType,
    rfc5545_types::set::ParticipantType,
    rfc5545_types::set::ProximityValue,
    rfc5545_types::set::Status,
    rfc5545_types::value::FormatType,
    rfc5545_types::value::FormatTypeBuf,
    calendar_types::css::Css3Color,
);

// ============================================================================
// Token<T, S>
// ============================================================================

impl<T: WriteIcal, S: fmt::Display> WriteIcal for calendar_types::set::Token<T, S> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            calendar_types::set::Token::Known(t) => t.write_ical(w),
            calendar_types::set::Token::Unknown(s) => write!(w, "{s}"),
        }
    }
}

// ============================================================================
// Simple wrapper types (delegate to inner)
// ============================================================================

impl WriteIcal for Version {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        match self {
            Version::V2_0 => w.write_str("2.0"),
        }
    }
}

impl WriteIcal for rfc5545_types::set::Gregorian {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str("GREGORIAN")
    }
}

impl WriteIcal for ThisAndFuture {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str("THISANDFUTURE")
    }
}

impl WriteIcal for Priority {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let n = *self as u8;
        write!(w, "{n}")
    }
}

impl WriteIcal for Percent {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{}", self.get())
    }
}

impl WriteIcal for bool {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(if *self { "TRUE" } else { "FALSE" })
    }
}

impl WriteIcal for i32 {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

impl WriteIcal for f64 {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

impl WriteIcal for str {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        super::escape_text(self, w)
    }
}

impl WriteIcal for String {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        super::escape_text(self, w)
    }
}

impl WriteIcal for Box<Uri> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self.as_str())
    }
}

impl WriteIcal for Box<Uid> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self.as_str())
    }
}

impl WriteIcal for Box<TzId> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self.as_str())
    }
}

// ============================================================================
// Vec<String> (comma-separated, e.g. CATEGORIES, RESOURCES)
// ============================================================================

impl WriteIcal for Vec<String> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        for (i, s) in self.iter().enumerate() {
            if i > 0 {
                w.write_char(',')?;
            }
            super::escape_text(s, w)?;
        }
        Ok(())
    }
}

// ============================================================================
// Vec<Period> (comma-separated FREEBUSY values)
// ============================================================================

impl WriteIcal for Vec<Period> {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write_comma_separated(self, w)
    }
}

// ============================================================================
// RRule
// ============================================================================

impl WriteIcal for RRule {
    fn write_ical<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let freq = Freq::from(&self.freq);
        write!(w, "FREQ={}", freq_str(freq))?;

        if let Some(interval) = &self.interval {
            write!(w, ";INTERVAL={}", interval.get())?;
        }

        if let Some(term) = &self.termination {
            match term {
                rfc5545_types::rrule::Termination::Count(c) => write!(w, ";COUNT={c}")?,
                rfc5545_types::rrule::Termination::Until(dtod) => {
                    w.write_str(";UNTIL=")?;
                    dtod.write_ical(w)?;
                }
            }
        }

        // Frequency-dependent by-rules
        match &self.freq {
            FreqByRules::Secondly(r) | FreqByRules::Minutely(r) | FreqByRules::Hourly(r) => {
                write_by_period_day_rules(r, w)?;
            }
            FreqByRules::Daily(r) | FreqByRules::Monthly(r) => {
                write_by_month_day_rule(r, w)?;
            }
            FreqByRules::Weekly => {}
            FreqByRules::Yearly(r) => {
                write_yearly_by_rules(r, w)?;
            }
        }

        // Core by-rules
        write_core_by_rules(&self.core_by_rules, w)?;

        if let Some(wkst) = &self.week_start {
            w.write_str(";WKST=")?;
            write_weekday(*wkst, w)?;
        }

        Ok(())
    }
}

// ============================================================================
// RRule helpers
// ============================================================================

fn freq_str(freq: Freq) -> &'static str {
    match freq {
        Freq::Secondly => "SECONDLY",
        Freq::Minutely => "MINUTELY",
        Freq::Hourly => "HOURLY",
        Freq::Daily => "DAILY",
        Freq::Weekly => "WEEKLY",
        Freq::Monthly => "MONTHLY",
        Freq::Yearly => "YEARLY",
    }
}

fn write_weekday<W: fmt::Write>(wd: calendar_types::time::Weekday, w: &mut W) -> fmt::Result {
    w.write_str(match wd {
        calendar_types::time::Weekday::Monday => "MO",
        calendar_types::time::Weekday::Tuesday => "TU",
        calendar_types::time::Weekday::Wednesday => "WE",
        calendar_types::time::Weekday::Thursday => "TH",
        calendar_types::time::Weekday::Friday => "FR",
        calendar_types::time::Weekday::Saturday => "SA",
        calendar_types::time::Weekday::Sunday => "SU",
    })
}

fn write_weekday_num<W: fmt::Write>(wn: &WeekdayNum, w: &mut W) -> fmt::Result {
    if let Some((sign, week)) = wn.ordinal {
        match sign {
            Sign::Pos => {}
            Sign::Neg => w.write_char('-')?,
        }
        write!(w, "{}", week as u8)?;
    }
    write_weekday(wn.weekday, w)
}

fn write_core_by_rules<W: fmt::Write>(rules: &CoreByRules, w: &mut W) -> fmt::Result {
    if let Some(set) = &rules.by_second {
        w.write_str(";BYSECOND=")?;
        write_second_set(set, w)?;
    }
    if let Some(set) = &rules.by_minute {
        w.write_str(";BYMINUTE=")?;
        write_minute_set(set, w)?;
    }
    if let Some(set) = &rules.by_hour {
        w.write_str(";BYHOUR=")?;
        write_hour_set(set, w)?;
    }
    if let Some(set) = &rules.by_day {
        w.write_str(";BYDAY=")?;
        let mut first = true;
        for wn in set.iter() {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write_weekday_num(&wn, w)?;
        }
    }
    if let Some(set) = &rules.by_month {
        w.write_str(";BYMONTH=")?;
        write_month_set(set, w)?;
    }
    if let Some(set) = &rules.by_set_pos {
        w.write_str(";BYSETPOS=")?;
        let mut first = true;
        for yd in set {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", yd.get())?;
        }
    }
    Ok(())
}

fn write_by_period_day_rules<W: fmt::Write>(rules: &ByPeriodDayRules, w: &mut W) -> fmt::Result {
    if let Some(set) = &rules.by_month_day {
        w.write_str(";BYMONTHDAY=")?;
        write_month_day_set(set, w)?;
    }
    if let Some(set) = &rules.by_year_day {
        w.write_str(";BYYEARDAY=")?;
        let mut first = true;
        for yd in set {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", yd.get())?;
        }
    }
    Ok(())
}

fn write_by_month_day_rule<W: fmt::Write>(rules: &ByMonthDayRule, w: &mut W) -> fmt::Result {
    if let Some(set) = &rules.by_month_day {
        w.write_str(";BYMONTHDAY=")?;
        write_month_day_set(set, w)?;
    }
    Ok(())
}

fn write_yearly_by_rules<W: fmt::Write>(rules: &YearlyByRules, w: &mut W) -> fmt::Result {
    if let Some(set) = &rules.by_month_day {
        w.write_str(";BYMONTHDAY=")?;
        write_month_day_set(set, w)?;
    }
    if let Some(set) = &rules.by_year_day {
        w.write_str(";BYYEARDAY=")?;
        let mut first = true;
        for yd in set {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", yd.get())?;
        }
    }
    if let Some(set) = &rules.by_week_no {
        w.write_str(";BYWEEKNO=")?;
        write_week_no_set(set, w)?;
    }
    Ok(())
}

fn write_second_set<W: fmt::Write>(set: &SecondSet, w: &mut W) -> fmt::Result {
    let mut first = true;
    for s in rfc5545_types::rrule::Second::iter() {
        if set.get(s) {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", s as u8)?;
        }
    }
    Ok(())
}

fn write_minute_set<W: fmt::Write>(set: &MinuteSet, w: &mut W) -> fmt::Result {
    let mut first = true;
    for m in rfc5545_types::rrule::Minute::iter() {
        if set.get(m) {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", m as u8)?;
        }
    }
    Ok(())
}

fn write_hour_set<W: fmt::Write>(set: &HourSet, w: &mut W) -> fmt::Result {
    let mut first = true;
    for h in rfc5545_types::rrule::Hour::iter() {
        if set.get(h) {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", h as u8)?;
        }
    }
    Ok(())
}

fn write_month_set<W: fmt::Write>(set: &MonthSet, w: &mut W) -> fmt::Result {
    let mut first = true;
    for m in calendar_types::time::Month::iter() {
        if set.get(m) {
            if !first {
                w.write_char(',')?;
            }
            first = false;
            write!(w, "{}", m as u8)?;
        }
    }
    Ok(())
}

fn write_month_day_set<W: fmt::Write>(set: &MonthDaySet, w: &mut W) -> fmt::Result {
    let mut first = true;
    // Positive days 1..=31
    for d in 1..=31u8 {
        if let Some(md) = MonthDay::from_repr(d) {
            let idx = MonthDaySetIndex::from_signed_month_day(Sign::Pos, md);
            if set.get(idx) {
                if !first {
                    w.write_char(',')?;
                }
                first = false;
                write!(w, "{d}")?;
            }
        }
    }
    // Negative days -1..=-31
    for d in 1..=31u8 {
        if let Some(md) = MonthDay::from_repr(d) {
            let idx = MonthDaySetIndex::from_signed_month_day(Sign::Neg, md);
            if set.get(idx) {
                if !first {
                    w.write_char(',')?;
                }
                first = false;
                write!(w, "-{d}")?;
            }
        }
    }
    Ok(())
}

fn write_week_no_set<W: fmt::Write>(set: &WeekNoSet, w: &mut W) -> fmt::Result {
    let mut first = true;
    // Positive weeks 1..=53
    for i in 1..=53u8 {
        if let Some(wk) = calendar_types::time::IsoWeek::from_index(i) {
            let idx = WeekNoSetIndex::from_signed_week(Sign::Pos, wk);
            if set.get(idx) {
                if !first {
                    w.write_char(',')?;
                }
                first = false;
                write!(w, "{i}")?;
            }
        }
    }
    // Negative weeks -1..=-53
    for i in 1..=53u8 {
        if let Some(wk) = calendar_types::time::IsoWeek::from_index(i) {
            let idx = WeekNoSetIndex::from_signed_week(Sign::Neg, wk);
            if set.get(idx) {
                if !first {
                    w.write_char(',')?;
                }
                first = false;
                write!(w, "-{i}")?;
            }
        }
    }
    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

fn write_comma_separated<T: WriteIcal, W: fmt::Write>(items: &[T], w: &mut W) -> fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            w.write_char(',')?;
        }
        item.write_ical(w)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_types::time::*;

    #[test]
    fn date_ical() {
        let d = Date::new(
            Year::new(2003).unwrap(),
            Month::Dec,
            Day::new(25).unwrap(),
        )
        .unwrap();
        assert_eq!(d.to_ical_string(), "20031225");
    }

    #[test]
    fn time_ical() {
        let t = Time::new(
            Hour::new(12).unwrap(),
            Minute::new(0).unwrap(),
            Second::new(0).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(t.to_ical_string(), "120000");
    }

    #[test]
    fn datetime_utc_ical() {
        let dt = DateTime {
            date: Date::new(
                Year::new(2003).unwrap(),
                Month::Dec,
                Day::new(25).unwrap(),
            )
            .unwrap(),
            time: Time::new(
                Hour::new(12).unwrap(),
                Minute::new(0).unwrap(),
                Second::new(0).unwrap(),
                None,
            )
            .unwrap(),
            marker: Utc,
        };
        assert_eq!(dt.to_ical_string(), "20031225T120000Z");
    }

    #[test]
    fn datetime_local_ical() {
        let dt = DateTime {
            date: Date::new(
                Year::new(2003).unwrap(),
                Month::Dec,
                Day::new(25).unwrap(),
            )
            .unwrap(),
            time: Time::new(
                Hour::new(12).unwrap(),
                Minute::new(0).unwrap(),
                Second::new(0).unwrap(),
                None,
            )
            .unwrap(),
            marker: Local,
        };
        assert_eq!(dt.to_ical_string(), "20031225T120000");
    }

    #[test]
    fn utc_offset_positive() {
        let offset = UtcOffset {
            sign: Sign::Pos,
            hour: Hour::new(8).unwrap(),
            minute: calendar_types::time::Minute::new(0).unwrap(),
            second: NonLeapSecond::new(0).unwrap(),
        };
        assert_eq!(offset.to_ical_string(), "+0800");
    }

    #[test]
    fn utc_offset_negative_with_seconds() {
        let offset = UtcOffset {
            sign: Sign::Neg,
            hour: Hour::new(5).unwrap(),
            minute: calendar_types::time::Minute::new(30).unwrap(),
            second: NonLeapSecond::new(15).unwrap(),
        };
        assert_eq!(offset.to_ical_string(), "-053015");
    }

    #[test]
    fn geo_ical() {
        let g = Geo { lat: 37.386013, lon: -122.082932 };
        assert_eq!(g.to_ical_string(), "37.386013;-122.082932");
    }

    #[test]
    fn signed_duration_ical() {
        let d = SignedDuration {
            sign: Sign::Neg,
            duration: Duration::Nominal(calendar_types::duration::NominalDuration {
                weeks: 0,
                days: 1,
                exact: None,
            }),
        };
        assert_eq!(d.to_ical_string(), "-P1D");
    }

    #[test]
    fn version_ical() {
        assert_eq!(Version::V2_0.to_ical_string(), "2.0");
    }

    #[test]
    fn priority_ical() {
        assert_eq!(Priority::Zero.to_ical_string(), "0");
        assert_eq!(Priority::A1.to_ical_string(), "1");
        assert_eq!(Priority::C3.to_ical_string(), "9");
    }

    #[test]
    fn boolean_ical() {
        assert_eq!(true.to_ical_string(), "TRUE");
        assert_eq!(false.to_ical_string(), "FALSE");
    }
}
