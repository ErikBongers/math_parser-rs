use std::fmt;
use serde::Serialize;
use crate::tokenizer::cursor::{Number, Range};

#[derive(Serialize, Debug)]
#[repr(u8)]
pub enum ValueType{
    None = 0, Number = 1, Date = 2, Duration = 3, List,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Serialize)]
pub enum NumberFormat {
    Dec, Hex, Oct, Bin, Exp
}

#[derive(Clone)]
pub enum Variant {
    Number { number: Number, is_constant: bool, fmt: NumberFormat },
    Date, //TODO
    Duration,
    List,
    FunctionDef,
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
    pub stmt_range: Range,
    pub variant: Variant,
    pub has_errors: bool,
}

impl Value {
    pub fn error(range: &Range) -> Self {
        Value {
            id: None,
            stmt_range: range.clone(),
            variant: Variant::Error,
            has_errors: true
        }
    }

    pub fn from_number(value: Number, range: &Range) -> Self {
        Value {
            id: None,
            stmt_range: range.clone(),
            variant: Variant::Number {number: value, is_constant: false, fmt: NumberFormat::Dec},
            has_errors: false
        }
    }
}
