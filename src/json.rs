//! Tools for interacting with types representing JSON values.

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    hash::Hash,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeError {
    pub expected: ValueType,
    pub received: ValueType,
}

/// A type representing a JSON value.
pub trait JsonValue: Sized {
    type Number;
    type String;
    type Array: JsonArray;
    type Object: JsonObject;
    type InvalidIntegerError;

    // CONSTRUCTORS

    fn null() -> Self;
    fn bool(value: bool) -> Self;

    fn string(value: String) -> Self;
    fn str(value: &str) -> Self;
    fn cow_str(value: Cow<'_, str>) -> Self;

    fn i32(value: i32) -> Self;
    fn u32(value: u32) -> Self;
    fn i64(value: i64) -> Result<Self, Self::InvalidIntegerError>;
    fn u64(value: u64) -> Result<Self, Self::InvalidIntegerError>;
    fn f64(value: f64) -> Self;

    fn array_with_capacity(capacity: usize) -> Self;
    fn object_with_capacity(capacity: usize) -> Self;

    fn i8(value: i8) -> Self {
        Self::i32(value as i32)
    }

    fn u8(value: u8) -> Self {
        Self::u32(value as u32)
    }

    fn i16(value: i16) -> Self {
        Self::i32(value as i32)
    }

    fn u16(value: u16) -> Self {
        Self::u32(value as u32)
    }

    fn f32(value: f32) -> Self {
        Self::f64(value as f64)
    }

    fn array() -> Self {
        Self::array_with_capacity(0)
    }

    fn object() -> Self {
        Self::object_with_capacity(0)
    }

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

    fn as_null(&self) -> Option<()>;
    fn as_bool(&self) -> Option<bool>;
    fn as_number(&self) -> Option<&Self::Number>;
    fn as_string(&self) -> Option<&Self::String>;
    fn as_array(&self) -> Option<&Self::Array>;
    fn as_object(&self) -> Option<&Self::Object>;

    // OWNED DOWNCASTS

    fn into_null(self) -> Option<()>;
    fn into_bool(self) -> Option<bool>;
    fn into_number(self) -> Option<Self::Number>;
    fn into_string(self) -> Option<Self::String>;
    fn into_array(self) -> Option<Self::Array>;
    fn into_object(self) -> Option<Self::Object>;
}

/// A type which represents a JSON object.
pub trait JsonObject {
    type Key;
    type Value;

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
pub trait JsonArray {
    type Elem;

    fn get(&self, index: usize) -> Option<&Self::Elem>;
    fn len(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = &Self::Elem>;
    fn into_iter(self) -> impl Iterator<Item = Self::Elem>;

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Eq + Hash, V> JsonObject for HashMap<K, V> {
    type Key = K;
    type Value = V;

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

    use super::{JsonObject, JsonValue, ValueType};

    impl JsonValue for Value {
        type Number = Number;
        type String = String;
        type Array = Vec<Value>;
        type Object = Map<String, Value>;
        type InvalidIntegerError = std::convert::Infallible;

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
        fn i32(value: i32) -> Self {
            value.into()
        }

        #[inline(always)]
        fn u32(value: u32) -> Self {
            value.into()
        }

        #[inline(always)]
        fn i64(value: i64) -> Result<Self, Self::InvalidIntegerError> {
            Ok(value.into())
        }

        #[inline(always)]
        fn u64(value: u64) -> Result<Self, Self::InvalidIntegerError> {
            Ok(value.into())
        }

        #[inline(always)]
        fn f64(value: f64) -> Self {
            value.into()
        }

        #[inline(always)]
        fn array_with_capacity(capacity: usize) -> Self {
            Self::Array(Vec::with_capacity(capacity))
        }

        #[inline(always)]
        fn object_with_capacity(capacity: usize) -> Self {
            Self::Object(Map::with_capacity(capacity))
        }

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
        fn as_null(&self) -> Option<()> {
            self.as_null()
        }

        #[inline(always)]
        fn as_bool(&self) -> Option<bool> {
            self.as_bool()
        }

        #[inline(always)]
        fn as_number(&self) -> Option<&<Self as JsonValue>::Number> {
            self.as_number()
        }

        #[inline(always)]
        fn as_string(&self) -> Option<&<Self as JsonValue>::String> {
            match self {
                Value::String(s) => Some(s),
                _ => None,
            }
        }

        #[inline(always)]
        fn as_array(&self) -> Option<&<Self as JsonValue>::Array> {
            self.as_array()
        }

        #[inline(always)]
        fn as_object(&self) -> Option<&<Self as JsonValue>::Object> {
            self.as_object()
        }

        #[inline(always)]
        fn into_null(self) -> Option<()> {
            match self {
                Value::Null => Some(()),
                _ => None,
            }
        }

        #[inline(always)]
        fn into_bool(self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(b),
                _ => None,
            }
        }

        #[inline(always)]
        fn into_number(self) -> Option<<Self as JsonValue>::Number> {
            match self {
                Value::Number(number) => Some(number),
                _ => None,
            }
        }

        #[inline(always)]
        fn into_string(self) -> Option<<Self as JsonValue>::String> {
            match self {
                Value::String(string) => Some(string),
                _ => None,
            }
        }

        #[inline(always)]
        fn into_array(self) -> Option<<Self as JsonValue>::Array> {
            match self {
                Value::Array(array) => Some(array),
                _ => None,
            }
        }

        #[inline(always)]
        fn into_object(self) -> Option<<Self as JsonValue>::Object> {
            match self {
                Value::Object(object) => Some(object),
                _ => None,
            }
        }
    }

    impl JsonObject for Map<String, Value> {
        type Key = String;
        type Value = Value;

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
