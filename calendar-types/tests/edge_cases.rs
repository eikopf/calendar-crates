use calendar_types::time::{
    Date, Day, FractionalSecond, Hour, InvalidFractionalSecondError, IsoWeek, Minute, Month,
    NonLeapSecond, Second, Time, Weekday, Year,
};
use calendar_types::string::{InvalidUidError, InvalidUriError, Uid, Uri};

#[test]
fn year_min_max_boundaries() {
    assert_eq!(Year::new(0), Ok(Year::MIN));
    assert_eq!(Year::new(9999), Ok(Year::MAX));
    assert!(Year::new(10000).is_err());
    assert!(Year::new(u16::MAX).is_err());
}

#[test]
fn year_leap_year_boundaries() {
    assert!(Year::new(2024).unwrap().is_leap_year());
    assert!(!Year::new(1900).unwrap().is_leap_year());
    assert!(!Year::new(2100).unwrap().is_leap_year());
    assert!(Year::new(2000).unwrap().is_leap_year());
    assert!(Year::new(2400).unwrap().is_leap_year());
    assert!(!Year::new(2023).unwrap().is_leap_year());
    assert!(Year::new(0).unwrap().is_leap_year());
    assert!(!Year::new(100).unwrap().is_leap_year());
    assert!(Year::new(400).unwrap().is_leap_year());
}

#[test]
fn month_boundaries() {
    assert!(Month::new(0).is_err());
    assert_eq!(Month::new(1), Ok(Month::Jan));
    assert_eq!(Month::new(12), Ok(Month::Dec));
    assert!(Month::new(13).is_err());
}

#[test]
fn day_boundaries() {
    assert!(Day::new(0).is_err());
    assert_eq!(Day::new(1), Ok(Day::D01));
    assert_eq!(Day::new(31), Ok(Day::D31));
    assert!(Day::new(32).is_err());
}

#[test]
fn date_feb_29_leap_vs_non_leap() {
    assert!(Date::new(Year::new(2000).unwrap(), Month::Feb, Day::D29).is_ok());
    assert!(Date::new(Year::new(1900).unwrap(), Month::Feb, Day::D29).is_err());
    assert!(Date::new(Year::new(2023).unwrap(), Month::Feb, Day::D29).is_err());
}

#[test]
fn date_feb_30_always_invalid() {
    for y in [2000u16, 2024, 1900, 2023] {
        assert!(Date::new(Year::new(y).unwrap(), Month::Feb, Day::D30).is_err());
    }
}

#[test]
fn date_30day_month_day31_invalid() {
    let year = Year::new(2024).unwrap();
    for month in [Month::Apr, Month::Jun, Month::Sep, Month::Nov] {
        assert!(Date::new(year, month, Day::D30).is_ok());
        assert!(Date::new(year, month, Day::D31).is_err());
    }
}

#[test]
fn date_31day_months_valid() {
    let year = Year::new(2024).unwrap();
    for month in [Month::Jan, Month::Mar, Month::May, Month::Jul, Month::Aug, Month::Oct, Month::Dec] {
        assert!(Date::new(year, month, Day::D31).is_ok());
    }
}

#[test]
fn date_feb_28_always_valid() {
    for y in [2000u16, 1900, 2023, 2024] {
        assert!(Date::new(Year::new(y).unwrap(), Month::Feb, Day::D28).is_ok());
    }
}

#[test]
fn hour_boundaries() {
    assert_eq!(Hour::new(0), Ok(Hour::H00));
    assert_eq!(Hour::new(23), Ok(Hour::H23));
    assert!(Hour::new(24).is_err());
}

#[test]
fn minute_boundaries() {
    assert_eq!(Minute::new(0), Ok(Minute::M00));
    assert_eq!(Minute::new(59), Ok(Minute::M59));
    assert!(Minute::new(60).is_err());
}

#[test]
fn second_boundaries_with_leap() {
    assert_eq!(Second::new(0), Ok(Second::S00));
    assert_eq!(Second::new(60), Ok(Second::S60));
    assert!(Second::new(61).is_err());
}

#[test]
fn non_leap_second_rejects_60() {
    assert_eq!(NonLeapSecond::new(59), Ok(NonLeapSecond::S59));
    assert!(NonLeapSecond::new(60).is_err());
}

#[test]
fn non_leap_second_converts_to_second() {
    assert_eq!(NonLeapSecond::S00.to_second(), Second::S00);
    assert_eq!(NonLeapSecond::S59.to_second(), Second::S59);
}

#[test]
fn fractional_second_boundaries() {
    assert_eq!(FractionalSecond::new(0), Err(InvalidFractionalSecondError::AllZero));
    assert_eq!(FractionalSecond::new(1).unwrap(), FractionalSecond::MIN);
    assert_eq!(FractionalSecond::new(999_999_999).unwrap(), FractionalSecond::MAX);
    assert!(FractionalSecond::new(1_000_000_000).is_err());
    assert!(FractionalSecond::new(u32::MAX).is_err());
}

#[test]
fn time_with_leap_second() {
    assert!(Time::new(Hour::H23, Minute::M59, Second::S60, None).is_ok());
}

#[test]
fn time_with_frac() {
    let frac = FractionalSecond::new(500_000_000).unwrap();
    assert!(Time::new(Hour::H00, Minute::M00, Second::S00, Some(frac)).is_ok());
}

#[test]
fn weekday_boundaries() {
    assert_eq!(Weekday::from_repr(0), Some(Weekday::Monday));
    assert_eq!(Weekday::from_repr(6), Some(Weekday::Sunday));
    assert_eq!(Weekday::from_repr(7), None);
    assert_eq!(Weekday::iter().count(), 7);
}

#[test]
fn iso_week_boundaries() {
    assert_eq!(IsoWeek::from_index(0), None);
    assert_eq!(IsoWeek::from_index(1), Some(IsoWeek::W1));
    assert_eq!(IsoWeek::from_index(53), Some(IsoWeek::W53));
    assert_eq!(IsoWeek::from_index(54), None);
    assert_eq!(IsoWeek::W1.index().get(), 1);
    assert_eq!(IsoWeek::W53.index().get(), 53);
}

#[test]
fn uid_edge_cases() {
    assert_eq!(Uid::new(""), Err(InvalidUidError::EmptyString));
    assert!(Uid::new("a").is_ok());
    assert!(Uid::new("hello world").is_ok());
}

#[test]
fn uri_edge_cases() {
    assert_eq!(Uri::new(""), Err(InvalidUriError::EmptyString));
    assert_eq!(Uri::new("nocolon"), Err(InvalidUriError::MissingColon));
    assert_eq!(Uri::new("1http:foo"), Err(InvalidUriError::SchemeStartsWithNonLetter));
    assert!(Uri::new("fo o:bar").is_err());
    assert!(Uri::new("http:foo").is_ok());
    assert!(Uri::new("a:b").is_ok());
    assert_eq!(Uri::new("http:foo").unwrap().scheme(), "http");
    assert!(Uri::new(":").is_err());
}

// ── Month iteration ──────────────────────────────────────────────────

#[test]
fn month_iteration_count() {
    assert_eq!(Month::iter().count(), 12);
    assert_eq!(Month::iter().next(), Some(Month::Jan));
    assert_eq!(Month::iter().last(), Some(Month::Dec));
}

// ── Date::maximum_day ────────────────────────────────────────────────

#[test]
fn date_maximum_day_feb_leap_vs_non_leap() {
    assert_eq!(Date::maximum_day(Year::new(2024).unwrap(), Month::Feb), Day::D29);
    assert_eq!(Date::maximum_day(Year::new(2023).unwrap(), Month::Feb), Day::D28);
    assert_eq!(Date::maximum_day(Year::new(2000).unwrap(), Month::Feb), Day::D29);
    assert_eq!(Date::maximum_day(Year::new(1900).unwrap(), Month::Feb), Day::D28);
}

#[test]
fn date_maximum_day_30_day_months() {
    let year = Year::new(2024).unwrap();
    for month in [Month::Apr, Month::Jun, Month::Sep, Month::Nov] {
        assert_eq!(Date::maximum_day(year, month), Day::D30);
    }
}

#[test]
fn date_maximum_day_31_day_months() {
    let year = Year::new(2024).unwrap();
    for month in [Month::Jan, Month::Mar, Month::May, Month::Jul, Month::Aug, Month::Oct, Month::Dec] {
        assert_eq!(Date::maximum_day(year, month), Day::D31);
    }
}

// ── Month::number ────────────────────────────────────────────────────

#[test]
fn month_number_values() {
    assert_eq!(Month::Jan.number().get(), 1);
    assert_eq!(Month::Dec.number().get(), 12);
}
