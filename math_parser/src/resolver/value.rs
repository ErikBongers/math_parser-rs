use serde::Serialize;
use crate::tokenizer::cursor::{Number, Range};

#[derive(Serialize)]
#[repr(u8)]
pub enum ValueType{
    None = 0, Number = 1, Date = 2, Duration = 3, List,
}

#[derive(Clone)]
pub enum Variant {
    Number { number: Number, is_constant: bool },
    Date, //TODO
    Duration,
    List,
    Comment, //echo comment
    Error //TODO
}

pub fn variant_to_value_type(variant: &Variant) -> ValueType {
    match variant {
        Variant::Number{..} => ValueType::Number,
        Variant::Date => ValueType::Date,
        Variant::Duration => ValueType::Duration,
        Variant::List => ValueType::List,
        _ => ValueType::None
    }
}

#[derive(Clone)]
pub struct Value {
    pub id: Option<Range>,
    pub range: Option<Range>,
    pub variant: Variant,
    pub has_errors: bool,
}

impl Value {
    pub fn error() -> Self {
        Value {
            id: None,
            range: None,
            variant: Variant::Error,
            has_errors: true
        }
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Value {
            id: None,
            range: None,
            variant: Variant::Number {number: value, is_constant: false},
            has_errors: false
        }
    }
}
