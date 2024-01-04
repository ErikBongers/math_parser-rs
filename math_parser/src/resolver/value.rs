use std::cmp::Ordering;
use std::fmt;
use serde::Serialize;
use crate::parser::date::Date;
use crate::tokenizer::cursor::{Number, Range};

#[derive(Serialize, Debug)]
#[repr(u8)]
pub enum ValueType{
    None = 0, Number = 1, Timepoint = 2, Duration = 3, List,
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
    Numeric { number: Number, fmt: NumberFormat },
    Date { date: Date},
    Duration,
    List { values: Vec<Value> },
    FunctionDef,
    Comment, //echo comment
    Error //TODO
}

pub fn variant_to_value_type(variant: &Variant) -> ValueType {
    match variant {
        Variant::Numeric {..} => ValueType::Number,
        Variant::Date {..} => ValueType::Timepoint,
        Variant::Duration => ValueType::Duration,
        Variant::List {..} => ValueType::List,
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
            variant: Variant::Numeric {number: value, fmt: NumberFormat::Dec},
            has_errors: false
        }
    }

    pub fn from_date(value: Date, range: &Range) -> Self {
        Value {
            id: None,
            stmt_range: range.clone(),
            variant: Variant::Date {date: value},
            has_errors: false
        }
    }

    pub fn as_number(&mut self) -> Option<&mut Number> {
        if let Variant::Numeric { ref mut number, ..} = self.variant {
            Some(number)
        } else {
            None
        }
    }

    /// converts a Value to an f64 where NaN is replaced with 0.0
    pub fn sortable_value(&self) -> f64 {
        if let Variant::Numeric { ref number, ..} = self.variant {
            if number.significand == f64::NAN { 0.0}  //TODO: use to_double()
            else { number.significand }
        } else {
            todo!("no sortable value defined for this value type.")
        }
    }

}
