//! Property parameter types for the object model.
//!
//! # Value Types
//! The VALUE parameter denotes the value type of the property on which it occurs. When it is
//! absent, this is because the relevant property has a default value type; hence we can say that
//! all properties always have a value type. Moreover, since it governs the allowed property value,
//! it can always be inferred by examining the property value.
//!
//! It would be redundant to retain the value type as a parameter when it is already present as
//! part of a property value, and so [`Params`] and its variants do not include the value type.
//!
//! # ORDER
//! RFC 9073 §5.1 introduced the ORDER property parameter, with text providing that it "MUST NOT be
//! applied to a property that does not allow multiple instances." As such, we are forced to draw
//! a distinction between the parameters of properties which can occur at most once and those of
//! properties which may occur an arbitrary number of times.
//!
//! This also introduces some ambiguity. Consider the ATTACH property (RFC 5545 §3.8.1.1), which
//! may occur several times in multiple components but only once in VALARM components with the
//! AUDIO action. So how should we handle this? The language of RFC 9073 seems to indicate that the
//! permissibility of ORDER is decided on a per-property basis, but we cannot decide whether ORDER
//! is permissible in this case unless we know the enclosing component. This problem applies in
//! general to any property whose multiplicity depends on the component it occurs within.
//!
//! To sidestep this issue entirely, as well as to make the overall parser more robust, we admit
//! the ORDER parameter for all properties. It is the business of the renderer to decide how ORDER
//! should be handled when it appears in an impermissible context.
//!
//! # Multiplicity
//! Whereas some properties (e.g. CATEGORIES, RFC 5545 §3.8.1.2) may appear multiple times on the
//! same component, no property parameters have ever been defined with the ability to appear more
//! than once on the same property. This is perfectly fine for the statically-known parameters,
//! but for unknown IANA-registered and extension parameters it poses a problem: how should we
//! handle instances where the same parameter name appears twice on a given property?
//!
//! More concretely, consider an example like `DTSTART;X-FOO=foo;X-FOO=bar:20081006`. This is
//! well-formed with respect to the grammar defined by RFC 5545,[^rfc-5545-intention] but it isn't
//! clear what it *means* with respect to the iCalendar object model. Should it be equivalent to a
//! comma-separated list (i.e. `X-FOO=foo,bar`), or should one of the values take precedence over
//! the other? If so, which one?
//!
//! [`ical.js`](https://github.com/kewisch/ical.js) uses a precedence system, always taking the
//! last occurrence as the actual value. It's a little harder to determine what
//! [`libical`](https://github.com/libical/libical) does, but we can notice that the
//! data in `design-data/ical-parameters.csv` marks "X" parameters as not being multivalued (in
//! contrast to parameters like MEMBER, whch have the `is_multivalued` flag set); the same is also
//! true for "IANA" parameters.
//!
//! TODO: explain what calico actually does here
//!
//! [^rfc-5545-intention]: The relevant grammar fragment from RFC 5545 is almost always rendered as
//! `*(";" other-param)` where `other-param` encompasses the rules for IANA and extension
//! parameters. I suspect that this was intended to neatly admit repetitions of parameters with
//! *distinct* names, but it also obviously allows the same parameter to occur several times.

use mitsein::vec1::Vec1;
use structible::structible;

use super::{
    primitive::{
        CalendarUserType, DisplayType, Encoding, FeatureType, FormatTypeBuf, FreeBusyType,
        Language, ParticipationRole, ParticipationStatus, PositiveInteger, RelationshipType,
        ThisAndFuture, Token, TriggerRelation, ValueType,
    },
    string::{CaselessStr, Name, NameKind, ParamValue, TzId, Uri},
};

// TODO: RFC 5545 §3.2.19 (Time Zone Identifier) says that the TZID parameter MUST be specified on
// the DTSTART, DTEND, DUE, EXDATE, and RDATE properties given certain conditions on their value
// types. should that be encoded here? and more generally, am i missing more cases like this?

/// A table of optional property parameters.
#[structible]
pub struct Params {
    // RFC 5545
    pub alternate_representation: Option<Box<Uri>>,
    pub common_name: Option<Box<ParamValue>>,
    pub calendar_user_type: Option<Token<CalendarUserType, Box<Name>>>,
    pub delegated_from: Option<Vec1<Box<Uri>>>,
    pub delegated_to: Option<Vec1<Box<Uri>>>,
    pub directory_reference: Option<Box<Uri>>,
    pub inline_encoding: Option<Encoding>,
    pub format_type: Option<FormatTypeBuf>,
    pub free_busy_type: Option<Token<FreeBusyType, Box<Name>>>,
    pub language: Option<Language>,
    pub membership: Option<Vec1<Box<Uri>>>,
    pub participation_status: Option<Token<ParticipationStatus, Box<Name>>>,
    pub recurrence_range: Option<ThisAndFuture>,
    pub trigger_relationship: Option<TriggerRelation>,
    pub relationship_type: Option<Token<RelationshipType, Box<Name>>>,
    pub participation_role: Option<Token<ParticipationRole, Box<Name>>>,
    pub rsvp_expectation: Option<bool>,
    pub sent_by: Option<Box<Uri>>,
    pub tz_id: Option<Box<TzId>>,

    // RFC 7986
    pub display_type: Option<Token<DisplayType, Box<Name>>>,
    pub email: Option<Box<ParamValue>>,
    pub feature_type: Option<Token<FeatureType, Box<Name>>>,
    pub label: Option<Box<ParamValue>>,

    // RFC 9073
    pub order: Option<PositiveInteger>,
    pub schema: Option<Box<Uri>>,
    pub derived: Option<bool>,

    // Unknown parameters
    #[structible(key = Box<CaselessStr>)]
    pub unknown_param: Option<Vec1<Box<ParamValue>>>,
}

impl Eq for Params {}

impl Params {
    /// Inserts a known parameter, overwriting any previous value.
    pub fn insert_known(&mut self, param: KnownParam) {
        match param {
            KnownParam::AltRep(v) => { self.set_alternate_representation(v); }
            KnownParam::CommonName(v) => { self.set_common_name(v); }
            KnownParam::CUType(v) => { self.set_calendar_user_type(v); }
            KnownParam::DelFrom(v) => { self.set_delegated_from(v); }
            KnownParam::DelTo(v) => { self.set_delegated_to(v); }
            KnownParam::Dir(v) => { self.set_directory_reference(v); }
            KnownParam::Encoding(v) => { self.set_inline_encoding(v); }
            KnownParam::FormatType(v) => { self.set_format_type(v); }
            KnownParam::FBType(v) => { self.set_free_busy_type(v); }
            KnownParam::Language(v) => { self.set_language(v); }
            KnownParam::Member(v) => { self.set_membership(v); }
            KnownParam::PartStatus(v) => { self.set_participation_status(v); }
            KnownParam::RecurrenceIdentifierRange => { self.set_recurrence_range(ThisAndFuture); }
            KnownParam::AlarmTrigger(v) => { self.set_trigger_relationship(v); }
            KnownParam::RelType(v) => { self.set_relationship_type(v); }
            KnownParam::Role(v) => { self.set_participation_role(v); }
            KnownParam::Rsvp(v) => { self.set_rsvp_expectation(v); }
            KnownParam::SentBy(v) => { self.set_sent_by(v); }
            KnownParam::TzId(v) => { self.set_tz_id(v); }
            KnownParam::Value(_) => { /* VALUE is not stored in Params */ }
            KnownParam::Display(v) => { self.set_display_type(v); }
            KnownParam::Email(v) => { self.set_email(v); }
            KnownParam::Feature(v) => { self.set_feature_type(v); }
            KnownParam::Label(v) => { self.set_label(v); }
            KnownParam::Order(v) => { self.set_order(v); }
            KnownParam::Schema(v) => { self.set_schema(v); }
            KnownParam::Derived(v) => { self.set_derived(v); }
        }
    }

    /// Returns `true` if the given known parameter is set.
    pub fn contains_known(&self, key: StaticParam) -> bool {
        match key {
            StaticParam::AltRep => self.alternate_representation().is_some(),
            StaticParam::CommonName => self.common_name().is_some(),
            StaticParam::CalUserType => self.calendar_user_type().is_some(),
            StaticParam::DelFrom => self.delegated_from().is_some(),
            StaticParam::DelTo => self.delegated_to().is_some(),
            StaticParam::Dir => self.directory_reference().is_some(),
            StaticParam::Encoding => self.inline_encoding().is_some(),
            StaticParam::FormatType => self.format_type().is_some(),
            StaticParam::FreeBusyType => self.free_busy_type().is_some(),
            StaticParam::Language => self.language().is_some(),
            StaticParam::Member => self.membership().is_some(),
            StaticParam::PartStat => self.participation_status().is_some(),
            StaticParam::Range => self.recurrence_range().is_some(),
            StaticParam::Related => self.trigger_relationship().is_some(),
            StaticParam::RelType => self.relationship_type().is_some(),
            StaticParam::Role => self.participation_role().is_some(),
            StaticParam::Rsvp => self.rsvp_expectation().is_some(),
            StaticParam::SentBy => self.sent_by().is_some(),
            StaticParam::TzId => self.tz_id().is_some(),
            StaticParam::Value => false, // VALUE is not stored in Params
            StaticParam::Display => self.display_type().is_some(),
            StaticParam::Email => self.email().is_some(),
            StaticParam::Feature => self.feature_type().is_some(),
            StaticParam::Label => self.label().is_some(),
            StaticParam::Order => self.order().is_some(),
            StaticParam::Schema => self.schema().is_some(),
            StaticParam::Derived => self.derived().is_some(),
        }
    }
}

/// The parameters of the STRUCTURED-DATA property when its value type is TEXT or BINARY. The
/// `format_type` and `schema` fields are mandatory, while all other fields are optional.
#[structible]
pub struct StructuredDataParams {
    // Required
    pub format_type: FormatTypeBuf,
    pub schema: Box<Uri>,

    // RFC 5545
    pub alternate_representation: Option<Box<Uri>>,
    pub common_name: Option<Box<ParamValue>>,
    pub calendar_user_type: Option<Token<CalendarUserType, Box<Name>>>,
    pub delegated_from: Option<Vec1<Box<Uri>>>,
    pub delegated_to: Option<Vec1<Box<Uri>>>,
    pub directory_reference: Option<Box<Uri>>,
    pub inline_encoding: Option<Encoding>,
    pub free_busy_type: Option<Token<FreeBusyType, Box<Name>>>,
    pub language: Option<Language>,
    pub membership: Option<Vec1<Box<Uri>>>,
    pub participation_status: Option<Token<ParticipationStatus, Box<Name>>>,
    pub recurrence_range: Option<ThisAndFuture>,
    pub trigger_relationship: Option<TriggerRelation>,
    pub relationship_type: Option<Token<RelationshipType, Box<Name>>>,
    pub participation_role: Option<Token<ParticipationRole, Box<Name>>>,
    pub rsvp_expectation: Option<bool>,
    pub sent_by: Option<Box<Uri>>,
    pub tz_id: Option<Box<TzId>>,

    // RFC 7986
    pub display_type: Option<Token<DisplayType, Box<Name>>>,
    pub email: Option<Box<ParamValue>>,
    pub feature_type: Option<Token<FeatureType, Box<Name>>>,
    pub label: Option<Box<ParamValue>>,

    // RFC 9073
    pub order: Option<PositiveInteger>,
    pub derived: Option<bool>,

    // Unknown parameters
    #[structible(key = Box<CaselessStr>)]
    pub unknown_param: Option<Vec1<Box<ParamValue>>>,
}

impl Eq for StructuredDataParams {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SDParamsFromParamsError {
    MissingFormatType,
    MissingSchema,
    MissingFormatTypeAndSchema,
}

impl TryFrom<Params> for StructuredDataParams {
    type Error = SDParamsFromParamsError;

    fn try_from(value: Params) -> Result<Self, Self::Error> {
        let has_format_type = value.format_type().is_some();
        let has_schema = value.schema().is_some();

        match (has_format_type, has_schema) {
            (false, false) => Err(SDParamsFromParamsError::MissingFormatTypeAndSchema),
            (false, true) => Err(SDParamsFromParamsError::MissingFormatType),
            (true, false) => Err(SDParamsFromParamsError::MissingSchema),
            (true, true) => {
                let mut fields = value.into_fields();
                let format_type = fields.take_format_type().unwrap();
                let schema = fields.take_schema().unwrap();
                let mut result = StructuredDataParams::new(format_type, schema);

                if let Some(v) = fields.take_alternate_representation() { result.set_alternate_representation(v); }
                if let Some(v) = fields.take_common_name() { result.set_common_name(v); }
                if let Some(v) = fields.take_calendar_user_type() { result.set_calendar_user_type(v); }
                if let Some(v) = fields.take_delegated_from() { result.set_delegated_from(v); }
                if let Some(v) = fields.take_delegated_to() { result.set_delegated_to(v); }
                if let Some(v) = fields.take_directory_reference() { result.set_directory_reference(v); }
                if let Some(v) = fields.take_inline_encoding() { result.set_inline_encoding(v); }
                if let Some(v) = fields.take_free_busy_type() { result.set_free_busy_type(v); }
                if let Some(v) = fields.take_language() { result.set_language(v); }
                if let Some(v) = fields.take_membership() { result.set_membership(v); }
                if let Some(v) = fields.take_participation_status() { result.set_participation_status(v); }
                if let Some(v) = fields.take_recurrence_range() { result.set_recurrence_range(v); }
                if let Some(v) = fields.take_trigger_relationship() { result.set_trigger_relationship(v); }
                if let Some(v) = fields.take_relationship_type() { result.set_relationship_type(v); }
                if let Some(v) = fields.take_participation_role() { result.set_participation_role(v); }
                if let Some(v) = fields.take_rsvp_expectation() { result.set_rsvp_expectation(v); }
                if let Some(v) = fields.take_sent_by() { result.set_sent_by(v); }
                if let Some(v) = fields.take_tz_id() { result.set_tz_id(v); }
                if let Some(v) = fields.take_display_type() { result.set_display_type(v); }
                if let Some(v) = fields.take_email() { result.set_email(v); }
                if let Some(v) = fields.take_feature_type() { result.set_feature_type(v); }
                if let Some(v) = fields.take_label() { result.set_label(v); }
                if let Some(v) = fields.take_order() { result.set_order(v); }
                if let Some(v) = fields.take_derived() { result.set_derived(v); }

                Ok(result)
            }
        }
    }
}

impl From<StructuredDataParams> for Params {
    fn from(value: StructuredDataParams) -> Self {
        let mut fields = value.into_fields();
        let mut result = Params::new();

        result.set_format_type(fields.take_format_type().unwrap());
        result.set_schema(fields.take_schema().unwrap());

        if let Some(v) = fields.take_alternate_representation() { result.set_alternate_representation(v); }
        if let Some(v) = fields.take_common_name() { result.set_common_name(v); }
        if let Some(v) = fields.take_calendar_user_type() { result.set_calendar_user_type(v); }
        if let Some(v) = fields.take_delegated_from() { result.set_delegated_from(v); }
        if let Some(v) = fields.take_delegated_to() { result.set_delegated_to(v); }
        if let Some(v) = fields.take_directory_reference() { result.set_directory_reference(v); }
        if let Some(v) = fields.take_inline_encoding() { result.set_inline_encoding(v); }
        if let Some(v) = fields.take_free_busy_type() { result.set_free_busy_type(v); }
        if let Some(v) = fields.take_language() { result.set_language(v); }
        if let Some(v) = fields.take_membership() { result.set_membership(v); }
        if let Some(v) = fields.take_participation_status() { result.set_participation_status(v); }
        if let Some(v) = fields.take_recurrence_range() { result.set_recurrence_range(v); }
        if let Some(v) = fields.take_trigger_relationship() { result.set_trigger_relationship(v); }
        if let Some(v) = fields.take_relationship_type() { result.set_relationship_type(v); }
        if let Some(v) = fields.take_participation_role() { result.set_participation_role(v); }
        if let Some(v) = fields.take_rsvp_expectation() { result.set_rsvp_expectation(v); }
        if let Some(v) = fields.take_sent_by() { result.set_sent_by(v); }
        if let Some(v) = fields.take_tz_id() { result.set_tz_id(v); }
        if let Some(v) = fields.take_display_type() { result.set_display_type(v); }
        if let Some(v) = fields.take_email() { result.set_email(v); }
        if let Some(v) = fields.take_feature_type() { result.set_feature_type(v); }
        if let Some(v) = fields.take_label() { result.set_label(v); }
        if let Some(v) = fields.take_order() { result.set_order(v); }
        if let Some(v) = fields.take_derived() { result.set_derived(v); }

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    Known(KnownParam),
    Unknown(UnknownParam),
}

impl Param {
    pub fn try_into_unknown(self) -> Result<UnknownParam, Self> {
        if let Self::Unknown(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_known(self) -> Result<KnownParam, Self> {
        if let Self::Known(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpcastParamValue {
    ValueType(Token<ValueType, Box<Name>>),
    Known(KnownParam),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownParam {
    // RFC 5545 PROPERTY PARAMETERS
    AltRep(Box<Uri>),
    CommonName(Box<ParamValue>),
    CUType(Token<CalendarUserType, Box<Name>>),
    DelFrom(Vec1<Box<Uri>>),
    DelTo(Vec1<Box<Uri>>),
    Dir(Box<Uri>),
    Encoding(Encoding),
    FormatType(FormatTypeBuf),
    FBType(Token<FreeBusyType, Box<Name>>),
    Language(Language),
    Member(Vec1<Box<Uri>>),
    PartStatus(Token<ParticipationStatus, Box<Name>>),
    RecurrenceIdentifierRange,
    AlarmTrigger(TriggerRelation),
    RelType(Token<RelationshipType, Box<Name>>),
    Role(Token<ParticipationRole, Box<Name>>),
    Rsvp(bool),
    SentBy(Box<Uri>),
    TzId(Box<TzId>),
    Value(Token<ValueType, Box<Name>>),
    // RFC 7986 PROPERTY PARAMETERS
    Display(Token<DisplayType, Box<Name>>),
    Email(Box<ParamValue>),
    Feature(Token<FeatureType, Box<Name>>),
    Label(Box<ParamValue>),
    // RFC 9073 PROPERTY PARAMETERS
    Order(PositiveInteger),
    Schema(Box<Uri>),
    Derived(bool),
}

impl KnownParam {
    pub const fn name(&self) -> StaticParam {
        match self {
            KnownParam::AltRep(_) => StaticParam::AltRep,
            KnownParam::CommonName(_) => StaticParam::CommonName,
            KnownParam::CUType(_) => StaticParam::CalUserType,
            KnownParam::DelFrom(_) => StaticParam::DelFrom,
            KnownParam::DelTo(_) => StaticParam::DelTo,
            KnownParam::Dir(_) => StaticParam::Dir,
            KnownParam::Encoding(_) => StaticParam::Encoding,
            KnownParam::FormatType(_) => StaticParam::FormatType,
            KnownParam::FBType(_) => StaticParam::FreeBusyType,
            KnownParam::Language(_) => StaticParam::Language,
            KnownParam::Member(_) => StaticParam::Member,
            KnownParam::PartStatus(_) => StaticParam::PartStat,
            KnownParam::RecurrenceIdentifierRange => StaticParam::Range,
            KnownParam::AlarmTrigger(_) => StaticParam::Related,
            KnownParam::RelType(_) => StaticParam::RelType,
            KnownParam::Role(_) => StaticParam::Role,
            KnownParam::Rsvp(_) => StaticParam::Rsvp,
            KnownParam::SentBy(_) => StaticParam::SentBy,
            KnownParam::TzId(_) => StaticParam::TzId,
            KnownParam::Value(_) => StaticParam::Value,
            KnownParam::Display(_) => StaticParam::Display,
            KnownParam::Email(_) => StaticParam::Email,
            KnownParam::Feature(_) => StaticParam::Feature,
            KnownParam::Label(_) => StaticParam::Label,
            KnownParam::Order(_) => StaticParam::Order,
            KnownParam::Schema(_) => StaticParam::Schema,
            KnownParam::Derived(_) => StaticParam::Derived,
        }
    }

    pub fn upcast(self) -> UpcastParamValue {
        match self {
            KnownParam::Value(value_type) => UpcastParamValue::ValueType(value_type),
            other => UpcastParamValue::Known(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownParam {
    pub name: Box<Name>,
    pub value: UnknownParamValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownParamValue {
    pub kind: NameKind,
    pub values: Vec1<Box<ParamValue>>,
}

/// A statically known parameter name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StaticParam {
    // RFC 5545
    /// RFC 5545 §3.2.1 (ALTREP)
    AltRep,
    /// RFC 5545 §3.2.2 (CN)
    CommonName,
    /// RFC 5545 §3.2.3 (CUTYPE)
    CalUserType,
    /// RFC 5545 §3.2.4 (DELEGATED-FROM)
    DelFrom,
    /// RFC 5545 §3.2.5 (DELEGATED-TO)
    DelTo,
    /// RFC 5545 §3.2.6 (DIR)
    Dir,
    /// RFC 5545 §3.2.7 (ENCODING)
    Encoding,
    /// RFC 5545 §3.2.8 (FMTTYPE)
    FormatType,
    /// RFC 5545 §3.2.9 (FBTYPE)
    FreeBusyType,
    /// RFC 5545 §3.2.10 (LANGUAGE)
    Language,
    /// RFC 5545 §3.2.11 (MEMBER)
    Member,
    /// RFC 5545 §3.2.12 (PARTSTAT)
    PartStat,
    /// RFC 5545 §3.2.13 (RANGE)
    Range,
    /// RFC 5545 §3.2.14 (RELATED)
    Related,
    /// RFC 5545 §3.2.15 (RELTYPE)
    RelType,
    /// RFC 5545 §3.2.16 (ROLE)
    Role,
    /// RFC 5545 §3.2.17 (RSVP)
    Rsvp,
    /// RFC 5545 §3.2.18 (SENT-BY)
    SentBy,
    /// RFC 5545 §3.2.19 (TZID)
    TzId,
    /// RFC 5545 §3.2.20 (VALUE)
    Value,

    // RFC 7986
    /// RFC 7986 §6.1 (DISPLAY)
    Display,
    /// RFC 7986 §6.2 (EMAIL)
    Email,
    /// RFC 7986 §6.3 (FEATURE)
    Feature,
    /// RFC 7986 §6.4 (LABEL)
    Label,

    // RFC 9073
    /// RFC 9073 §5.1 (ORDER)
    Order,
    /// RFC 9073 §5.2 (SCHEMA)
    Schema,
    /// RFC 9073 §5.3 (DERIVED)
    Derived,
}

/// A property parameter name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamName<S> {
    Known(StaticParam),
    Unknown(S),
}

#[cfg(test)]
mod tests {
    use mitsein::vec1;

    use super::*;

    #[test]
    fn simple_any_params_usage() {
        let mut params = Params::default();
        params.set_rsvp_expectation(true);
        params.set_inline_encoding(Encoding::Base64);
        params.insert_unknown_param(
            CaselessStr::from_box_str("X-FOO".into()),
            vec1![ParamValue::new("bar").unwrap().into()],
        );

        assert_eq!(params.rsvp_expectation(), Some(&true));
        assert_eq!(params.inline_encoding(), Some(&Encoding::Base64));
        assert_eq!(
            params.unknown_param(CaselessStr::new("x-foo")),
            Some(&vec1![ParamValue::new("bar").unwrap().into()]),
        );

        assert!(params.alternate_representation().is_none());
        assert!(params.common_name().is_none());
        assert!(params.tz_id().is_none());
    }
}
