use serde::Serialize;
use crate::tokenizer::cursor::{Number, Range};

#[derive(Clone)]
pub struct Error {
    pub(crate) msg: String
}

#[derive(Serialize)]
#[repr(u8)]
pub enum ValueType{
    None = 0, Number = 1, Date = 2, Duration = 3, //TODO
}

pub enum Variant {
    Number { number: Number, constant: bool },
    Date, //TODO
    Duration,
    Comment, //echo comment
    Error //TODO
}

pub fn variant_to_value_type(variant: &Variant) -> ValueType {
    match variant {
        Variant::Number{..} => ValueType::Number,
        Variant::Date => ValueType::Date,
        Variant::Duration => ValueType::Duration,
        _ => ValueType::None
    }
}
pub struct Value {
    pub id: Option<Range>,
    pub range: Option<Range>,
    pub variant: Variant,
    pub errors: Vec<Error>, //TODO: make references?
}

impl From<Error> for Value {
    fn from(value: Error) -> Self {
        let mut v = Value {
            id: None,
            range: None,
            variant: Variant::Error,
            errors: Vec::new()
        };
        v.errors.push(value);
        v
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Value {
            id: None,
            range: None,
            variant: Variant::Number {number: value, constant: false},
            errors: Vec::new()
        }
    }
}