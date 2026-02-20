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

use std::hash::Hash;

use hashbrown::{Equivalent, HashMap};
use mitsein::{slice1::Slice1, vec1::Vec1};
use paste::paste;

use super::{
    primitive::{
        CalendarUserType, DisplayType, Encoding, FeatureType, FormatType, FreeBusyType, Language,
        ParticipationRole, ParticipationStatus, PositiveInteger, RelationshipType, ThisAndFuture,
        TriggerRelation, ValueType,
    },
    string::{CaselessStr, Name, NameKind, NeverStr, ParamValue, TzId, Uri},
};

// TODO: RFC 5545 §3.2.19 (Time Zone Identifier) says that the TZID parameter MUST be specified on
// the DTSTART, DTEND, DUE, EXDATE, and RDATE properties given certain conditions on their value
// types. should that be encoded here? and more generally, am i missing more cases like this?

macro_rules! define_parameter_type {
    ($(#[$m:meta])* $v:vis $name:ident { $($tail:tt)* }) => {
        $(#[$m])*
        $v struct $name(ParameterTable);

        impl std::ops::Deref for $name {
            type Target = ParameterTable;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $name {
            define_methods!({$($tail)*});
        }
    };

}

/// A helper macro for [`define_parameter_type`] that creates accessor methods for each of "fields."
/// Each field has the form `(name, key) <sep> ty` where <sep> may either be `?` or `!`. If it is
/// `?`, then the field is _optional_; if it is `!` then the field is _mandatory_. Aside from the
/// difference in return types, the only notable difference is that mandatory fields cannot be
/// removed, only modified or replaced.
macro_rules! define_methods {
    // optional
    ({$(#[$m:meta])* ($name:ident, $key:ident) ? $ret_ty:ty $(, $($tail:tt)*)?}) => {
        paste! {
            $(#[$m])*
            pub fn $name(&self) -> Option<&$ret_ty> {
                self.get_known(StaticParam::$key)
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
            }

            $(#[$m])*
            pub fn [<$name _mut>](&mut self) -> Option<&mut $ret_ty> {
                self.get_known_mut(StaticParam::$key)
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
            }

            $(#[$m])*
            pub fn [<remove_ $name>](&mut self) -> Option<$ret_ty> {
                self.0.remove_known(StaticParam::$key)
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
            }

            $(#[$m])*
            pub fn [<set_ $name>](&mut self, value: $ret_ty) -> Option<$ret_ty> {
                self.0
                    .insert(ParamName::Known(StaticParam::$key), AnyParamValue::from(value))
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
            }
        }

        $(define_methods!({$($tail)*});)?
    };
    // required
    ({$(#[$m:meta])* ($name:ident, $key:ident) ! $ret_ty:ty $(, $($tail:tt)*)?}) => {
        paste! {
            $(#[$m])*
            pub fn $name(&self) -> &$ret_ty {
                self.get_known(StaticParam::$key)
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
                    .unwrap()
            }

            $(#[$m])*
            pub fn [<$name _mut>](&mut self) -> &mut $ret_ty {
                self.get_known_mut(StaticParam::$key)
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
                    .unwrap()
            }

            $(#[$m])*
            pub fn [<replace_ $name>](&mut self, value: $ret_ty) -> $ret_ty {
                self.0
                    .insert(ParamName::Known(StaticParam::$key), AnyParamValue::from(value))
                    .map(|raw_value| raw_value.try_into().ok().unwrap())
                    .unwrap()
            }
        }

        $(define_methods!({$($tail)*});)?
    };
    ({$(,)?}) => {};
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParameterTable(HashMap<ParamName<Box<CaselessStr>>, AnyParamValue>);

impl ParameterTable {
    // public APIs accessible via deref coercion

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn contains_known(&self, key: StaticParam) -> bool {
        self.0.contains_key(&ParamName::<NeverStr>::Known(key))
    }

    pub fn contains_unknown(&self, key: &str) -> bool {
        let key: ParamName<&CaselessStr> = ParamName::Unknown(key.into());
        self.0.contains_key(&key)
    }

    pub fn get_known(&self, key: StaticParam) -> Option<&AnyParamValue> {
        self.0.get(&ParamName::<NeverStr>::Known(key))
    }

    pub fn get_known_mut(&mut self, key: StaticParam) -> Option<&mut AnyParamValue> {
        self.0.get_mut(&ParamName::<NeverStr>::Known(key))
    }

    pub fn get_unknown(&self, key: &str) -> Option<&Slice1<Box<ParamValue>>> {
        let key: ParamName<&CaselessStr> = ParamName::Unknown(key.into());
        let value = self.0.get(&key);

        value.map(|value| {
            let vec: &Vec1<Box<ParamValue>> = value
                .try_into()
                .expect("unknown parameters must be param value vectors");

            vec.as_slice1()
        })
    }

    pub fn get_unknown_mut(&mut self, key: &str) -> Option<&mut Vec1<Box<ParamValue>>> {
        let key: ParamName<&CaselessStr> = ParamName::Unknown(key.into());
        let value = self.0.get_mut(&key);

        value.map(|value| {
            value
                .try_into()
                .expect("unknown parameters must be param value vectors")
        })
    }

    pub fn insert_unknown(
        &mut self,
        key: impl Into<Box<str>>,
        value: Vec1<Box<ParamValue>>,
    ) -> Option<Vec1<Box<ParamValue>>> {
        let key = ParamName::Unknown(CaselessStr::from_box_str(key.into()));

        self.0.insert(key, AnyParamValue::from(value)).map(|value| {
            value
                .try_into()
                .expect("unknown parameters must be param value vectors")
        })
    }

    pub fn remove_unknown(&mut self, key: &str) -> Option<Vec1<Box<ParamValue>>> {
        let key: ParamName<&CaselessStr> = ParamName::Unknown(key.into());

        self.0.remove(&key).map(|value| {
            value
                .try_into()
                .expect("unknown parameters must be param value vectors")
        })
    }

    // private APIs that should not be "inherited"

    fn insert(
        &mut self,
        key: ParamName<Box<CaselessStr>>,
        value: AnyParamValue,
    ) -> Option<AnyParamValue> {
        self.0.insert(key, value)
    }

    fn remove_known(&mut self, key: StaticParam) -> Option<AnyParamValue> {
        self.0.remove(&ParamName::<NeverStr>::Known(key))
    }
}

define_parameter_type! {
    /// The parameters of the STRUCTURED-DATA property when its value type is TEXT or BINARY. The
    /// `format_type` and `schema` fields are mandatory, while all other fields are optional.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub StructuredDataParams {
        // RFC 5545
        (alternate_representation, AltRep) ? Box<Uri>,
        (common_name, CommonName) ? Box<ParamValue>,
        (calendar_user_type, CalUserType) ? CalendarUserType<Box<Name>>,
        (delegated_from, DelFrom) ? Vec1<Box<Uri>>,
        (delegated_to, DelTo) ? Vec1<Box<Uri>>,
        (directory_reference, Dir) ? Box<Uri>,
        (inline_encoding, Encoding) ? Encoding,
        (format_type, FormatType) ! FormatType,
        (free_busy_type, FreeBusyType) ? FreeBusyType<Box<Name>>,
        (language, Language) ? Language,
        (membership, Member) ? Vec1<Box<Uri>>,
        (participation_status, PartStat) ? ParticipationStatus<Box<Name>>,
        (recurrence_range, Range) ? ThisAndFuture,
        (trigger_relationship, Related) ? TriggerRelation,
        (relationship_type, RelType) ? RelationshipType<Box<Name>>,
        (participation_role, Role) ? ParticipationRole<Box<Name>>,
        (rsvp_expectation, Rsvp) ? bool,
        (sent_by, SentBy) ? Box<Uri>,
        (tz_id, TzId) ? Box<TzId>,

        // RFC 7986
        (display_type, Display) ? DisplayType<Box<Name>>,
        (email, Email) ? Box<ParamValue>,
        (feature_type, Feature) ? FeatureType<Box<Name>>,
        (label, Label) ? Box<ParamValue>,

        // RFC 9073
        (order, Order) ? PositiveInteger,
        (schema, Schema) ! Box<Uri>,
        (derived, Derived) ? bool,
    }
}

impl StructuredDataParams {
    pub fn new(format_type: FormatType, schema: Box<Uri>) -> Self {
        let mut params = Params::with_capacity(2);
        params.set_format_type(format_type);
        params.set_schema(schema);
        Self(params.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SDParamsFromParamsError {
    MissingFormatType,
    MissingSchema,
    MissingFormatTypeAndSchema,
}

impl TryFrom<Params> for StructuredDataParams {
    type Error = SDParamsFromParamsError;

    fn try_from(value: Params) -> Result<Self, Self::Error> {
        let format_type = value.contains_known(StaticParam::FormatType);
        let schema = value.contains_known(StaticParam::Schema);

        match (format_type, schema) {
            (false, false) => Err(SDParamsFromParamsError::MissingFormatTypeAndSchema),
            (false, true) => Err(SDParamsFromParamsError::MissingFormatType),
            (true, false) => Err(SDParamsFromParamsError::MissingSchema),
            (true, true) => Ok(Self(value.0)),
        }
    }
}

define_parameter_type! {
    /// A table of optional property parameters.
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub Params {
        // RFC 5545
        (alternate_representation, AltRep) ? Box<Uri>,
        (common_name, CommonName) ? Box<ParamValue>,
        (calendar_user_type, CalUserType) ? CalendarUserType<Box<Name>>,
        (delegated_from, DelFrom) ? Vec1<Box<Uri>>,
        (delegated_to, DelTo) ? Vec1<Box<Uri>>,
        (directory_reference, Dir) ? Box<Uri>,
        (inline_encoding, Encoding) ? Encoding,
        (format_type, FormatType) ? FormatType,
        (free_busy_type, FreeBusyType) ? FreeBusyType<Box<Name>>,
        (language, Language) ? Language,
        (membership, Member) ? Vec1<Box<Uri>>,
        (participation_status, PartStat) ? ParticipationStatus<Box<Name>>,
        (recurrence_range, Range) ? ThisAndFuture,
        (trigger_relationship, Related) ? TriggerRelation,
        (relationship_type, RelType) ? RelationshipType<Box<Name>>,
        (participation_role, Role) ? ParticipationRole<Box<Name>>,
        (rsvp_expectation, Rsvp) ? bool,
        (sent_by, SentBy) ? Box<Uri>,
        (tz_id, TzId) ? Box<TzId>,

        // RFC 7986
        (display_type, Display) ? DisplayType<Box<Name>>,
        (email, Email) ? Box<ParamValue>,
        (feature_type, Feature) ? FeatureType<Box<Name>>,
        (label, Label) ? Box<ParamValue>,

        // RFC 9073
        (order, Order) ? PositiveInteger,
        (schema, Schema) ? Box<Uri>,
        (derived, Derived) ? bool,
    }
}

impl From<StructuredDataParams> for Params {
    fn from(value: StructuredDataParams) -> Self {
        Self(value.0)
    }
}

impl Params {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(ParameterTable(HashMap::with_capacity(capacity)))
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
    ValueType(ValueType<Box<Name>>),
    RawValue(AnyParamValue),
}

macro_rules! impl_any_param_value_conversions {
    ($($variant:ident => $t:ty),* $(,)?) => {
        $(
            impl From<$t> for AnyParamValue {
                fn from(value: $t) -> Self {
                    Self(AnyParamValueInner::$variant(value))
                }
            }

            impl TryFrom<AnyParamValue> for $t {
                type Error = AnyParamValue;

                fn try_from(value: AnyParamValue) -> Result<$t, Self::Error> {
                    if let AnyParamValue(AnyParamValueInner::$variant(value)) = value {
                        Ok(value)
                    } else {
                        Err(value)
                    }
                }
            }

            impl<'a> TryFrom<&'a AnyParamValue> for &'a $t {
                type Error = &'a AnyParamValue;

                fn try_from(value: &'a AnyParamValue) -> Result<&'a $t, Self::Error> {
                    if let AnyParamValue(AnyParamValueInner::$variant(value)) = value {
                        Ok(value)
                    } else {
                        Err(value)
                    }
                }
            }

            impl<'a> TryFrom<&'a mut AnyParamValue> for &'a mut $t {
                type Error = &'a mut AnyParamValue;

                fn try_from(value: &'a mut AnyParamValue) -> Result<&'a mut $t, Self::Error> {
                    if let AnyParamValue(AnyParamValueInner::$variant(value)) = value {
                        Ok(value)
                    } else {
                        Err(value)
                    }
                }
            }
        )*
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyParamValue(AnyParamValueInner);

#[derive(Debug, Clone, PartialEq, Eq)]
enum AnyParamValueInner {
    Bool(bool),
    CalAddressSeq(Vec1<Box<Uri>>),
    CUType(CalendarUserType<Box<Name>>),
    DisplayType(DisplayType<Box<Name>>),
    Encoding(Encoding),
    FBType(FreeBusyType<Box<Name>>),
    FeatureType(FeatureType<Box<Name>>),
    FormatType(FormatType),
    Language(Language),
    ParamValue1(Box<ParamValue>),
    ParamValue(Vec1<Box<ParamValue>>),
    PartStatus(ParticipationStatus<Box<Name>>),
    PositiveInteger(PositiveInteger),
    RelType(RelationshipType<Box<Name>>),
    Role(ParticipationRole<Box<Name>>),
    ThisAndFuture(ThisAndFuture),
    TrigRel(TriggerRelation),
    TzId(Box<TzId>),
    Uri(Box<Uri>),
}

impl_any_param_value_conversions! {
    Bool => bool,
    CalAddressSeq => Vec1<Box<Uri>>,
    CUType => CalendarUserType<Box<Name>>,
    DisplayType => DisplayType<Box<Name>>,
    Encoding => Encoding,
    FBType => FreeBusyType<Box<Name>>,
    FeatureType => FeatureType<Box<Name>>,
    FormatType => FormatType,
    Language => Language,
    ParamValue1 => Box<ParamValue>,
    ParamValue => Vec1<Box<ParamValue>>,
    PartStatus => ParticipationStatus<Box<Name>>,
    PositiveInteger => PositiveInteger,
    RelType => RelationshipType<Box<Name>>,
    Role => ParticipationRole<Box<Name>>,
    ThisAndFuture => ThisAndFuture,
    TrigRel => TriggerRelation,
    TzId => Box<TzId>,
    Uri => Box<Uri>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownParam {
    // RFC 5545 PROPERTY PARAMETERS
    AltRep(Box<Uri>),
    CommonName(Box<ParamValue>),
    CUType(CalendarUserType<Box<Name>>),
    DelFrom(Vec1<Box<Uri>>),
    DelTo(Vec1<Box<Uri>>),
    Dir(Box<Uri>),
    Encoding(Encoding),
    FormatType(FormatType),
    FBType(FreeBusyType<Box<Name>>),
    Language(Language),
    Member(Vec1<Box<Uri>>),
    PartStatus(ParticipationStatus<Box<Name>>),
    RecurrenceIdentifierRange,
    AlarmTrigger(TriggerRelation),
    RelType(RelationshipType<Box<Name>>),
    Role(ParticipationRole<Box<Name>>),
    Rsvp(bool),
    SentBy(Box<Uri>),
    TzId(Box<TzId>),
    Value(ValueType<Box<Name>>),
    // RFC 7986 PROPERTY PARAMETERS
    Display(DisplayType<Box<Name>>),
    Email(Box<ParamValue>),
    Feature(FeatureType<Box<Name>>),
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
        UpcastParamValue::RawValue(match self {
            KnownParam::AltRep(uri)
            | KnownParam::Dir(uri)
            | KnownParam::SentBy(uri)
            | KnownParam::Schema(uri) => uri.into(),
            KnownParam::CommonName(param_value)
            | KnownParam::Email(param_value)
            | KnownParam::Label(param_value) => param_value.into(),
            KnownParam::CUType(calendar_user_type) => calendar_user_type.into(),
            KnownParam::DelFrom(cal_addresses)
            | KnownParam::DelTo(cal_addresses)
            | KnownParam::Member(cal_addresses) => cal_addresses.into(),
            KnownParam::Encoding(encoding) => encoding.into(),
            KnownParam::FormatType(format_type) => format_type.into(),
            KnownParam::FBType(free_busy_type) => free_busy_type.into(),
            KnownParam::Language(language) => language.into(),
            KnownParam::PartStatus(participation_status) => participation_status.into(),
            KnownParam::RecurrenceIdentifierRange => ThisAndFuture.into(),
            KnownParam::AlarmTrigger(trigger_relation) => trigger_relation.into(),
            KnownParam::RelType(relationship_type) => relationship_type.into(),
            KnownParam::Role(participation_role) => participation_role.into(),
            KnownParam::Rsvp(value) | KnownParam::Derived(value) => value.into(),
            KnownParam::TzId(tz_id) => tz_id.into(),
            KnownParam::Value(value_type) => return UpcastParamValue::ValueType(value_type),
            KnownParam::Display(display_type) => display_type.into(),
            KnownParam::Feature(feature_type) => feature_type.into(),
            KnownParam::Order(non_zero) => non_zero.into(),
        })
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

impl Equivalent<ParamName<Box<CaselessStr>>> for ParamName<NeverStr> {
    fn equivalent(&self, key: &ParamName<Box<CaselessStr>>) -> bool {
        match (self, key) {
            (ParamName::Known(lhs), ParamName::Known(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Equivalent<ParamName<Box<CaselessStr>>> for ParamName<&CaselessStr> {
    fn equivalent(&self, key: &ParamName<Box<CaselessStr>>) -> bool {
        match (self, key) {
            (ParamName::Known(lhs), ParamName::Known(rhs)) => lhs == rhs,
            (ParamName::Unknown(lhs), ParamName::Unknown(rhs)) => rhs.as_ref() == *lhs,
            _ => false,
        }
    }
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
        params.insert_unknown("X-FOO", vec1![ParamValue::new("bar").unwrap().into()]);

        assert_eq!(params.rsvp_expectation(), Some(&true));
        assert_eq!(params.inline_encoding(), Some(&Encoding::Base64));
        assert_eq!(
            params.get_unknown("x-foo"),
            Some(vec1![ParamValue::new("bar").unwrap().into()].as_slice1()),
        );

        assert!(params.alternate_representation().is_none());
        assert!(params.common_name().is_none());
        assert!(params.tz_id().is_none());
    }
}
