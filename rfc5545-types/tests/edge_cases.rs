use std::num::NonZero;

use calendar_types::primitive::Sign;
use calendar_types::time::{IsoWeek, Month};
use calendar_types::time::Weekday;
use rfc5545_types::rrule::{
    ByRuleBehavior, ByRuleName, Freq, Hour, HourSet, Interval, Minute, MinuteSet, MonthDay,
    MonthDaySet, MonthDaySetIndex, MonthSet, Second, SecondSet, WeekNoSet, WeekNoSetIndex,
    YearDayNum,
};
use rfc5545_types::rrule::weekday_num_set::WeekdayNumSet;
use rfc5545_types::rrule::WeekdayNum;

// ── behavior_with table ──────────────────────────────────────────

#[test]
fn behavior_with_table_complete() {
    use ByRuleBehavior::*;
    use ByRuleName::*;
    use Freq::*;

    // RFC 5545 page 44 table: rows = by-rules, columns = frequencies
    // Secondly, Minutely, Hourly, Daily, Weekly, Monthly, Yearly
    let table: &[(ByRuleName, [Option<ByRuleBehavior>; 7])] = &[
        (ByMonth,    [Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Expand)]),
        (ByWeekNo,   [None,        None,        None,        None,        None,        None,        Some(Expand)]),
        (ByYearDay,  [Some(Limit), Some(Limit), Some(Limit), None,        None,        None,        Some(Expand)]),
        (ByMonthDay, [Some(Limit), Some(Limit), Some(Limit), Some(Limit), None,        Some(Expand),Some(Expand)]),
        (ByDay,      [Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Expand),Some(Note1), Some(Note2)]),
        (ByHour,     [Some(Limit), Some(Limit), Some(Limit), Some(Expand),Some(Expand),Some(Expand),Some(Expand)]),
        (ByMinute,   [Some(Limit), Some(Limit), Some(Expand),Some(Expand),Some(Expand),Some(Expand),Some(Expand)]),
        (BySecond,   [Some(Limit), Some(Expand),Some(Expand),Some(Expand),Some(Expand),Some(Expand),Some(Expand)]),
        (BySetPos,   [Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Limit), Some(Limit)]),
    ];

    let freqs = [Secondly, Minutely, Hourly, Daily, Weekly, Monthly, Yearly];

    for (rule, expected_row) in table {
        for (j, freq) in freqs.iter().enumerate() {
            assert_eq!(
                rule.behavior_with(*freq),
                expected_row[j],
                "{rule:?} x {freq:?}"
            );
        }
    }
}

// ── YearDayNum signed index boundaries ───────────────────

#[test]
fn year_day_num_valid_boundaries() {
    assert!(YearDayNum::from_signed_index(Sign::Pos, 1).is_some());
    assert!(YearDayNum::from_signed_index(Sign::Pos, 366).is_some());
    assert!(YearDayNum::from_signed_index(Sign::Neg, 1).is_some());
    assert!(YearDayNum::from_signed_index(Sign::Neg, 366).is_some());
}

#[test]
fn year_day_num_invalid_boundaries() {
    assert!(YearDayNum::from_signed_index(Sign::Pos, 0).is_none());
    assert!(YearDayNum::from_signed_index(Sign::Pos, 367).is_none());
    assert!(YearDayNum::from_signed_index(Sign::Neg, 0).is_none());
    assert!(YearDayNum::from_signed_index(Sign::Neg, 367).is_none());
    assert!(YearDayNum::from_signed_index(
        Sign::Pos,
        u16::MAX,
    ).is_none());
}

// ── MonthDay boundaries ──────────────────────────────────

#[test]
fn month_day_from_repr_boundaries() {
    assert!(MonthDay::from_repr(0).is_none());
    assert_eq!(MonthDay::from_repr(1), Some(MonthDay::D1));
    assert_eq!(MonthDay::from_repr(31), Some(MonthDay::D31));
    assert!(MonthDay::from_repr(32).is_none());
    assert!(MonthDay::from_repr(255).is_none());
}

// ── MonthDaySet signed index operations ────────────────────

#[test]
fn month_day_set_positive_and_negative() {
    let mut set = MonthDaySet::default();

    let pos_1 = MonthDaySetIndex::from_signed_month_day(Sign::Pos, MonthDay::D1);
    let pos_31 = MonthDaySetIndex::from_signed_month_day(Sign::Pos, MonthDay::D31);
    let neg_1 = MonthDaySetIndex::from_signed_month_day(Sign::Neg, MonthDay::D1);
    let neg_31 = MonthDaySetIndex::from_signed_month_day(Sign::Neg, MonthDay::D31);

    assert!(!set.get(pos_1));
    assert!(!set.get(pos_31));
    assert!(!set.get(neg_1));
    assert!(!set.get(neg_31));

    set.set(pos_1);
    set.set(neg_31);

    assert!(set.get(pos_1));
    assert!(!set.get(pos_31));
    assert!(!set.get(neg_1));
    assert!(set.get(neg_31));
}

// ── WeekNoSet signed index operations ────────────────────

#[test]
fn week_no_set_positive_and_negative() {
    let mut set = WeekNoSet::default();

    let pos_1 = WeekNoSetIndex::from_signed_week(Sign::Pos, IsoWeek::W1);
    let pos_53 = WeekNoSetIndex::from_signed_week(Sign::Pos, IsoWeek::W53);
    let neg_1 = WeekNoSetIndex::from_signed_week(Sign::Neg, IsoWeek::W1);
    let neg_53 = WeekNoSetIndex::from_signed_week(Sign::Neg, IsoWeek::W53);

    assert!(!set.get(pos_1));
    assert!(!set.get(pos_53));
    assert!(!set.get(neg_1));
    assert!(!set.get(neg_53));

    set.set(pos_1);
    set.set(neg_53);

    assert!(set.get(pos_1));
    assert!(!set.get(pos_53));
    assert!(!set.get(neg_1));
    assert!(set.get(neg_53));
}

// ── Interval construction ──────────────────────────────

#[test]
fn interval_default_is_one() {
    let interval = Interval::default();
    assert_eq!(interval.get().get(), 1);
}

#[test]
fn interval_from_nonzero() {
    let interval = Interval::new(NonZero::new(42).unwrap());
    assert_eq!(interval.get().get(), 42);

    let max_interval = Interval::new(NonZero::new(u64::MAX).unwrap());
    assert_eq!(max_interval.get().get(), u64::MAX);
}

#[test]
fn interval_zero_impossible() {
    // NonZero::new(0) returns None, so Interval::new can't be called with 0
    assert!(NonZero::<u64>::new(0).is_none());
}

// ── SecondSet bitset operations ──────────────────────────

#[test]
fn second_set_min_max_indices() {
    let mut set = SecondSet::default();

    set.set(Second::S0);
    set.set(Second::S60);

    assert!(set.get(Second::S0));
    assert!(set.get(Second::S60));
    assert!(!set.get(Second::S30));
}

#[test]
fn second_set_full_iteration() {
    let mut set = SecondSet::default();
    for s in Second::iter() {
        set.set(s);
    }
    for s in Second::iter() {
        assert!(set.get(s));
    }
}

// ── MinuteSet bitset operations ──────────────────────────

#[test]
fn minute_set_min_max_indices() {
    let mut set = MinuteSet::default();

    set.set(Minute::M0);
    set.set(Minute::M59);

    assert!(set.get(Minute::M0));
    assert!(set.get(Minute::M59));
    assert!(!set.get(Minute::M30));
}

// ── HourSet bitset operations ────────────────────────────

#[test]
fn hour_set_min_max_indices() {
    let mut set = HourSet::default();

    set.set(Hour::H0);
    set.set(Hour::H23);

    assert!(set.get(Hour::H0));
    assert!(set.get(Hour::H23));
    assert!(!set.get(Hour::H12));
}

// ── MonthSet bitset operations ───────────────────────────

#[test]
fn month_set_min_max_indices() {
    let mut set = MonthSet::default();

    set.set(Month::Jan);
    set.set(Month::Dec);

    assert!(set.get(Month::Jan));
    assert!(set.get(Month::Dec));
    assert!(!set.get(Month::Jun));
}

#[test]
fn month_set_all_months() {
    let mut set = MonthSet::default();
    for month in Month::iter() {
        set.set(month);
    }
    for month in Month::iter() {
        assert!(set.get(month));
    }
}

// ── Second/Minute/Hour from_repr boundaries ──────────────────

#[test]
fn second_from_repr_boundaries() {
    assert!(Second::from_repr(0).is_some());
    assert!(Second::from_repr(60).is_some());
    assert!(Second::from_repr(61).is_none());
}

#[test]
fn minute_from_repr_boundaries() {
    assert!(Minute::from_repr(0).is_some());
    assert!(Minute::from_repr(59).is_some());
    assert!(Minute::from_repr(60).is_none());
}

#[test]
fn hour_from_repr_boundaries() {
    assert!(Hour::from_repr(0).is_some());
    assert!(Hour::from_repr(23).is_some());
    assert!(Hour::from_repr(24).is_none());
}

// ── WeekdayNum construction and ordering ─────────────────────────────

#[test]
fn weekday_num_plain() {
    let wdn = WeekdayNum {
        ordinal: None,
        weekday: Weekday::Monday,
    };
    assert!(wdn.ordinal.is_none());
    assert_eq!(wdn.weekday, Weekday::Monday);
}

#[test]
fn weekday_num_with_ordinal() {
    let wdn = WeekdayNum {
        ordinal: Some((Sign::Pos, IsoWeek::W1)),
        weekday: Weekday::Friday,
    };
    assert_eq!(wdn.ordinal, Some((Sign::Pos, IsoWeek::W1)));
    assert_eq!(wdn.weekday, Weekday::Friday);
}

#[test]
fn weekday_num_negative_ordinal() {
    let wdn = WeekdayNum {
        ordinal: Some((Sign::Neg, IsoWeek::W53)),
        weekday: Weekday::Sunday,
    };
    assert_eq!(wdn.ordinal, Some((Sign::Neg, IsoWeek::W53)));
}

// ── WeekdayNumSet operations ─────────────────────────────────────────

#[test]
fn weekday_num_set_default_is_empty() {
    let set = WeekdayNumSet::default();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
}

#[test]
fn weekday_num_set_insert_and_contains() {
    let mut set = WeekdayNumSet::default();
    let wdn = WeekdayNum {
        ordinal: None,
        weekday: Weekday::Monday,
    };
    assert!(!set.contains(wdn));
    set.insert(wdn);
    assert!(set.contains(wdn));
    assert_eq!(set.len(), 1);
}

#[test]
fn weekday_num_set_multiple_weekdays() {
    let mut set = WeekdayNumSet::default();
    for weekday in Weekday::iter() {
        set.insert(WeekdayNum {
            ordinal: None,
            weekday,
        });
    }
    assert_eq!(set.len(), 7);
    for weekday in Weekday::iter() {
        assert!(set.contains(WeekdayNum {
            ordinal: None,
            weekday,
        }));
    }
}

#[test]
fn weekday_num_set_with_ordinals() {
    let mut set = WeekdayNumSet::default();
    let first_monday = WeekdayNum {
        ordinal: Some((Sign::Pos, IsoWeek::W1)),
        weekday: Weekday::Monday,
    };
    let last_friday = WeekdayNum {
        ordinal: Some((Sign::Neg, IsoWeek::W1)),
        weekday: Weekday::Friday,
    };
    set.insert(first_monday);
    set.insert(last_friday);
    assert!(set.contains(first_monday));
    assert!(set.contains(last_friday));
    assert!(!set.contains(WeekdayNum {
        ordinal: None,
        weekday: Weekday::Monday,
    }));
}

// ── Empty set iteration via default ──────────────────────────────────

#[test]
fn second_set_default_all_unset() {
    let set = SecondSet::default();
    for s in Second::iter() {
        assert!(!set.get(s));
    }
}

#[test]
fn minute_set_default_all_unset() {
    let set = MinuteSet::default();
    for m in Minute::iter() {
        assert!(!set.get(m));
    }
}

#[test]
fn hour_set_default_all_unset() {
    let set = HourSet::default();
    for h in Hour::iter() {
        assert!(!set.get(h));
    }
}

#[test]
fn month_set_default_all_unset() {
    let set = MonthSet::default();
    for m in Month::iter() {
        assert!(!set.get(m));
    }
}

// ── Full iteration for all bitset types ──────────────────────────────

#[test]
fn minute_set_full_iteration() {
    let mut set = MinuteSet::default();
    for m in Minute::iter() {
        set.set(m);
    }
    for m in Minute::iter() {
        assert!(set.get(m));
    }
}

#[test]
fn hour_set_full_iteration() {
    let mut set = HourSet::default();
    for h in Hour::iter() {
        set.set(h);
    }
    for h in Hour::iter() {
        assert!(set.get(h));
    }
}
