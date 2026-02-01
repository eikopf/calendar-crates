//! Tools for interacting with types representing JSON values.

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    hash::Hash,
};

use thiserror::Error;

use crate::model::primitive::{Int, UnsignedInt};

pub trait TryFromJson: Sized {
    type Error;

    fn try_from_json<V: DestructibleJsonValue>(value: V) -> Result<Self, Self::Error>;
}

pub trait TryIntoJson {
    type Error;

    fn try_into_json<V: ConstructibleJsonValue>(self) -> Result<V, Self::Error>;
}

pub trait IntoJson {
    fn into_json<V: ConstructibleJsonValue>(self) -> V;
}

impl<T: IntoJson> TryIntoJson for T {
    type Error = std::convert::Infallible;

    fn try_into_json<V: ConstructibleJsonValue>(self) -> Result<V, Self::Error> {
        Ok(self.into_json())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ValueType::Null => "null",
            ValueType::Bool => "bool",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("expected a value of type {expected} but received type {received} instead")]
pub struct TypeError {
    pub expected: ValueType,
    pub received: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoIntError {
    #[error("type error: {0}")]
    TypeError(#[from] TypeError),
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("the signed integer {0} falls outside the valid range for Int")]
    OutsideRangeSigned(i64),
    #[error("the unsigned integer {0} falls outside the valid range for Int")]
    OutsideRangeUnsigned(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum IntoUnsignedIntError {
    #[error("type error: {0}")]
    TypeError(#[from] TypeError),
    #[error("expected an integer but received {0}")]
    NotAnInteger(f64),
    #[error("expected an unsigned integer but received {0}")]
    NegativeInteger(i64),
    #[error("the unsigned integer {0} falls outside the valid range for UnsignedInt")]
    OutsideRange(u64),
}

/// A type representing a JSON value that can be converted into Rust values.
pub trait DestructibleJsonValue: Sized {
    type Number;
    type String;
    type Array: JsonArray;
    type Object: JsonObject;

    // TYPE CHECKS

    fn value_type(&self) -> ValueType;

    #[inline(always)]
    fn is_null(&self) -> bool {
        self.value_type() == ValueType::Null
    }

    #[inline(always)]
    fn is_bool(&self) -> bool {
        self.value_type() == ValueType::Bool
    }

    #[inline(always)]
    fn is_number(&self) -> bool {
        self.value_type() == ValueType::Number
    }

    #[inline(always)]
    fn is_string(&self) -> bool {
        self.value_type() == ValueType::String
    }

    #[inline(always)]
    fn is_array(&self) -> bool {
        self.value_type() == ValueType::Array
    }

    #[inline(always)]
    fn is_object(&self) -> bool {
        self.value_type() == ValueType::Object
    }

    // REFERENTIAL DOWNCASTS

    #[inline(always)]
    fn try_as_null(&self) -> Result<(), TypeError> {
        match self.value_type() {
            ValueType::Null => Ok(()),
            received => Err(TypeError {
                expected: ValueType::Null,
                received,
            }),
        }
    }

    fn try_as_bool(&self) -> Result<bool, TypeError>;
    fn try_as_number(&self) -> Result<&Self::Number, TypeError>;
    fn try_as_string(&self) -> Result<&Self::String, TypeError>;
    fn try_as_array(&self) -> Result<&Self::Array, TypeError>;
    fn try_as_object(&self) -> Result<&Self::Object, TypeError>;

    fn try_as_int(&self) -> Result<Int, IntoIntError>;
    fn try_as_unsigned_int(&self) -> Result<UnsignedInt, IntoUnsignedIntError>;

    // OWNED DOWNCASTS

    fn try_into_number(self) -> Result<Self::Number, TypeError>;
    fn try_into_string(self) -> Result<Self::String, TypeError>;
    fn try_into_array(self) -> Result<Self::Array, TypeError>;
    fn try_into_object(self) -> Result<Self::Object, TypeError>;
}

/// A type representing a JSON value that can be built from Rust values.
pub trait ConstructibleJsonValue: Sized {
    type Array: JsonArray;
    type Object: JsonObject;

    // CONSTRUCTORS

    fn null() -> Self;
    fn bool(value: bool) -> Self;

    fn string(value: String) -> Self;
    fn str(value: &str) -> Self;
    fn cow_str(value: Cow<'_, str>) -> Self;

    fn f64(value: f64) -> Self;
    fn int(value: Int) -> Self;
    fn unsigned_int(value: UnsignedInt) -> Self;

    fn array(value: Self::Array) -> Self;
    fn object(value: Self::Object) -> Self;
}

/// A type which represents a JSON object.
pub trait JsonObject: Sized {
    type Key;
    type Value;

    fn with_capacity(capacity: usize) -> Self;

    fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord;

    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
    fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)>;

    #[inline(always)]
    fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    fn keys(&self) -> impl Iterator<Item = &Self::Key> {
        self.iter().map(|(key, _)| key)
    }

    #[inline(always)]
    fn values(&self) -> impl Iterator<Item = &Self::Value> {
        self.iter().map(|(_, value)| value)
    }
}

/// A type which represents a JSON array.
pub trait JsonArray: Sized {
    type Elem;

    fn with_capacity(capacity: usize) -> Self;
    fn get(&self, index: usize) -> Option<&Self::Elem>;
    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = &Self::Elem>;
    fn into_iter(self) -> impl Iterator<Item = Self::Elem>;

    #[inline(always)]
    fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Eq + Hash, V> JsonObject for HashMap<K, V> {
    type Key = K;
    type Value = V;

    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        HashMap::with_capacity(capacity)
    }

    #[inline(always)]
    fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Eq + Hash + Ord,
    {
        HashMap::get(self, key)
    }

    #[inline(always)]
    fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        HashMap::contains_key(self, key)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
        HashMap::iter(self)
    }

    #[inline(always)]
    fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)> {
        IntoIterator::into_iter(self)
    }
}

impl<T> JsonArray for Vec<T> {
    type Elem = T;

    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&Self::Elem> {
        self.as_slice().get(index)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = &Self::Elem> {
        self.as_slice().iter()
    }

    #[inline(always)]
    fn into_iter(self) -> impl Iterator<Item = Self::Elem> {
        IntoIterator::into_iter(self)
    }
}

#[cfg(feature = "serde_json")]
mod serde_json_impl {
    use std::{borrow::Cow, hash::Hash};

    use serde_json::{Map, Number, Value};

    use crate::model::primitive::{Int, UnsignedInt};

    use super::{
        ConstructibleJsonValue, DestructibleJsonValue, IntoIntError, IntoUnsignedIntError,
        JsonObject, TypeError, ValueType,
    };

    impl DestructibleJsonValue for Value {
        type Number = Number;
        type String = String;
        type Array = Vec<Value>;
        type Object = Map<String, Value>;

        #[inline(always)]
        fn value_type(&self) -> ValueType {
            match self {
                Value::Null => ValueType::Null,
                Value::Bool(_) => ValueType::Bool,
                Value::Number(_) => ValueType::Number,
                Value::String(_) => ValueType::String,
                Value::Array(_) => ValueType::Array,
                Value::Object(_) => ValueType::Object,
            }
        }

        #[inline(always)]
        fn try_as_bool(&self) -> Result<bool, TypeError> {
            self.as_bool().ok_or_else(|| TypeError {
                expected: ValueType::Bool,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_number(&self) -> Result<&<Self as DestructibleJsonValue>::Number, TypeError> {
            self.as_number().ok_or_else(|| TypeError {
                expected: ValueType::Number,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_string(&self) -> Result<&<Self as DestructibleJsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_as_array(&self) -> Result<&<Self as DestructibleJsonValue>::Array, TypeError> {
            self.as_array().ok_or_else(|| TypeError {
                expected: ValueType::Array,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_object(&self) -> Result<&<Self as DestructibleJsonValue>::Object, TypeError> {
            self.as_object().ok_or_else(|| TypeError {
                expected: ValueType::Object,
                received: self.value_type(),
            })
        }

        #[inline(always)]
        fn try_as_int(&self) -> Result<Int, IntoIntError> {
            let number = self.try_as_number()?;

            if let Some(n) = number.as_i64() {
                Int::new(n).ok_or(IntoIntError::OutsideRangeSigned(n))
            } else if let Some(n) = number.as_u64() {
                i64::try_from(n)
                    .ok()
                    .and_then(Int::new)
                    .ok_or(IntoIntError::OutsideRangeUnsigned(n))
            } else if let Some(n) = number.as_f64() {
                Err(IntoIntError::NotAnInteger(n))
            } else {
                unreachable!()
            }
        }

        #[inline(always)]
        fn try_as_unsigned_int(&self) -> Result<UnsignedInt, IntoUnsignedIntError> {
            let number = self.try_as_number()?;

            if let Some(n) = number.as_u64() {
                UnsignedInt::new(n).ok_or(IntoUnsignedIntError::OutsideRange(n))
            } else if let Some(n) = number.as_i64() {
                Err(IntoUnsignedIntError::NegativeInteger(n))
            } else if let Some(n) = number.as_f64() {
                Err(IntoUnsignedIntError::NotAnInteger(n))
            } else {
                unreachable!()
            }
        }

        #[inline(always)]
        fn try_into_number(self) -> Result<<Self as DestructibleJsonValue>::Number, TypeError> {
            match self {
                Value::Number(number) => Ok(number),
                _ => Err(TypeError {
                    expected: ValueType::Number,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_string(self) -> Result<<Self as DestructibleJsonValue>::String, TypeError> {
            match self {
                Value::String(s) => Ok(s),
                _ => Err(TypeError {
                    expected: ValueType::String,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_array(self) -> Result<<Self as DestructibleJsonValue>::Array, TypeError> {
            match self {
                Value::Array(array) => Ok(array),
                _ => Err(TypeError {
                    expected: ValueType::Array,
                    received: self.value_type(),
                }),
            }
        }

        #[inline(always)]
        fn try_into_object(self) -> Result<<Self as DestructibleJsonValue>::Object, TypeError> {
            match self {
                Value::Object(object) => Ok(object),
                _ => Err(TypeError {
                    expected: ValueType::Object,
                    received: self.value_type(),
                }),
            }
        }
    }

    impl ConstructibleJsonValue for Value {
        type Array = Vec<Self>;
        type Object = Map<String, Self>;

        #[inline(always)]
        fn null() -> Self {
            Self::Null
        }

        #[inline(always)]
        fn bool(value: bool) -> Self {
            Self::Bool(value)
        }

        #[inline(always)]
        fn string(value: String) -> Self {
            value.into()
        }

        #[inline(always)]
        fn str(value: &str) -> Self {
            value.into()
        }

        #[inline(always)]
        fn cow_str(value: Cow<'_, str>) -> Self {
            value.into()
        }

        #[inline(always)]
        fn f64(value: f64) -> Self {
            value.into()
        }

        #[inline(always)]
        fn int(value: Int) -> Self {
            value.get().into()
        }

        #[inline(always)]
        fn unsigned_int(value: UnsignedInt) -> Self {
            value.get().into()
        }

        #[inline(always)]
        fn array(value: <Self as ConstructibleJsonValue>::Array) -> Self {
            Value::Array(value)
        }

        #[inline(always)]
        fn object(value: <Self as ConstructibleJsonValue>::Object) -> Self {
            Value::Object(value)
        }
    }

    impl JsonObject for Map<String, Value> {
        type Key = String;
        type Value = Value;

        #[inline(always)]
        fn with_capacity(capacity: usize) -> Self {
            Map::with_capacity(capacity)
        }

        #[inline(always)]
        fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
        where
            Self::Key: std::borrow::Borrow<Q>,
            Q: ?Sized + Hash + Eq + Ord,
        {
            Map::get(self, key)
        }

        #[inline(always)]
        fn contains_key<Q>(&self, key: &Q) -> bool
        where
            Self::Key: std::borrow::Borrow<Q>,
            Q: ?Sized + Hash + Eq + Ord,
        {
            Map::contains_key(self, key)
        }

        #[inline(always)]
        fn len(&self) -> usize {
            self.len()
        }

        #[inline(always)]
        fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
            Map::iter(self)
        }

        #[inline(always)]
        fn into_iter(self) -> impl Iterator<Item = (Self::Key, Self::Value)> {
            IntoIterator::into_iter(self)
        }
    }
}
