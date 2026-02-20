use jscalendar::parser;

// ── UTC DateTime parsing edge cases ──────────────────────────────────

#[test]
fn parse_valid_utc_datetime() {
    let result = parser::parse_full(parser::utc_date_time)("2024-01-15T13:00:00Z");
    assert!(result.is_ok());
}

#[test]
fn parse_utc_datetime_leap_year_feb_29() {
    assert!(parser::parse_full(parser::utc_date_time)("2000-02-29T00:00:00Z").is_ok());
    assert!(parser::parse_full(parser::utc_date_time)("2024-02-29T12:30:00Z").is_ok());
}

#[test]
fn parse_utc_datetime_non_leap_feb_29_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("1900-02-29T00:00:00Z").is_err());
    assert!(parser::parse_full(parser::utc_date_time)("2023-02-29T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_feb_30_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-02-30T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_apr_31_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-04-31T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_jun_31_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-06-31T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_sep_31_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-09-31T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_nov_31_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-11-31T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_month_extremes() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-00-15T00:00:00Z").is_err());
    assert!(parser::parse_full(parser::utc_date_time)("2024-13-15T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_day_zero_invalid() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-01-00T00:00:00Z").is_err());
}

#[test]
fn parse_utc_datetime_year_boundaries() {
    assert!(parser::parse_full(parser::utc_date_time)("0000-01-01T00:00:00Z").is_ok());
    assert!(parser::parse_full(parser::utc_date_time)("9999-12-31T23:59:59Z").is_ok());
}

// ── Fractional seconds in parsing ────────────────────────────────────

#[test]
fn parse_utc_datetime_with_fractional_seconds() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-01-01T00:00:00.1Z").is_ok());
    assert!(parser::parse_full(parser::utc_date_time)("2024-01-01T00:00:00.123456789Z").is_ok());
}

#[test]
fn parse_utc_datetime_fractional_too_many_digits() {
    assert!(parser::parse_full(parser::utc_date_time)("2024-01-01T00:00:00.1234567890Z").is_err());
}

// ── Duration parsing edge cases ──────────────────────────────────────

#[test]
fn parse_duration_week_only() {
    assert!(parser::parse_full(parser::duration)("P1W").is_ok());
}

#[test]
fn parse_duration_day_only() {
    assert!(parser::parse_full(parser::duration)("P1D").is_ok());
}

#[test]
fn parse_duration_time_only() {
    assert!(parser::parse_full(parser::duration)("PT1H").is_ok());
    assert!(parser::parse_full(parser::duration)("PT1H1M").is_ok());
    assert!(parser::parse_full(parser::duration)("PT1H0M1S").is_ok());
}

#[test]
fn parse_duration_combined() {
    assert!(parser::parse_full(parser::duration)("P1DT2H30M").is_ok());
}

#[test]
fn parse_duration_max_u32_values() {
    assert!(parser::parse_full(parser::duration)("P4294967295W").is_ok());
    assert!(parser::parse_full(parser::duration)("PT4294967295H").is_ok());
}

#[test]
fn parse_duration_missing_prefix() {
    assert!(parser::parse_full(parser::duration)("1D").is_err());
    assert!(parser::parse_full(parser::duration)("T1H").is_err());
}

#[test]
fn parse_duration_empty_time() {
    assert!(parser::parse_full(parser::duration)("PT").is_err());
}

// ── Signed duration edge cases ───────────────────────────────────────

#[test]
fn parse_signed_duration_positive() {
    assert!(parser::parse_full(parser::signed_duration)("+P1D").is_ok());
}

#[test]
fn parse_signed_duration_negative() {
    assert!(parser::parse_full(parser::signed_duration)("-P1D").is_ok());
}

#[test]
fn parse_signed_duration_no_sign() {
    assert!(parser::parse_full(parser::signed_duration)("P1D").is_ok());
}

// ── Local DateTime parsing ───────────────────────────────────────────

#[test]
fn parse_local_datetime_valid() {
    assert!(parser::parse_full(parser::local_date_time)("2024-01-15T13:00:00").is_ok());
}

#[test]
fn parse_local_datetime_rejects_z_suffix() {
    assert!(parser::parse_full(parser::local_date_time)("2024-01-15T13:00:00Z").is_err());
}

// ── Leap year boundary: divisible by 100 but not 400 ─────────────────

#[test]
fn parse_utc_datetime_century_leap_year_2100() {
    // 2100 is not a leap year (divisible by 100 but not 400)
    assert!(parser::parse_full(parser::utc_date_time)("2100-02-29T00:00:00Z").is_err());
    assert!(parser::parse_full(parser::utc_date_time)("2100-02-28T00:00:00Z").is_ok());
}

#[test]
fn parse_utc_datetime_century_leap_year_2400() {
    // 2400 is a leap year (divisible by 400)
    assert!(parser::parse_full(parser::utc_date_time)("2400-02-29T00:00:00Z").is_ok());
}

// ── Empty and malformed inputs ───────────────────────────────────────

#[test]
fn parse_utc_datetime_empty_string() {
    assert!(parser::parse_full(parser::utc_date_time)("").is_err());
}

#[test]
fn parse_duration_empty_string() {
    assert!(parser::parse_full(parser::duration)("").is_err());
}

#[test]
fn parse_duration_p_only() {
    assert!(parser::parse_full(parser::duration)("P").is_err());
}

// ── Duration u32 overflow ────────────────────────────────────────────

#[test]
fn parse_duration_overflow_u32() {
    // 4294967296 = u32::MAX + 1
    assert!(parser::parse_full(parser::duration)("P4294967296W").is_err());
}

// ── Date-time without timezone (generic) ─────────────────────────────

#[test]
fn parse_generic_datetime_valid() {
    assert!(parser::parse_full(parser::date_time)("2024-01-15T13:00:00").is_ok());
}

#[test]
fn parse_generic_datetime_with_fractional() {
    assert!(parser::parse_full(parser::date_time)("2024-06-15T12:30:00.5").is_ok());
}
