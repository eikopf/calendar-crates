use jscalendar::model::string::{
    AlphaNumeric, CalAddress, ContentId, CustomTimeZoneId, EmailAddr, GeoUri, Id,
    ImplicitJsonPointer, InvalidCalAddressError, InvalidContentIdError,
    InvalidCustomTimeZoneIdError, InvalidEmailAddrError, InvalidGeoUriError, InvalidIdError,
    InvalidImplicitJsonPointerError, InvalidMediaTypeError, InvalidVendorStrError, MediaType,
    VendorStr,
};

// Id edge cases

#[test]
fn id_empty_string() {
    assert_eq!(Id::new(""), Err(InvalidIdError::EmptyString));
}

#[test]
fn id_single_char() {
    assert!(Id::new("a").is_ok());
    assert!(Id::new("Z").is_ok());
    assert!(Id::new("0").is_ok());
    assert!(Id::new("-").is_ok());
    assert!(Id::new("_").is_ok());
}

#[test]
fn id_max_length_255() {
    let s: String = "a".repeat(255);
    assert!(Id::new(&s).is_ok());
}

#[test]
fn id_too_long_256() {
    let s: String = "a".repeat(256);
    assert_eq!(Id::new(&s), Err(InvalidIdError::TooLong));
}

#[test]
fn id_invalid_chars() {
    assert!(Id::new(" ").is_err());
    assert!(Id::new("hello world").is_err());
    assert!(Id::new("foo@bar").is_err());
    assert!(Id::new("foo.bar").is_err());
}

// VendorStr edge cases

#[test]
fn vendor_str_empty() {
    assert_eq!(VendorStr::new(""), Err(InvalidVendorStrError::EmptyString));
}

#[test]
fn vendor_str_no_colon() {
    assert_eq!(VendorStr::new("abc"), Err(InvalidVendorStrError::MissingColon));
}

#[test]
fn vendor_str_colon_at_start() {
    assert_eq!(VendorStr::new(":suffix"), Err(InvalidVendorStrError::EmptyPrefix));
}

#[test]
fn vendor_str_colon_at_end() {
    assert_eq!(VendorStr::new("prefix:"), Err(InvalidVendorStrError::EmptySuffix));
}

#[test]
fn vendor_str_valid() {
    assert!(VendorStr::new("a:b").is_ok());
    assert!(VendorStr::new("example.com:property").is_ok());
}

#[test]
fn vendor_str_multiple_colons() {
    let v = VendorStr::new("a:b:c").unwrap();
    assert_eq!(v.vendor_domain(), "a");
    assert_eq!(v.suffix(), "b:c");
}

// EmailAddr edge cases

#[test]
fn email_empty() {
    assert_eq!(EmailAddr::new(""), Err(InvalidEmailAddrError::EmptyString));
}

#[test]
fn email_no_at() {
    assert_eq!(EmailAddr::new("userexample.com"), Err(InvalidEmailAddrError::InvalidAtSign));
}

#[test]
fn email_multiple_at() {
    assert_eq!(EmailAddr::new("user@host@domain"), Err(InvalidEmailAddrError::InvalidAtSign));
}

#[test]
fn email_empty_local() {
    assert_eq!(EmailAddr::new("@domain.com"), Err(InvalidEmailAddrError::EmptyLocalPart));
}

#[test]
fn email_empty_domain() {
    assert_eq!(EmailAddr::new("user@"), Err(InvalidEmailAddrError::EmptyDomainPart));
}

#[test]
fn email_just_at() {
    assert_eq!(EmailAddr::new("@"), Err(InvalidEmailAddrError::EmptyLocalPart));
}

#[test]
fn email_valid() {
    let e = EmailAddr::new("user@example.com").unwrap();
    assert_eq!(e.local_part(), "user");
    assert_eq!(e.domain(), "example.com");
}

// GeoUri edge cases

#[test]
fn geo_uri_empty() {
    assert_eq!(GeoUri::new(""), Err(InvalidGeoUriError::EmptyString));
}

#[test]
fn geo_uri_missing_prefix() {
    assert_eq!(GeoUri::new("40.7,-74.0"), Err(InvalidGeoUriError::NotGeoScheme));
}

#[test]
fn geo_uri_missing_longitude() {
    assert_eq!(GeoUri::new("geo:40.7"), Err(InvalidGeoUriError::MissingLongitude));
}

#[test]
fn geo_uri_empty_latitude() {
    assert_eq!(GeoUri::new("geo:,-74.0"), Err(InvalidGeoUriError::MissingLatitude));
}

#[test]
fn geo_uri_empty_longitude() {
    assert_eq!(GeoUri::new("geo:40.7,"), Err(InvalidGeoUriError::MissingLongitude));
}

#[test]
fn geo_uri_invalid_latitude() {
    assert_eq!(GeoUri::new("geo:abc,-74.0"), Err(InvalidGeoUriError::InvalidLatitude));
}

#[test]
fn geo_uri_invalid_longitude() {
    assert_eq!(GeoUri::new("geo:40.7,abc"), Err(InvalidGeoUriError::InvalidLongitude));
}

#[test]
fn geo_uri_valid_with_altitude() {
    assert!(GeoUri::new("geo:40.7128,-74.0060,100").is_ok());
}

#[test]
fn geo_uri_valid_with_params() {
    assert!(GeoUri::new("geo:40.7128,-74.0060;u=10").is_ok());
}

#[test]
fn geo_uri_extreme_coords() {
    assert!(GeoUri::new("geo:90.0,180.0").is_ok());
    assert!(GeoUri::new("geo:-90.0,-180.0").is_ok());
    assert!(GeoUri::new("geo:0.0,0.0").is_ok());
}

// CalAddress edge cases

#[test]
fn cal_address_empty() {
    assert_eq!(CalAddress::new(""), Err(InvalidCalAddressError::EmptyString));
}

#[test]
fn cal_address_not_mailto() {
    assert_eq!(CalAddress::new("http://example.com"), Err(InvalidCalAddressError::NotMailto));
    assert_eq!(CalAddress::new("user@example.com"), Err(InvalidCalAddressError::NotMailto));
}

#[test]
fn cal_address_valid() {
    let ca = CalAddress::new("mailto:user@example.com").unwrap();
    assert_eq!(ca.email(), "user@example.com");
}

// ImplicitJsonPointer edge cases

#[test]
fn implicit_json_pointer_empty_is_valid() {
    assert!(ImplicitJsonPointer::new("").is_ok());
}

#[test]
fn implicit_json_pointer_explicit_slash_invalid() {
    assert_eq!(
        ImplicitJsonPointer::new("/foo"),
        Err(InvalidImplicitJsonPointerError::Explicit)
    );
}

#[test]
fn implicit_json_pointer_bare_tilde_invalid() {
    assert!(ImplicitJsonPointer::new("foo~").is_err());
    assert!(ImplicitJsonPointer::new("~2").is_err());
}

#[test]
fn implicit_json_pointer_valid_escapes() {
    assert!(ImplicitJsonPointer::new("~0").is_ok());
    assert!(ImplicitJsonPointer::new("~1").is_ok());
    assert!(ImplicitJsonPointer::new("a~0b~1c").is_ok());
}

#[test]
fn implicit_json_pointer_with_segments() {
    assert!(ImplicitJsonPointer::new("foo/bar/baz").is_ok());
}

// CustomTimeZoneId edge cases

#[test]
fn custom_tz_empty() {
    assert_eq!(
        CustomTimeZoneId::new(""),
        Err(InvalidCustomTimeZoneIdError::EmptyString)
    );
}

#[test]
fn custom_tz_missing_slash() {
    assert_eq!(
        CustomTimeZoneId::new("America/New_York"),
        Err(InvalidCustomTimeZoneIdError::MissingSlash)
    );
}

#[test]
fn custom_tz_valid() {
    assert!(CustomTimeZoneId::new("/Custom-Zone").is_ok());
    assert!(CustomTimeZoneId::new("/My Zone").is_ok());
}

#[test]
fn custom_tz_invalid_body_char() {
    // Semicolons, commas, colons are not valid paramtext chars
    assert!(CustomTimeZoneId::new("/bad;zone").is_err());
    assert!(CustomTimeZoneId::new("/bad\"zone").is_err());
}

// MediaType edge cases

#[test]
fn media_type_empty() {
    assert_eq!(MediaType::new(""), Err(InvalidMediaTypeError::EmptyString));
}

#[test]
fn media_type_no_slash() {
    assert_eq!(MediaType::new("textplain"), Err(InvalidMediaTypeError::MissingSlash));
}

#[test]
fn media_type_empty_type() {
    assert_eq!(MediaType::new("/plain"), Err(InvalidMediaTypeError::EmptyType));
}

#[test]
fn media_type_empty_subtype() {
    assert_eq!(MediaType::new("text/"), Err(InvalidMediaTypeError::EmptySubtype));
}

#[test]
fn media_type_valid() {
    let mt = MediaType::new("text/plain").unwrap();
    assert_eq!(mt.type_part(), "text");
    assert_eq!(mt.subtype(), "plain");
}

#[test]
fn media_type_with_params() {
    let mt = MediaType::new("text/plain;charset=utf-8").unwrap();
    assert_eq!(mt.type_part(), "text");
    assert_eq!(mt.subtype(), "plain");
}

// ── ContentId edge cases ─────────────────────────────────────────────

#[test]
fn content_id_empty() {
    assert_eq!(
        ContentId::new(""),
        Err(InvalidContentIdError::EmptyString)
    );
}

#[test]
fn content_id_single_char() {
    assert!(ContentId::new("a").is_ok());
}

#[test]
fn content_id_valid() {
    assert!(ContentId::new("cid:part1@example.com").is_ok());
    assert!(ContentId::new("any-string-is-valid").is_ok());
}

// ── AlphaNumeric edge cases ──────────────────────────────────────────

#[test]
fn alphanumeric_empty_is_valid() {
    // Empty string has no non-alphanumeric chars, so it passes validation
    assert!(AlphaNumeric::new("").is_ok());
}

#[test]
fn alphanumeric_valid() {
    assert!(AlphaNumeric::new("abc").is_ok());
    assert!(AlphaNumeric::new("ABC").is_ok());
    assert!(AlphaNumeric::new("123").is_ok());
    assert!(AlphaNumeric::new("abc123XYZ").is_ok());
}

#[test]
fn alphanumeric_invalid_chars() {
    assert!(AlphaNumeric::new(" ").is_err());
    assert!(AlphaNumeric::new("hello world").is_err());
    assert!(AlphaNumeric::new("foo-bar").is_err());
    assert!(AlphaNumeric::new("foo_bar").is_err());
    assert!(AlphaNumeric::new("foo@bar").is_err());
}
