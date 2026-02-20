//! Parsers for primitive (i.e. terminal) grammar elements.

use std::num::NonZero;

use lexical_parse_float::{FromLexicalWithOptions, NumberFormatBuilder, Options, OptionsBuilder};
use mitsein::vec1::Vec1;
use winnow::{
    Parser,
    ascii::{Caseless, alpha1, digit1},
    combinator::{
        alt, delimited, empty, fail, not, opt, preceded, repeat, separated, separated_pair,
        terminated, trace,
    },
    error::{FromExternalError, ParserError},
    stream::{AsChar, Compare, Stream, StreamIsPartial},
    token::{any, literal, none_of, one_of, take_while},
};

use crate::{
    model::{
        css::Css3Color,
        primitive::{
            AlarmAction, CalendarUserType, Class, ClassValue, CompletionPercentage, Date, DateTime,
            DateTimeOrDate, Day, DisplayType, Duration, Encoding, ExactDuration, FeatureType,
            Float, FormatType, FormatTypeBuf, FreeBusyType, Geo, Gregorian, Hour, Integer, IsoWeek,
            Language, Method, Minute, Month, NominalDuration, NonLeapSecond, ParticipantType,
            ParticipationRole, ParticipationStatus, Period, PositiveInteger, Priority,
            ProximityValue, RelationshipType, RequestStatus, RequestStatusCode, ResourceType,
            Second, Sign, SignedDuration, Status, Time, TimeFormat, TimeTransparency, Token,
            TriggerRelation, Utc, UtcOffset, ValueType, Version, Year,
        },
        string::{InvalidCharError, Name, ParamValue, TextBuf, TzId, Uid, Uri},
    },
    parser::config::{Config, DefaultConfig},
};

use super::{
    InputStream,
    error::{
        CalendarParseError, InvalidCompletionPercentageError, InvalidDateError,
        InvalidDurationTimeError, InvalidGeoError, InvalidIntegerError, InvalidPriorityError,
        InvalidRawTimeError, InvalidUtcOffsetError,
    },
};

/// The format for parsing floats with [`lexical_parse_float`].
const ICALENDAR_FLOAT_FORMAT: u128 = NumberFormatBuilder::new()
    .required_integer_digits(true)
    .required_fraction_digits(false)
    .no_exponent_notation(true)
    .required_mantissa_digits(false)
    .no_special(true)
    .build_strict();

/// The options for parsing floats with [`lexical_parse_float`].
const ICALENDAR_FLOAT_OPTIONS: Options = OptionsBuilder::new()
    .nan_string(None)
    .inf_string(None)
    .infinity_string(None)
    .build_strict();

macro_rules! match_iana_token {
    ($input:ident, $enum_name:ident, $($lhs:literal => $rhs:ident),* $(,)?) => {{
        let name = name.parse_next($input)?;

        let res = ::hashify::map_ignore_case!(
            name.as_str().as_bytes(),
            $enum_name,
            $($lhs => $enum_name::$rhs,)*
        );

        match res {
            Some(&res) => Ok($crate::model::primitive::Token::Known(res)),
            None => Ok($crate::model::primitive::Token::Unknown(name)),
        }
    }};
}

/// Parses a [`RequestStatus`].
pub fn request_status<I, E>(input: &mut I) -> Result<RequestStatus, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    Ok(RequestStatus {
        code: status_code.parse_next(input)?,
        description: preceded(';', text)
            .map(Box::<str>::from)
            .parse_next(input)?,
        exception_data: opt(preceded(';', text).map(Box::<str>::from)).parse_next(input)?,
    })
}

/// Parses a [`ParticipantType`].
pub fn participant_type<I, E>(input: &mut I) -> Result<Token<ParticipantType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ParticipantType,
        "ACTIVE" => Active,
        "INACTIVE" => Inactive,
        "SPONSOR" => Sponsor,
        "CONTACT" => Contact,
        "BOOKING-CONTACT" => BookingContact,
        "EMERGENCY-CONTACT" => EmergencyContact,
        "PUBLICITY-CONTACT" => PublicityContact,
        "PLANNER-CONTACT" => PlannerContact,
        "PERFORMER" => Performer,
        "SPEAKER" => Speaker,
    )
}

/// Parses a [`ResourceType`].
pub fn resource_type<I, E>(input: &mut I) -> Result<Token<ResourceType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ResourceType,
        "ROOM" => Room,
        "PROJECTOR" => Projector,
        "REMOTE-CONFERENCE-AUDIO" => RemoteConferenceAudio,
        "REMOTE-CONFERENCE-VIDEO" => RemoteConferenceVideo,
    )
}

/// Parses a [`RequestStatusCode`].
pub fn status_code<I, E>(input: &mut I) -> Result<RequestStatusCode, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let (class_byte, _, major, minor) = (
        lz_dec_uint::<I, u8, E>,
        '.',
        lz_dec_uint::<I, u8, E>,
        opt(preceded('.', lz_dec_uint::<I, u8, E>)),
    )
        .parse_next(input)?;

    let class = Class::from_u8(class_byte).ok_or_else(|| {
        E::from_external_error(input, CalendarParseError::InvalidStatusClass(class_byte))
    })?;

    Ok(RequestStatusCode {
        class,
        major,
        minor,
    })
}

/// Parses an [`AlarmAction`].
pub fn alarm_action<I, E>(input: &mut I) -> Result<Token<AlarmAction, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, AlarmAction,
        "AUDIO" => Audio,
        "DISPLAY" => Display,
        "EMAIL" => Email,
    )
}

/// Parses a [`TzId`].
pub fn tz_id<I, E>(input: &mut I) -> Result<Box<TzId>, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    // the grammar for this parser is as follows in EBNF:
    //
    // tzid value = [ "/" ], text ;
    //
    // but a literal forward slash is perfectly permissible in a text value, so this is equivalent
    // to just parsing a text value

    text.map(TextBuf::into_boxed_text)
        .map(TzId::from_boxed_text)
        .parse_next(input)
}

/// Parses a [`TimeTransparency`].
pub fn time_transparency<I, E>(input: &mut I) -> Result<TimeTransparency, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    alt((
        Caseless("TRANSPARENT").value(TimeTransparency::Transparent),
        Caseless("OPAQUE").value(TimeTransparency::Opaque),
    ))
    .parse_next(input)
}

/// Parses a [`FeatureType`].
pub fn feature_type<I, E>(input: &mut I) -> Result<Token<FeatureType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, FeatureType,
        "AUDIO" => Audio,
        "CHAT" => Chat,
        "FEED" => Feed,
        "MODERATOR" => Moderator,
        "PHONE" => Phone,
        "SCREEN" => Screen,
        "VIDEO" => Video,
    )
}

/// Parses a [`DisplayType`].
pub fn display_type<I, E>(input: &mut I) -> Result<Token<DisplayType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, DisplayType,
        "BADGE" => Badge,
        "GRAPHIC" => Graphic,
        "FULLSIZE" => Fullsize,
        "THUMBNAIL" => Thumbnail,
    )
}

/// Parses the exact string `GREGORIAN`, which occurs in the calendar scale
/// property. This parser returns `()` because the Gregorian calendar is the
/// _only_ calendar scale recognised by RFC 5545 and its successors.
pub fn gregorian<I, E>(input: &mut I) -> Result<Gregorian, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    Caseless("GREGORIAN").value(Gregorian).parse_next(input)
}

/// Parses the exact string `2.0`, which occurs in the version property.
pub fn version<I, E>(input: &mut I) -> Result<Version, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    // using Caseless here does nothing, but it makes the trait bounds match
    // the other parsers in this module
    Caseless("2.0").value(Version::V2_0).parse_next(input)
}

/// Parses a [`Method`].
pub fn method<I, E>(input: &mut I) -> Result<Token<Method, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, Method,
        "PUBLISH" => Publish,
        "REQUEST" => Request,
        "REPLY" => Reply,
        "ADD" => Add,
        "CANCEL" => Cancel,
        "REFRESH" => Refresh,
        "COUNTER" => Counter,
        "DECLINECOUNTER" => DeclineCounter,
    )
}

/// Parses a [`Uid`].
pub fn uid<I, E>(input: &mut I) -> Result<Box<Uid>, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    text.map(TextBuf::into_boxed_text)
        .map(Uid::from_boxed_text)
        .parse_next(input)
}

/// Parses an RFC 5646 [language tag](Language).
pub fn language<I, E>(input: &mut I) -> Result<Language, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let name = name.parse_next(input)?;

    // TODO: this throws away the allocation produced by the name parser; find a way to reuse the
    // allocation here instead
    Language::parse(name.as_str()).map_err(|err| E::from_external_error(input, err.into()))
}

/// Parses an RFC 3986 URI. The description of the grammar in RFC 5545 is
/// somewhat ambiguous, so in particular we first parse a sequence of characters
/// which may occur in a URI and then attempt to verify that it is actually a
/// valid URI.
///
/// # Escaping
/// URIs are notable in iCalendar because they can appear as values in both properties and property
/// parameters. When they appear in property parameters, they MUST occur as quoted strings (RFC
/// 5545 §3.3.13), and in particular quoted strings do not admit the escape sequences for
/// semicolons and commas (RFC 5545 §3.2). The `ESCAPED` parameter controls whether or not these
/// escape sequences are enabled, and it should only be `true` if the parser is used to parse the
/// value of a property.
///
/// # Calendar Addresses
/// Since there is no difference between the grammar for URIs and the grammar for "calendar user
/// addresses" (RFC 5545 § 3.3.3), this parser is also invoked whenever an iCalendar RFC calls for
/// the `CAL-ADDRESS` value type.
pub fn uri<I, E, const ESCAPED: bool>(input: &mut I) -> Result<Box<Uri>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    /// Parses the longest sequence of characters which can occur in a URI. See
    /// RFC 3986 sections 2.1, 2.2, and 2.3 for details.
    fn uri_character<I, E>(input: &mut I) -> Result<I::Token, E>
    where
        I: StreamIsPartial + Stream,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        one_of(('!', '#'..=';', '=', '?'..='Z', '[', ']', '_', 'a'..='z')).parse_next(input)
    }

    /// Accepts a subset of textual escapes if ESCAPED is true.
    fn text_escape<I, E>(input: &mut I) -> Result<(), E>
    where
        I: StreamIsPartial + Stream + Compare<char>,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        ('\\', alt((';', ','))).void().parse_next(input)
    }

    let slice = if ESCAPED {
        repeat::<_, _, (), _, _>(0.., alt((text_escape, uri_character.void())))
            .take()
            //.map(Uri)
            .parse_next(input)
    } else {
        repeat::<_, _, (), _, _>(0.., uri_character)
            .take()
            //.map(Uri)
            .parse_next(input)
    }?;

    I::try_into_string(&slice)
        .map(|s| {
            // SAFETY: the parser guarantees that this string is a valid URI
            unsafe { Uri::from_boxed_unchecked(s.into_boxed_str()) }
        })
        .map_err(|e| E::from_external_error(input, e))
}

/// Parses a base64-encoded binary blob.
pub fn binary<I, E>(input: &mut I) -> Result<Vec<u8>, E>
where
    I: InputStream,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let mut config = DefaultConfig;
    binary_with_config(input, &mut config)
}

pub fn binary_with_config<I, E>(input: &mut I, _: &mut impl Config) -> Result<Vec<u8>, E>
where
    I: InputStream,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    const ENGINE: base64::engine::GeneralPurpose = base64::prelude::BASE64_STANDARD;

    // TODO: if Escaped implemented FindSlice<&str> then we could add that bound to InputStream and
    // use `take_until(0.., "\r\n")` here instead

    let source = repeat::<_, _, (), _, _>(0.., (not(("\r\n", alt(("\t", " ")))), any))
        .take()
        .parse_next(input)?;

    let str = I::try_into_str(&source).map_err(|err| E::from_external_error(input, err.into()))?;

    base64::Engine::decode(&ENGINE, str.as_ref().as_bytes())
        .map_err(|err| E::from_external_error(input, err.into()))
}

pub fn class_value<I, E>(input: &mut I) -> Result<Token<ClassValue, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ClassValue,
        "PUBLIC" => Public,
        "PRIVATE" => Private,
        "CONFIDENTIAL" => Confidential,
    )
}

/// Parses a calendar user type value (RFC 5545 §3.2.3).
pub fn calendar_user_type<I, E>(input: &mut I) -> Result<Token<CalendarUserType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, CalendarUserType,
        "INDIVIDUAL" => Individual,
        "GROUP" => Group,
        "RESOURCE" => Resource,
        "ROOM" => Room,
        "UNKNOWN" => Unknown,
    )
}

/// Parses an [`Encoding`].
pub fn inline_encoding<I, E>(input: &mut I) -> Result<Encoding, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    alt((
        Caseless("8BIT").value(Encoding::Bit8),
        Caseless("BASE64").value(Encoding::Base64),
    ))
    .parse_next(input)
}

/// Parses a [`FormatTypeBuf`] (effectively a MIME type).
pub fn format_type<I, E>(input: &mut I) -> Result<FormatTypeBuf, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    /// The `reg-name` grammar rule as in RFC 4288 §4.2
    fn reg_name<I, E>(input: &mut I) -> Result<I::Slice, E>
    where
        I: StreamIsPartial + Stream + Compare<char>,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        repeat::<_, _, (), _, _>(
            1..,
            one_of((
                'a'..='z',
                'A'..='Z',
                '!',
                '#',
                '$',
                '&',
                '.',
                '+',
                '-',
                '^',
                '_',
            )),
        )
        .take()
        .parse_next(input)
    }

    let slice = (reg_name, '/', reg_name).take().parse_next(input)?;
    let s = I::try_into_str(&slice).map_err(|e| E::from_external_error(input, e.into()))?;

    FormatType::new(s.as_ref())
        .map(FormatType::to_owned)
        .map_err(|_| E::from_external_error(input, CalendarParseError::InvalidFormatType(slice)))
}

/// Parses a [`FreeBusyType`].
pub fn free_busy_type<I, E>(input: &mut I) -> Result<Token<FreeBusyType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, FreeBusyType,
        "FREE" => Free,
        "BUSY" => Busy,
        "BUSY-UNAVAILABLE" => BusyUnavailable,
        "BUSY-TENTATIVE" => BusyTentative,
    )
}

/// Parses a [`Status`].
pub fn status<I, E>(input: &mut I) -> Result<Status, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let name = name.parse_next(input)?;

    let res = hashify::map_ignore_case!(
        name.as_str().as_bytes(),
        Status,
        "TENTATIVE" => Status::Tentative,
        "CONFIRMED" => Status::Confirmed,
        "CANCELLED" => Status::Cancelled,
        "NEEDS-ACTION" => Status::NeedsAction,
        "COMPLETED" => Status::Completed,
        "IN-PROCESS" => Status::InProcess,
        "DRAFT" => Status::Draft,
        "FINAL" => Status::Final,
    );

    match res {
        Some(&res) => Ok(res),
        None => fail.parse_next(input),
    }
}

/// Parses a [`ParticipationStatus`].
pub fn participation_status<I, E>(input: &mut I) -> Result<Token<ParticipationStatus, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ParticipationStatus,
        "NEEDS-ACTION" => NeedsAction,
        "IN-PROCESS" => InProcess,
        "COMPLETED" => Completed,
        "DELEGATED" => Delegated,
        "TENTATIVE" => Tentative,
        "ACCEPTED" => Accepted,
        "DECLINED" => Declined,
    )
}

/// Parses a [`TriggerRelation`].
pub fn alarm_trigger_relationship<I, E>(input: &mut I) -> Result<TriggerRelation, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    alt((
        Caseless("START").value(TriggerRelation::Start),
        Caseless("END").value(TriggerRelation::End),
    ))
    .parse_next(input)
}

/// Parses a [`RelationshipType`].
pub fn relationship_type<I, E>(input: &mut I) -> Result<Token<RelationshipType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, RelationshipType,
        "PARENT" => Parent,
        "CHILD" => Child,
        "SIBLING" => Sibling,
        "SNOOZE" => Snooze,
    )
}

/// Parses a [`ProximityValue`].
pub fn proximity_value<I, E>(input: &mut I) -> Result<Token<ProximityValue, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ProximityValue,
        "ARRIVE" => Arrive,
        "DEPART" => Depart,
        "CONNECT" => Connect,
        "DISCONNECT" => Disconnect,
    )
}

/// Parses a [`ParticipationRole`].
pub fn participation_role<I, E>(input: &mut I) -> Result<Token<ParticipationRole, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ParticipationRole,
        "CHAIR" => Chair,
        "REQ-PARTICIPANT" => ReqParticipant,
        "OPT-PARTICIPANT" => OptParticipant,
        "NON-PARTICIPANT" => NonParticipant,
    )
}

/// Parses a [`ValueType`].
pub fn value_type<I, E>(input: &mut I) -> Result<Token<ValueType, Box<Name>>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    match_iana_token!(input, ValueType,
        "BINARY" => Binary,
        "BOOLEAN" => Boolean,
        "CAL-ADDRESS" => CalAddress,
        "DATE" => Date,
        "DATE-TIME" => DateTime,
        "DURATION" => Duration,
        "FLOAT" => Float,
        "INTEGER" => Integer,
        "PERIOD" => Period,
        "RECUR" => Recur,
        "TEXT" => Text,
        "TIME" => Time,
        "URI" => Uri,
        "UTC-OFFSET" => UtcOffset,
    )
}

/// Parses a [`Name`].
pub fn name<I, E>(input: &mut I) -> Result<Box<Name>, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let slice = take_while(1.., |t: I::Token| {
        let c = t.as_char();
        c == '-' || c.is_ascii_alphanumeric()
    })
    .parse_next(input)?;

    let s = I::try_into_string(&slice).map_err(|e| E::from_external_error(input, e.into()))?;

    // SAFETY: the result of the parser is definitely non-empty
    assert!(!s.is_empty());
    let s = unsafe { mitsein::string1::String1::from_string_unchecked(s) };

    // SAFETY: the bytes of `s` are definitely either alphanumeric or the hyphen, as guaranteed by
    // the parser
    let name = unsafe { Name::from_boxed_unchecked(s.into_boxed_str1()) };

    Ok(name)
}

/// Parses a [`ParamValue`].
pub fn param_value<I, E>(input: &mut I) -> Result<Box<ParamValue>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn param_text<I, E>(input: &mut I) -> Result<I::Slice, E>
    where
        I: StreamIsPartial + Stream,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        repeat(0.., none_of((..' ', '"', ',', ':', ';', '\u{007F}')))
            .map(|()| ())
            .take()
            .parse_next(input)
    }

    fn quoted_string<I, E>(input: &mut I) -> Result<I::Slice, E>
    where
        I: StreamIsPartial + Stream + Compare<char>,
        I::Token: AsChar + Clone,
        E: ParserError<I>,
    {
        delimited(
            '"',
            repeat(0.., none_of((..' ', '"', '\u{007F}')))
                .map(|()| ())
                .take(),
            '"',
        )
        .parse_next(input)
    }

    alt((quoted_string, param_text))
        .try_map(|slice| {
            I::try_into_string(&slice)?
                .try_into()
                .map_err(|e: InvalidCharError| {
                    CalendarParseError::InvalidCharInParamValue(e.invalid_char)
                })
        })
        .parse_next(input)
}

/// Parses a comma-separated sequence of one or more values.
pub fn comma_seq1<I, O, E>(p: impl Parser<I, O, E>) -> impl Parser<I, Vec1<O>, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    E: ParserError<I>,
{
    separated(1.., p, literal(',')).map(|v: Vec<_>| {
        v.try_into()
            .expect("the parser must produce at least one value")
    })
}

/// Parses a comma-delimited non-empty sequence of text values.
pub fn text_seq<I, E>(input: &mut I) -> Result<Vec1<TextBuf>, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    comma_seq1(text).parse_next(input)
}

/// Parses a [`Text`].
pub fn text<I, E>(input: &mut I) -> Result<TextBuf, E>
where
    I: InputStream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn safe_text<I, E>(input: &mut I) -> Result<I::Str, E>
    where
        I: InputStream,
        I::Token: AsChar + Clone,
        E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
    {
        repeat::<_, _, (), _, _>(1.., none_of(('\\', ';', ',', ..' ')))
            .take()
            .try_map(|s| I::try_into_str(&s).map_err(Into::into))
            .parse_next(input)
    }

    fn text_escape<I, E>(input: &mut I) -> Result<I::Str, E>
    where
        I: InputStream,
        E: ParserError<I>,
    {
        preceded(
            '\\',
            alt((
                '\\'.value("\\"),
                'n'.value("\n"),
                'N'.value("\n"),
                ';'.value(";"),
                ','.value(","),
            )),
        )
        .map(I::str_from_static_str)
        .parse_next(input)
    }

    let mut buf = String::new();

    loop {
        match alt((safe_text, text_escape)).parse_next(input) {
            Ok(str) => buf.push_str(str.as_ref()),
            Err(()) => {
                // SAFETY: the parser guarantees this is a valid TextBuf
                let t = unsafe { TextBuf::new_unchecked(buf) };
                return Ok(t);
            }
        }
    }
}

/// Parses a [`Period`].
///
/// Since an explicit period may admit both absolute and local (floating) times
/// in the same object, we cannot immediately determine whether a given period
/// is valid as described in RFC 5545 §3.3.9.
pub fn period<I, E>(input: &mut I) -> Result<Period, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    enum DtOrDur {
        Dt(DateTime<TimeFormat>),
        Dur(Duration),
    }

    separated_pair(
        datetime,
        '/',
        alt((
            datetime.map(DtOrDur::Dt),
            duration.map(|sd| DtOrDur::Dur(sd.duration)),
        )),
    )
    .map(|(start, end)| match end {
        DtOrDur::Dt(end) => Period::Explicit { start, end },
        DtOrDur::Dur(duration) => Period::Start { start, duration },
    })
    .parse_next(input)
}

/// Parses a [`SignedDuration`].
pub fn duration<I, E>(input: &mut I) -> Result<SignedDuration, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn dur_time<I, E>(input: &mut I) -> Result<ExactDuration, E>
    where
        I: InputStream,
        <I as Stream>::Token: AsChar,
        E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
    {
        let checkpoint = input.checkpoint();

        let components = preceded(
            'T',
            (
                opt(terminated(lz_dec_uint::<I, u32, E>, 'H')),
                opt(terminated(lz_dec_uint::<I, u32, E>, 'M')),
                opt(terminated(lz_dec_uint::<I, u32, E>, 'S')),
            ),
        )
        .parse_next(input)?;

        match components {
            (hours, None, seconds) if hours.is_none() && seconds.is_none() => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    CalendarParseError::InvalidDurationTime(InvalidDurationTimeError {
                        hours: None::<u32>,
                        seconds: None::<u32>,
                    }),
                ))
            }
            (Some(_), None, Some(_)) => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    CalendarParseError::InvalidDurationTime(InvalidDurationTimeError {
                        hours: components.0,
                        seconds: components.2,
                    }),
                ))
            }
            (hours, minutes, seconds) => Ok(ExactDuration {
                hours: hours.unwrap_or(0),
                minutes: minutes.unwrap_or(0),
                seconds: seconds.unwrap_or(0),
                frac: None,
            }),
        }
    }

    separated_pair(
        opt(sign),
        'P',
        alt((
            dur_time.map(|exact| Duration::Exact(exact)),
            separated_pair(lz_dec_uint::<I, u32, E>, 'D', opt(dur_time)).map(|(days, exact)| {
                Duration::Nominal(NominalDuration {
                    weeks: 0,
                    days,
                    exact,
                })
            }),
            terminated(lz_dec_uint::<I, u32, E>, 'W').map(|weeks| {
                Duration::Nominal(NominalDuration {
                    weeks,
                    days: 0,
                    exact: None,
                })
            }),
        )),
    )
    .map(|(s, dur)| SignedDuration {
        sign: s.unwrap_or(Sign::Pos),
        duration: dur,
    })
    .parse_next(input)
}

/// Parses a [`DateTimeOrDate<TimeFormat>`].
pub fn datetime_or_date<I, E>(input: &mut I) -> Result<DateTimeOrDate<TimeFormat>, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let (date, time) = (date, opt(preceded('T', time))).parse_next(input)?;

    Ok(match time {
        Some((time, marker)) => DateTimeOrDate::DateTime(DateTime { date, time, marker }),
        None => DateTimeOrDate::Date(date),
    })
}

/// Parses a datetime of the form `YYYYMMDDThhmmss`, with an optional time
/// format suffix.
pub fn datetime<I, E>(input: &mut I) -> Result<DateTime<TimeFormat>, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    (date, 'T', time)
        .map(|(date, _, (time, marker))| DateTime { date, time, marker })
        .parse_next(input)
}

/// Parses a datetime of the form `YYYYMMDDThhmmssZ`, including the mandatory
/// UTC marker suffix.
pub fn datetime_utc<I, E>(input: &mut I) -> Result<DateTime<Utc>, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    (date, 'T', time_utc)
        .map(|(date, _, time)| DateTime {
            date,
            time,
            marker: Utc,
        })
        .parse_next(input)
}

/// Parses a date of the form YYYYMMDD.
pub fn date<I, E>(input: &mut I) -> Result<Date, E>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let checkpoint = input.checkpoint();

    let year = (
        digit::<_, _, 10>,
        digit::<_, _, 10>,
        digit::<_, _, 10>,
        digit::<_, _, 10>,
    )
        .map(|(x, y, z, w)| (x as u16) * 1000 + (y as u16) * 100 + (z as u16) * 10 + w as u16)
        .parse_next(input)?;

    let month = (digit::<_, _, 10>, digit::<_, _, 10>)
        .map(|(x, y)| x * 10 + y)
        .parse_next(input)?;

    let day = (digit::<_, _, 10>, digit::<_, _, 10>)
        .map(|(x, y)| x * 10 + y)
        .parse_next(input)?;

    let y = Year::new(year);
    let m = Month::new(month);
    let d = Day::new(day);

    match (y, m, d) {
        (Ok(y), Ok(m), Ok(d)) => match Date::new(y, m, d) {
            Ok(date) => Ok(date),
            Err(_) => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    CalendarParseError::InvalidDate(InvalidDateError { year, month, day }),
                ))
            }
        },
        _ => {
            input.reset(&checkpoint);
            Err(E::from_external_error(
                input,
                CalendarParseError::InvalidDate(InvalidDateError { year, month, day }),
            ))
        }
    }
}

/// Parses a [`UtcOffset`].
pub fn utc_offset<I, E>(input: &mut I) -> Result<UtcOffset, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    fn digit2<I, E>(input: &mut I) -> Result<u8, E>
    where
        I: StreamIsPartial + Stream,
        I::Token: AsChar,
        E: ParserError<I>,
    {
        (digit::<_, _, 10>, digit::<_, _, 10>)
            .map(|(a, b)| 10 * a + b)
            .parse_next(input)
    }

    let sign = sign.parse_next(input)?;
    let hours = digit2.parse_next(input)?;

    if hours >= 24 {
        return Err(E::from_external_error(
            input,
            CalendarParseError::InvalidUtcOffset(InvalidUtcOffsetError::BadHours(hours)),
        ));
    }

    let minutes = digit2.parse_next(input)?;

    if minutes >= 60 {
        return Err(E::from_external_error(
            input,
            CalendarParseError::InvalidUtcOffset(InvalidUtcOffsetError::BadMinutes(minutes)),
        ));
    }

    let seconds = opt(digit2).parse_next(input)?;

    if let Some(seconds @ 60..) = seconds {
        return Err(E::from_external_error(
            input,
            CalendarParseError::InvalidUtcOffset(InvalidUtcOffsetError::BadSeconds(seconds)),
        ));
    }

    match seconds {
        Some(0) | None if hours == 0 && minutes == 0 && sign == Sign::Neg => {
            Err(E::from_external_error(
                input,
                CalendarParseError::InvalidUtcOffset(InvalidUtcOffsetError::NegativeZero),
            ))
        }
        Some(seconds @ 60..) => Err(E::from_external_error(
            input,
            CalendarParseError::InvalidUtcOffset(InvalidUtcOffsetError::BadSeconds(seconds)),
        )),
        _ => Ok(UtcOffset {
            sign,
            hour: Hour::new(hours).unwrap(),
            minute: Minute::new(minutes).unwrap(),
            second: NonLeapSecond::new(seconds.unwrap_or(0)).unwrap(),
        }),
    }
}

/// Parses a [`Time`] followed by a [`TimeFormat`] suffix.
pub fn time<I, E>(input: &mut I) -> Result<(Time, TimeFormat), E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    (raw_time, time_format).parse_next(input)
}

/// Parses a [`Time`] followed by the mandatory UTC marker.
pub fn time_utc<I, E>(input: &mut I) -> Result<Time, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    (raw_time, utc_marker)
        .parse_next(input)
        .map(|(time, ())| time)
}

/// Parses a [`Time`] (without time format suffix).
pub fn raw_time<I, E>(input: &mut I) -> Result<Time, E>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let checkpoint = input.checkpoint();

    let hours = (digit::<_, _, 10>, digit::<_, _, 10>)
        .map(|(tens, ones)| tens * 10 + ones)
        .parse_next(input)?;

    let minutes = (digit::<_, _, 10>, digit::<_, _, 10>)
        .map(|(tens, ones)| tens * 10 + ones)
        .parse_next(input)?;

    let seconds = (digit::<_, _, 10>, digit::<_, _, 10>)
        .map(|(tens, ones)| tens * 10 + ones)
        .parse_next(input)?;

    let h = Hour::new(hours);
    let m = Minute::new(minutes);
    let s = Second::new(seconds);

    match (h, m, s) {
        (Ok(h), Ok(m), Ok(s)) => match Time::new(h, m, s, None) {
            Ok(time) => Ok(time),
            Err(_) => {
                input.reset(&checkpoint);
                Err(E::from_external_error(
                    input,
                    CalendarParseError::InvalidRawTime(InvalidRawTimeError {
                        hours,
                        minutes,
                        seconds,
                    }),
                ))
            }
        },
        _ => {
            input.reset(&checkpoint);
            Err(E::from_external_error(
                input,
                CalendarParseError::InvalidRawTime(InvalidRawTimeError {
                    hours,
                    minutes,
                    seconds,
                }),
            ))
        }
    }
}

/// Parses the time format string suffix (an optional `Z`).
pub fn time_format<I, E>(input: &mut I) -> Result<TimeFormat, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    E: ParserError<I>,
{
    alt((
        utc_marker.value(TimeFormat::Utc),
        empty.value(TimeFormat::Local),
    ))
    .parse_next(input)
}

/// Parses the UTC marker string (`Z`).
pub fn utc_marker<I, E>(input: &mut I) -> Result<(), E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    E: ParserError<I>,
{
    'Z'.void().parse_next(input)
}

/// Parses 1 or 2 digits into an [`IsoWeek`].
pub fn iso_week_index<I, E>(input: &mut I) -> Result<IsoWeek, E>
where
    I: StreamIsPartial + Stream,
    I::Token: AsChar + Clone,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let (a, b) = (digit::<_, _, 10>, opt(digit::<_, _, 10>)).parse_next(input)?;

    let value = match b {
        Some(b) => 10 * a + b,
        None => a,
    };

    match IsoWeek::from_index(value) {
        Some(week) => Ok(week),
        None => Err(E::from_external_error(
            input,
            CalendarParseError::InvalidIsoWeekIndex(value),
        )),
    }
}

/// Parses a [`Priority`].
pub fn priority<I, E>(input: &mut I) -> Result<Priority, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let value = integer.parse_next(input)?;

    match value {
        0 => Ok(Priority::Zero),
        1 => Ok(Priority::A1),
        2 => Ok(Priority::A2),
        3 => Ok(Priority::A3),
        4 => Ok(Priority::B1),
        5 => Ok(Priority::B2),
        6 => Ok(Priority::B3),
        7 => Ok(Priority::C1),
        8 => Ok(Priority::C2),
        9 => Ok(Priority::C3),
        _ => Err(E::from_external_error(
            input,
            CalendarParseError::InvalidPriority(InvalidPriorityError(value)),
        )),
    }
}

/// Parses a [`CompletionPercentage`].
pub fn completion_percentage<I, E>(input: &mut I) -> Result<CompletionPercentage, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let value = integer.parse_next(input)?;

    match value {
        pct @ 0..=100 => Ok(CompletionPercentage::new(pct as u8).unwrap()),
        other => Err(E::from_external_error(
            input,
            CalendarParseError::InvalidCompletionPercentage(InvalidCompletionPercentageError(
                other,
            )),
        )),
    }
}

/// Parses a [`Geo`].
pub fn geo_with_config<I, E>(input: &mut I, config: &mut impl Config) -> Result<Geo, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let lat = float_with_config(input, config)?;
    let _ = ';'.parse_next(input)?;
    let lon = float_with_config(input, config)?;

    if lat.abs() > 91.0 {
        Err(E::from_external_error(
            input,
            CalendarParseError::InvalidGeo(InvalidGeoError::LatOutOfBounds(lat)),
        ))
    } else if lon.abs() > 181.0 {
        Err(E::from_external_error(
            input,
            CalendarParseError::InvalidGeo(InvalidGeoError::LonOutOfBounds(lon)),
        ))
    } else {
        Ok(Geo { lat, lon })
    }
}

/// Parses a [`Geo`].
pub fn geo<I, E>(input: &mut I) -> Result<Geo, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let mut config = DefaultConfig;
    geo_with_config(input, &mut config)
}

/// Parses the boolean value of `TRUE` or `FALSE`, ignoring case.
pub fn bool_caseless<I, E>(input: &mut I) -> Result<bool, E>
where
    I: StreamIsPartial + Stream + Compare<Caseless<&'static str>>,
    E: ParserError<I>,
{
    alt((Caseless("TRUE").value(true), Caseless("FALSE").value(false))).parse_next(input)
}

/// Parses a [`PositiveInteger`].
pub fn positive_integer<I, E>(input: &mut I) -> Result<PositiveInteger, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let int = integer.parse_next(input)?;
    let res = u32::try_from(int).ok().and_then(NonZero::new);

    match res {
        Some(value) => Ok(value),
        None => Err(E::from_external_error(
            input,
            CalendarParseError::InvalidPositiveInteger(int),
        )),
    }
}

/// Parses an [`Integer`].
pub fn integer<I, E>(input: &mut I) -> Result<Integer, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let sign = opt(sign).parse_next(input)?;
    let digits: u64 = lz_dec_uint.parse_next(input)?;

    i64::try_from(digits)
        .ok()
        .and_then(|d| d.checked_mul(sign.unwrap_or_default() as i64))
        .and_then(|i| Integer::try_from(i).ok())
        .ok_or_else(|| {
            E::from_external_error(
                input,
                CalendarParseError::InvalidInteger(InvalidIntegerError { sign, digits }),
            )
        })
}

pub fn float_with_config<I, E>(input: &mut I, config: &mut impl Config) -> Result<Float, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let slice = (opt(sign), digit1, opt(('.', digit1)))
        .take()
        .parse_next(input)?;

    let str = I::try_into_str(&slice).map_err(|e| E::from_external_error(input, e.into()))?;

    let parsed_float = f64::from_lexical_with_options::<ICALENDAR_FLOAT_FORMAT>(
        str.as_ref().as_bytes(),
        &ICALENDAR_FLOAT_OPTIONS,
    );

    match parsed_float {
        Ok(f) => Ok(f),
        Err(error) => config
            .handle_float_parse_failure(str.as_ref(), error)
            .map_err(|e| E::from_external_error(input, e)),
    }
}

/// Parses a [`Float`].
pub fn float<I, E>(input: &mut I) -> Result<Float, E>
where
    I: InputStream,
    I::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let mut config = DefaultConfig;
    float_with_config(input, &mut config)
}

/// Parses a [`Sign`].
pub fn sign<I, E>(input: &mut I) -> Result<Sign, E>
where
    I: StreamIsPartial + Stream + Compare<char>,
    E: ParserError<I>,
{
    alt(('+'.value(Sign::Pos), '-'.value(Sign::Neg))).parse_next(input)
}

pub fn color<I, E>(input: &mut I) -> Result<Css3Color, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    let source = alpha1.parse_next(input)?;
    let s = I::try_into_str(&source).map_err(|e| E::from_external_error(input, e.into()))?;

    let color = hashify::map_ignore_case!(
        s.as_ref().as_bytes(),
        Css3Color,
        "aliceblue" => Css3Color::AliceBlue,
        "antiquewhite" => Css3Color::AntiqueWhite,
        "aqua" => Css3Color::Aqua,
        "aquamarine" => Css3Color::Aquamarine,
        "azure" => Css3Color::Azure,
        "beige" => Css3Color::Beige,
        "bisque" => Css3Color::Bisque,
        "black" => Css3Color::Black,
        "blanchedalmond" => Css3Color::BlanchedAlmond,
        "blue" => Css3Color::Blue,
        "blueviolet" => Css3Color::BlueViolet,
        "brown" => Css3Color::Brown,
        "burlywood" => Css3Color::BurlyWood,
        "cadetblue" => Css3Color::CadetBlue,
        "chartreuse" => Css3Color::Chartreuse,
        "chocolate" => Css3Color::Chocolate,
        "coral" => Css3Color::Coral,
        "cornflowerblue" => Css3Color::CornflowerBlue,
        "cornsilk" => Css3Color::Cornsilk,
        "crimson" => Css3Color::Crimson,
        "cyan" => Css3Color::Cyan,
        "darkblue" => Css3Color::DarkBlue,
        "darkcyan" => Css3Color::DarkCyan,
        "darkgoldenrod" => Css3Color::DarkGoldenRod,
        "darkgray" => Css3Color::DarkGray,
        "darkgrey" => Css3Color::DarkGrey,
        "darkgreen" => Css3Color::DarkGreen,
        "darkkhaki" => Css3Color::DarkKhaki,
        "darkmagenta" => Css3Color::DarkMagenta,
        "darkolivegreen" => Css3Color::DarkOliveGreen,
        "darkorange" => Css3Color::DarkOrange,
        "darkorchid" => Css3Color::DarkOrchid,
        "darkred" => Css3Color::DarkRed,
        "darksalmon" => Css3Color::DarkSalmon,
        "darkseagreen" => Css3Color::DarkSeaGreen,
        "darkslateblue" => Css3Color::DarkSlateBlue,
        "darkslategray" => Css3Color::DarkSlateGray,
        "darkslategrey" => Css3Color::DarkSlateGrey,
        "darkturquoise" => Css3Color::DarkTurquoise,
        "darkviolet" => Css3Color::DarkViolet,
        "deeppink" => Css3Color::DeepPink,
        "deepskyblue" => Css3Color::DeepSkyBlue,
        "dimgray" => Css3Color::DimGray,
        "dimgrey" => Css3Color::DimGrey,
        "dodgerblue" => Css3Color::DodgerBlue,
        "firebrick" => Css3Color::FireBrick,
        "floralwhite" => Css3Color::FloralWhite,
        "forestgreen" => Css3Color::ForestGreen,
        "fuchsia" => Css3Color::Fuchsia,
        "gainsboro" => Css3Color::Gainsboro,
        "ghostwhite" => Css3Color::GhostWhite,
        "gold" => Css3Color::Gold,
        "goldenrod" => Css3Color::GoldenRod,
        "gray" => Css3Color::Gray,
        "grey" => Css3Color::Grey,
        "green" => Css3Color::Green,
        "greenyellow" => Css3Color::GreenYellow,
        "honeydew" => Css3Color::HoneyDew,
        "hotpink" => Css3Color::HotPink,
        "indianred" => Css3Color::IndianRed,
        "indigo" => Css3Color::Indigo,
        "ivory" => Css3Color::Ivory,
        "khaki" => Css3Color::Khaki,
        "lavender" => Css3Color::Lavender,
        "lavenderblush" => Css3Color::LavenderBlush,
        "lawngreen" => Css3Color::LawnGreen,
        "lemonchiffon" => Css3Color::LemonChiffon,
        "lightblue" => Css3Color::LightBlue,
        "lightcoral" => Css3Color::LightCoral,
        "lightcyan" => Css3Color::LightCyan,
        "lightgoldenrodyellow" => Css3Color::LightGoldenRodYellow,
        "lightgray" => Css3Color::LightGray,
        "lightgrey" => Css3Color::LightGrey,
        "lightgreen" => Css3Color::LightGreen,
        "lightpink" => Css3Color::LightPink,
        "lightsalmon" => Css3Color::LightSalmon,
        "lightseagreen" => Css3Color::LightSeaGreen,
        "lightskyblue" => Css3Color::LightSkyBlue,
        "lightslategray" => Css3Color::LightSlateGray,
        "lightslategrey" => Css3Color::LightSlateGrey,
        "lightsteelblue" => Css3Color::LightSteelBlue,
        "lightyellow" => Css3Color::LightYellow,
        "lime" => Css3Color::Lime,
        "limegreen" => Css3Color::LimeGreen,
        "linen" => Css3Color::Linen,
        "magenta" => Css3Color::Magenta,
        "maroon" => Css3Color::Maroon,
        "mediumaquamarine" => Css3Color::MediumAquaMarine,
        "mediumblue" => Css3Color::MediumBlue,
        "mediumorchid" => Css3Color::MediumOrchid,
        "mediumpurple" => Css3Color::MediumPurple,
        "mediumseagreen" => Css3Color::MediumSeaGreen,
        "mediumslateblue" => Css3Color::MediumSlateBlue,
        "mediumspringgreen" => Css3Color::MediumSpringGreen,
        "mediumturquoise" => Css3Color::MediumTurquoise,
        "mediumvioletred" => Css3Color::MediumVioletRed,
        "midnightblue" => Css3Color::MidnightBlue,
        "mintcream" => Css3Color::MintCream,
        "mistyrose" => Css3Color::MistyRose,
        "moccasin" => Css3Color::Moccasin,
        "navajowhite" => Css3Color::NavajoWhite,
        "navy" => Css3Color::Navy,
        "oldlace" => Css3Color::OldLace,
        "olive" => Css3Color::Olive,
        "olivedrab" => Css3Color::OliveDrab,
        "orange" => Css3Color::Orange,
        "orangered" => Css3Color::OrangeRed,
        "orchid" => Css3Color::Orchid,
        "palegoldenrod" => Css3Color::PaleGoldenRod,
        "palegreen" => Css3Color::PaleGreen,
        "paleturquoise" => Css3Color::PaleTurquoise,
        "palevioletred" => Css3Color::PaleVioletRed,
        "papayawhip" => Css3Color::PapayaWhip,
        "peachpuff" => Css3Color::PeachPuff,
        "peru" => Css3Color::Peru,
        "pink" => Css3Color::Pink,
        "plum" => Css3Color::Plum,
        "powderblue" => Css3Color::PowderBlue,
        "purple" => Css3Color::Purple,
        "red" => Css3Color::Red,
        "rosybrown" => Css3Color::RosyBrown,
        "royalblue" => Css3Color::RoyalBlue,
        "saddlebrown" => Css3Color::SaddleBrown,
        "salmon" => Css3Color::Salmon,
        "sandybrown" => Css3Color::SandyBrown,
        "seagreen" => Css3Color::SeaGreen,
        "seashell" => Css3Color::SeaShell,
        "sienna" => Css3Color::Sienna,
        "silver" => Css3Color::Silver,
        "skyblue" => Css3Color::SkyBlue,
        "slateblue" => Css3Color::SlateBlue,
        "slategray" => Css3Color::SlateGray,
        "slategrey" => Css3Color::SlateGrey,
        "snow" => Css3Color::Snow,
        "springgreen" => Css3Color::SpringGreen,
        "steelblue" => Css3Color::SteelBlue,
        "tan" => Css3Color::Tan,
        "teal" => Css3Color::Teal,
        "thistle" => Css3Color::Thistle,
        "tomato" => Css3Color::Tomato,
        "turquoise" => Css3Color::Turquoise,
        "violet" => Css3Color::Violet,
        "wheat" => Css3Color::Wheat,
        "white" => Css3Color::White,
        "whitesmoke" => Css3Color::WhiteSmoke,
        "yellow" => Css3Color::Yellow,
        "yellowgreen" => Css3Color::YellowGreen,
    );

    match color {
        Some(&color) => Ok(color),
        None => fail.parse_next(input),
    }
}

/// Parses a single token from the `input`, converts it into a `char`, and then
/// invokes [`char::make_ascii_lowercase`] and returns the result.
pub fn ascii_lower<I, E>(input: &mut I) -> Result<char, E>
where
    I: StreamIsPartial + Stream,
    I::Token: AsChar + Clone,
    E: ParserError<I>,
{
    let mut c = any.parse_next(input)?.as_char();
    c.make_ascii_lowercase();
    Ok(c)
}

/// Parses a single digit (of the base given by `RADIX`) and returns its value.
pub fn digit<I, E, const RADIX: u32>(input: &mut I) -> Result<u8, E>
where
    I: StreamIsPartial + Stream,
    I::Token: AsChar,
    E: ParserError<I>,
{
    match any.parse_next(input)?.as_char().to_digit(RADIX) {
        Some(value) => Ok(value as u8),
        None => Err(E::from_input(input)),
    }
}

/// A version of [`dec_uint`] that accepts leading zeros.
///
/// [`dec_uint`]: winnow::ascii::dec_uint
pub(crate) fn lz_dec_uint<I, O, E>(input: &mut I) -> Result<O, E>
where
    I: InputStream,
    <I as Stream>::Token: AsChar,
    O: winnow::ascii::Uint,
    E: ParserError<I> + FromExternalError<I, CalendarParseError<I::Slice>>,
{
    trace("lz_dec_uint", move |input: &mut I| {
        digit1
            .try_map(|s: <I as Stream>::Slice| {
                let source = I::try_into_str(&s)?;

                // the digit1 parser guarantees that `source` will be a sequence of ascii digits,
                // so the only reason that O::try_from_dec_uint could fail is if there's a mistake
                // somewhere else within this crate.

                match O::try_from_dec_uint(source.as_ref()) {
                    Some(uint) => Ok(uint),
                    None => unreachable!(),
                }
            })
            .parse_next(input)
    })
    .parse_next(input)
}

/// Returns `true` iff `s` starts with `"X-"` or `x-`.
#[inline(always)]
const fn str_has_extension_prefix(s: &str) -> bool {
    match s.as_bytes().first_chunk::<2>() {
        None => false,
        Some(prefix) => prefix.eq_ignore_ascii_case(b"X-"),
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::date;
    use crate::parser::escaped::{AsEscaped, Escaped};

    use super::*;

    #[test]
    fn status_code_parser() {
        assert_eq!(
            status_code::<_, ()>.parse_peek("3.1"),
            Ok((
                "",
                RequestStatusCode {
                    class: Class::C3,
                    major: 1,
                    minor: None
                }
            )),
        );

        assert_eq!(
            status_code::<_, ()>.parse_peek("3.1.12"),
            Ok((
                "",
                RequestStatusCode {
                    class: Class::C3,
                    major: 1,
                    minor: Some(12)
                }
            )),
        );
    }

    #[test]
    fn alarm_action_parser() {
        assert_eq!(
            alarm_action::<_, ()>.parse_peek("audio"),
            Ok(("", Token::Known(AlarmAction::Audio)))
        );

        assert_eq!(
            alarm_action::<_, ()>.parse_peek("DISPLAY"),
            Ok(("", Token::Known(AlarmAction::Display)))
        );

        assert_eq!(
            alarm_action::<_, ()>.parse_peek("Email"),
            Ok(("", Token::Known(AlarmAction::Email)))
        );

        assert_eq!(
            alarm_action::<_, ()>.parse_peek("X-extension"),
            Ok(("", Token::Unknown(Name::new("X-extension").unwrap().into())))
        );

        assert_eq!(
            alarm_action::<_, ()>.parse_peek("iana-token"),
            Ok(("", Token::Unknown(Name::new("iana-token").unwrap().into())))
        );
    }

    #[test]
    fn tz_id_parser() {
        assert!(tz_id::<_, ()>.parse_peek("/some text").is_ok());
        assert!(tz_id::<_, ()>.parse_peek("no prefix").is_ok());
    }

    #[test]
    fn time_transparency_parser() {
        assert_eq!(
            time_transparency::<_, ()>.parse_peek("opaque"),
            Ok(("", TimeTransparency::Opaque))
        );

        assert_eq!(
            time_transparency::<_, ()>.parse_peek("TRANSPARENT"),
            Ok(("", TimeTransparency::Transparent))
        );

        assert!(
            time_transparency::<_, ()>
                .parse_peek("anything else")
                .is_err()
        );
    }

    #[test]
    fn feature_type_parser() {
        assert_eq!(
            feature_type::<_, ()>.parse_peek("chat").unwrap().1,
            Token::Known(FeatureType::Chat)
        );

        assert_eq!(
            feature_type::<_, ()>.parse_peek("SCREEN").unwrap().1,
            Token::Known(FeatureType::Screen)
        );

        assert_eq!(
            feature_type::<_, ()>
                .parse_peek(Escaped("vi\r\n\tdeo".as_bytes()))
                .unwrap()
                .1,
            Token::Known(FeatureType::Video)
        );

        assert_eq!(
            feature_type::<_, ()>
                .parse_peek(Escaped("\r\n\tX-TH\r\n\tING".as_bytes()))
                .unwrap()
                .1,
            Token::Unknown(Name::new("X-THING").unwrap().into()),
        );
    }

    #[test]
    fn display_type_parser() {
        assert_eq!(
            display_type::<_, ()>.parse_peek("badge").unwrap().1,
            Token::Known(DisplayType::Badge)
        );
        assert_eq!(
            display_type::<_, ()>.parse_peek("GRAPHIC").unwrap().1,
            Token::Known(DisplayType::Graphic)
        );
        assert_eq!(
            display_type::<_, ()>.parse_peek("X-OTHER").unwrap().1,
            Token::Unknown(Name::new("X-OTHER").unwrap().into()),
        );
    }

    #[test]
    fn gregorian_parser() {
        assert!(gregorian::<_, ()>.parse_peek("GREGORIAN").is_ok());
        assert!(gregorian::<_, ()>.parse_peek("GRUGORIAN").is_err());
    }

    #[test]
    fn v2_0_parser() {
        assert!(version::<_, ()>.parse_peek("2.0").is_ok());
        assert!(version::<_, ()>.parse_peek("3.0").is_err());
    }

    #[test]
    fn method_parser() {
        assert!(method::<_, ()>.parse_peek("REFRESH").is_ok());
        assert!(method::<_, ()>.parse_peek("CANCEL").is_ok());
        assert!(method::<_, ()>.parse_peek("ADD").is_ok());
        assert!(method::<_, ()>.parse_peek("any-iana-token").is_ok());
    }

    #[test]
    fn uid_parser() {
        assert!(uid::<_, ()>.parse_peek("some random text").is_ok());
        assert!(
            uid::<_, ()>
                .parse_peek("550e8400e29b41d4a716446655440000")
                .is_ok()
        );
    }

    #[test]
    fn language_parser() {
        assert!(language::<_, ()>.parse_peek("en-US").is_ok());
        assert!(language::<_, ()>.parse_peek("de-CH").is_ok());
        assert!(language::<_, ()>.parse_peek("!!!garbage").is_err());
    }

    #[test]
    fn uri_parser() {
        // these examples are from RFC 3986 §3
        assert!(
            uri::<_, (), false>
                .parse_peek("foo://example.com:8042/over/there?name=ferret#nose")
                .is_ok()
        );
        assert!(
            uri::<_, (), false>
                .parse_peek("urn:example:animal:ferret:nose")
                .is_ok()
        );
    }

    #[test]
    fn binary_parser() {
        assert!(binary::<_, ()>.parse("AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAAACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAEREAAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_ok());

        assert!(binary::<_, ()>.parse("AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAAACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAEREAAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".as_bytes()).is_ok());

        assert!(binary::<_, ()>.parse("AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAAACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAEREAAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\r\n\tAAAAAAAAA\r\n\tAAAAAAAAAAAAAAAAAAAAAA".as_escaped()).is_ok());
    }

    #[test]
    fn class_value_parser() {
        assert_eq!(
            class_value::<_, ()>.parse_peek("CONFIDENTIAL"),
            Ok(("", Token::Known(ClassValue::Confidential)))
        );

        assert_eq!(
            class_value::<_, ()>.parse_peek("public"),
            Ok(("", Token::Known(ClassValue::Public)))
        );

        assert_eq!(
            class_value::<_, ()>.parse_peek("X-SOMETHING"),
            Ok(("", Token::Unknown(Name::new("X-SOMETHING").unwrap().into())))
        );

        assert_eq!(
            class_value::<_, ()>.parse_peek("IANA-TOKEN"),
            Ok(("", Token::Unknown(Name::new("IANA-TOKEN").unwrap().into())))
        );
    }

    #[test]
    fn calendar_user_type_parser() {
        assert_eq!(
            calendar_user_type::<_, ()>
                .parse_peek("INDIVIDUAL")
                .unwrap()
                .1,
            Token::Known(CalendarUserType::Individual),
        );

        assert_eq!(
            calendar_user_type::<_, ()>.parse_peek("room").unwrap().1,
            Token::Known(CalendarUserType::Room),
        );

        assert_eq!(
            calendar_user_type::<_, ()>
                .parse_peek("iana-token")
                .unwrap()
                .1,
            Token::Unknown(Name::new("iana-token").unwrap().into()),
        );
    }

    #[test]
    fn inline_encoding_parser() {
        assert_eq!(
            inline_encoding::<_, ()>.parse_peek("8bit"),
            inline_encoding::<_, ()>.parse_peek("8BIT"),
        );

        assert_eq!(
            inline_encoding::<_, ()>.parse_peek("Base64"),
            inline_encoding::<_, ()>.parse_peek("BASE64"),
        );

        assert!(
            inline_encoding::<_, ()>
                .parse_peek("anything_else")
                .is_err()
        );
    }

    #[test]
    fn format_type_parser() {
        assert!(
            format_type::<_, ()>
                .parse_peek("application/msword")
                .is_ok()
        );
        assert!(format_type::<_, ()>.parse_peek("image/bmp").is_ok());
        assert!(format_type::<_, ()>.parse_peek("garbage").is_err());

        assert_eq!(
            format_type::<_, ()>.parse_peek("application/postscript"),
            Ok((
                "",
                FormatType::new("application/postscript")
                    .unwrap()
                    .to_owned(),
            ))
        );
    }

    #[test]
    fn free_busy_type_parser() {
        assert_eq!(
            free_busy_type::<_, ()>.parse_peek("busy"),
            Ok(("", Token::Known(FreeBusyType::Busy)))
        );
        assert_eq!(
            free_busy_type::<_, ()>.parse_peek("Free"),
            Ok(("", Token::Known(FreeBusyType::Free)))
        );
    }

    #[test]
    fn participation_status_parser() {
        assert!(
            participation_status::<_, ()>
                .parse_peek("NEEDS-ACTION")
                .is_ok()
        );
        assert!(
            participation_status::<_, ()>
                .parse_peek("in-process")
                .is_ok()
        );
        assert!(
            participation_status::<_, ()>
                .parse_peek("some-iana-token")
                .is_ok()
        );
        assert!(
            participation_status::<_, ()>
                .parse_peek(",garbage")
                .is_err()
        );
    }

    #[test]
    fn alarm_trigger_relationship_parser() {
        assert_eq!(
            alarm_trigger_relationship::<_, ()>.parse_peek("START"),
            Ok(("", TriggerRelation::Start)),
        );

        assert_eq!(
            alarm_trigger_relationship::<_, ()>.parse_peek("END"),
            Ok(("", TriggerRelation::End)),
        );

        assert!(
            alarm_trigger_relationship::<_, ()>
                .parse_peek("anything_else")
                .is_err()
        );
    }

    #[test]
    fn relationship_type_parser() {
        assert_eq!(
            relationship_type::<_, ()>.parse_peek("SIBLING"),
            Ok(("", Token::Known(RelationshipType::Sibling))),
        );

        assert_eq!(
            relationship_type::<_, ()>.parse_peek("parent"),
            Ok(("", Token::Known(RelationshipType::Parent))),
        );

        assert_eq!(
            relationship_type::<_, ()>.parse_peek("Child"),
            Ok(("", Token::Known(RelationshipType::Child))),
        );

        assert_eq!(
            relationship_type::<_, ()>.parse_peek("X-SOMETHING-ELSE"),
            Ok((
                "",
                Token::Unknown(Name::new("X-SOMETHING-ELSE").unwrap().into())
            )),
        );
    }

    #[test]
    fn participation_role_parser() {
        assert_eq!(
            participation_role::<_, ()>.parse_peek("req-participant"),
            Ok(("", Token::Known(ParticipationRole::ReqParticipant))),
        );

        assert_eq!(
            participation_role::<_, ()>.parse_peek("Chair"),
            Ok(("", Token::Known(ParticipationRole::Chair))),
        );

        assert_eq!(
            participation_role::<_, ()>.parse_peek("X-ANYTHING"),
            Ok(("", Token::Unknown(Name::new("X-ANYTHING").unwrap().into()))),
        );
    }

    #[test]
    fn value_type_parser() {
        assert_eq!(
            value_type::<_, ()>.parse_peek("float"),
            Ok(("", Token::Known(ValueType::Float)))
        );
        assert_eq!(
            value_type::<_, ()>.parse_peek("TIME"),
            Ok(("", Token::Known(ValueType::Time)))
        );
        assert_eq!(
            value_type::<_, ()>.parse_peek("Recur"),
            Ok(("", Token::Known(ValueType::Recur)))
        );
        assert_eq!(
            value_type::<_, ()>
                .parse_peek("BOO\r\n\tLEAN".as_escaped())
                .map(|(_, v)| v),
            Ok(Token::Known(ValueType::Boolean))
        );
        assert_eq!(
            value_type::<_, ()>
                .parse_peek("\r\n X-TY\r\n\tPE".as_escaped())
                .map(|(_, v)| v),
            Ok(Token::Unknown(Name::new("X-TYPE").unwrap().into()))
        );
    }

    // TODO: add tests for the name parser

    #[test]
    fn period_parser() {
        assert!(matches!(
            period::<_, ()>.parse_peek("19970101T180000Z/19970102T070000Z"),
            Ok(("", Period::Explicit { .. })),
        ));

        assert!(matches!(
            period::<_, ()>.parse_peek("19970101T180000Z/PT5H30M"),
            Ok(("", Period::Start { .. })),
        ));
    }

    #[test]
    fn duration_parser() {
        assert_eq!(
            duration::<_, ()>.parse_peek("P7W"),
            Ok((
                "",
                SignedDuration {
                    sign: Sign::Pos,
                    duration: Duration::Nominal(NominalDuration {
                        weeks: 7,
                        days: 0,
                        exact: None
                    }),
                }
            )),
        );

        assert_eq!(
            duration::<_, ()>.parse_peek("+P15DT5H0M20S"),
            Ok((
                "",
                SignedDuration {
                    sign: Sign::Pos,
                    duration: Duration::Nominal(NominalDuration {
                        weeks: 0,
                        days: 15,
                        exact: Some(ExactDuration {
                            hours: 5,
                            minutes: 0,
                            seconds: 20,
                            frac: None,
                        }),
                    }),
                }
            )),
        );
    }

    #[test]
    fn datetime_or_date_parser() {
        assert!(
            datetime_or_date::<_, ()>
                .parse_peek("19850714")
                .is_ok_and(|(_, d)| d.is_date())
        );

        assert!(
            datetime_or_date::<_, ()>
                .parse_peek("19850714T234040")
                .is_ok_and(|(_, d)| d.is_date_time())
        );
    }

    #[test]
    fn datetime_parser() {
        assert!(datetime::<_, ()>.parse_peek("19970714T045015Z").is_ok());
        assert!(datetime::<_, ()>.parse_peek("19970714T045015").is_ok());

        assert!(
            datetime::<_, ()>
                .parse_peek("19970\r\n\t714T\r\n 045015".as_escaped())
                .is_ok_and(|(_tail, dt)| {
                    dt == DateTime {
                        date: date!(1997;7;14),
                        time: crate::time!(4;50;15),
                        marker: TimeFormat::Local,
                    }
                })
        );
    }

    #[test]
    fn datetime_utc_parser() {
        assert!(datetime_utc::<_, ()>.parse_peek("19970714T045015Z").is_ok());
        assert!(datetime_utc::<_, ()>.parse_peek("19970714T045015").is_err());
    }

    #[test]
    fn date_parser() {
        assert!(date::<_, ()>.parse_peek("19970714").is_ok());
        assert!(date::<_, ()>.parse_peek("20150229").is_err()); // 2015 is not a leap year

        assert_eq!(
            date::<_, ()>.parse_peek("20040620"),
            Ok(("", date!(2004;6;20)))
        );
    }

    #[test]
    fn time_parser() {
        assert_eq!(
            time::<_, ()>.parse_peek("111111Z").unwrap().1,
            (crate::time!(11;11;11), TimeFormat::Utc),
        );

        assert!(time::<_, ()>.parse_peek("123456").is_ok());
    }

    #[test]
    fn time_utc_parser() {
        assert!(time_utc::<_, ()>.parse_peek("202020Z").is_ok());
        assert!(time_utc::<_, ()>.parse_peek("202020").is_err());
    }

    #[test]
    fn raw_time_parser() {
        assert_eq!(
            raw_time::<_, ()>.parse_peek("123456".as_bytes()).unwrap().1,
            crate::time!(12;34;56),
        );

        assert!(raw_time::<_, ()>.parse_peek("123456").is_ok());
        assert!(raw_time::<_, ()>.parse_peek("000000").is_ok());
        assert!(raw_time::<_, ()>.parse_peek("235959").is_ok());
        assert!(raw_time::<_, ()>.parse_peek("235960").is_ok());
        assert!(raw_time::<_, ()>.parse_peek("240000").is_err());
    }

    #[test]
    fn utc_offset_parser() {
        assert_eq!(
            utc_offset::<_, ()>.parse_peek("+235959"),
            Ok(("", crate::utc_offset!(+23;59;59)))
        );

        assert_eq!(
            utc_offset::<_, ()>.parse_peek("-2340"),
            Ok(("", crate::utc_offset!(-23;40)))
        );

        assert!(utc_offset::<_, ()>.parse_peek("-0000").is_err());
        assert!(utc_offset::<_, ()>.parse_peek("-000000").is_err());
        assert!(utc_offset::<_, ()>.parse_peek("-000015").is_ok());
        assert!(utc_offset::<_, ()>.parse_peek("+000060").is_err());
        assert!(utc_offset::<_, ()>.parse_peek("+0000").is_ok());
        assert!(utc_offset::<_, ()>.parse_peek("+000000").is_ok());
        assert!(utc_offset::<_, ()>.parse_peek("000000").is_err());
    }

    #[test]
    fn time_format_parser() {
        assert_eq!(
            time_format::<_, ()>.parse_peek("Z"),
            Ok(("", TimeFormat::Utc))
        );
        assert_eq!(
            time_format::<_, ()>.parse_peek("ZZ"),
            Ok(("Z", TimeFormat::Utc))
        );
        assert_eq!(
            time_format::<_, ()>.parse_peek("Y"),
            Ok(("Y", TimeFormat::Local))
        );
    }

    #[test]
    fn geo_parser() {
        assert_eq!(
            geo::<_, ()>.parse_peek("00;00"),
            Ok(("", Geo { lat: 0.0, lon: 0.0 }))
        );

        assert_eq!(
            geo::<_, ()>.parse_peek("00;00.12345678"),
            Ok((
                "",
                Geo {
                    lat: 0.0,
                    lon: 0.12345678,
                }
            ))
        );

        assert!(geo::<_, ()>.parse_peek("90;90").is_ok());
        assert!(geo::<_, ()>.parse_peek("92;90").is_err());
        assert!(geo::<_, ()>.parse_peek("90;180").is_ok());
        assert!(geo::<_, ()>.parse_peek("90;182").is_err());
    }

    #[test]
    fn utc_marker_parser() {
        assert_eq!(utc_marker::<_, ()>.parse_peek("Z"), Ok(("", ())));
        assert!(utc_marker::<_, ()>.parse_peek("Y").is_err());
    }

    #[test]
    fn iso_week_index_parser() {
        assert_eq!(
            iso_week_index::<_, ()>.parse_peek("1"),
            Ok(("", IsoWeek::W1))
        );

        assert_eq!(
            iso_week_index::<_, ()>.parse_peek("01"),
            Ok(("", IsoWeek::W1))
        );

        assert_eq!(
            iso_week_index::<_, ()>.parse_peek("10"),
            Ok(("", IsoWeek::W10))
        );

        assert_eq!(
            iso_week_index::<_, ()>.parse_peek("53"),
            Ok(("", IsoWeek::W53))
        );

        assert!(iso_week_index::<_, ()>.parse_peek("00").is_err());
        assert!(iso_week_index::<_, ()>.parse_peek("54").is_err());
    }

    #[test]
    fn priority_parser() {
        assert_eq!(priority::<_, ()>.parse_peek("0"), Ok(("", Priority::Zero)));
        assert_eq!(priority::<_, ()>.parse_peek("1"), Ok(("", Priority::A1)));
        assert_eq!(priority::<_, ()>.parse_peek("2"), Ok(("", Priority::A2)));
        assert_eq!(priority::<_, ()>.parse_peek("3"), Ok(("", Priority::A3)));
        assert_eq!(priority::<_, ()>.parse_peek("4"), Ok(("", Priority::B1)));
        assert_eq!(priority::<_, ()>.parse_peek("5"), Ok(("", Priority::B2)));
        assert_eq!(priority::<_, ()>.parse_peek("6"), Ok(("", Priority::B3)));
        assert_eq!(priority::<_, ()>.parse_peek("7"), Ok(("", Priority::C1)));
        assert_eq!(priority::<_, ()>.parse_peek("8"), Ok(("", Priority::C2)));
        assert_eq!(priority::<_, ()>.parse_peek("9"), Ok(("", Priority::C3)));
        assert!(priority::<_, ()>.parse_peek("10").is_err());
    }

    #[test]
    fn bool_parser() {
        assert_eq!(bool_caseless::<_, ()>.parse_peek("TRUE"), Ok(("", true)));
        assert_eq!(bool_caseless::<_, ()>.parse_peek("FALSE"), Ok(("", false)));
        assert_eq!(bool_caseless::<_, ()>.parse_peek("True"), Ok(("", true)));
        assert_eq!(bool_caseless::<_, ()>.parse_peek("False"), Ok(("", false)));
        assert_eq!(bool_caseless::<_, ()>.parse_peek("true"), Ok(("", true)));
        assert_eq!(bool_caseless::<_, ()>.parse_peek("false"), Ok(("", false)));

        assert_eq!(
            bool_caseless::<_, ()>.parse_peek(Escaped("tr\r\n\tue".as_bytes())),
            Ok(("".as_escaped(), true))
        );

        assert_eq!(
            bool_caseless::<_, ()>.parse_peek(Escaped("fals\r\n\te".as_bytes())),
            Ok(("".as_escaped(), false))
        );
    }

    #[test]
    fn integer_parser() {
        assert_eq!(integer::<_, ()>.parse_peek("370"), Ok(("", 370)));
        assert_eq!(integer::<_, ()>.parse_peek("-17"), Ok(("", -17)));
        assert_eq!(
            integer::<_, ()>.parse_peek("2147483647"),
            Ok(("", Integer::MAX))
        );
        assert_eq!(
            integer::<_, ()>.parse_peek("-2147483648"),
            Ok(("", Integer::MIN))
        );
        assert!(integer::<_, ()>.parse_peek("2147483648").is_err());
    }

    #[test]
    fn float_parser() {
        assert_eq!(
            float::<_, ()>.parse_peek("1000000.0000001"),
            Ok(("", 1000000.0000001)),
        );

        assert_eq!(
            float::<_, ()>.parse_peek("1000\r\n\t000.00\r\n 00001".as_escaped()),
            Ok(("".as_escaped(), 1000000.0000001)),
        );

        assert_eq!(float::<_, ()>.parse_peek("1.333"), Ok(("", 1.333)));
        assert_eq!(float::<_, ()>.parse_peek("-3.15"), Ok(("", -3.15)));
        assert_eq!(float::<_, ()>.parse_peek("12."), Ok((".", 12.)));
        assert!(float::<_, ()>.parse_peek("+.002").is_err());
    }

    #[test]
    fn sign_parser() {
        assert_eq!(sign::<_, ()>.parse_peek("+"), Ok(("", Sign::Pos)));
        assert_eq!(sign::<_, ()>.parse_peek("-"), Ok(("", Sign::Neg)));
        assert!(sign::<_, ()>.parse_peek("0").is_err());

        assert_eq!(
            sign::<_, ()>.parse_peek(Escaped("\r\n\t+".as_bytes())),
            Ok((Escaped("".as_bytes()), Sign::Pos))
        );

        assert_eq!(
            sign::<_, ()>.parse_peek(Escaped("\r\n -".as_bytes())),
            Ok((Escaped("".as_bytes()), Sign::Neg))
        );
    }

    #[test]
    fn digit_parser() {
        assert_eq!(digit::<_, (), 10>.parse_peek("0"), Ok(("", 0)));
        assert_eq!(digit::<_, (), 10>.parse_peek("1"), Ok(("", 1)));
        assert_eq!(digit::<_, (), 10>.parse_peek("2"), Ok(("", 2)));
        // ...
        assert_eq!(digit::<_, (), 10>.parse_peek("8"), Ok(("", 8)));
        assert_eq!(digit::<_, (), 10>.parse_peek("9"), Ok(("", 9)));

        assert!(digit::<_, (), 10>.parse_peek("A").is_err());
        assert!(digit::<_, (), 16>.parse_peek("A").is_ok());
    }

    #[test]
    fn color_parser() {
        for c in Css3Color::iter() {
            dbg![c];
            let input = c.to_string();
            let (tail, res) = color::<_, ()>.parse_peek(input.as_str()).unwrap();
            dbg![tail];
            assert!(tail.is_empty());
            assert_eq!(c, res);
        }
    }
}
