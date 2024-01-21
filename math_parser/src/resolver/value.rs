use serde::Serialize;
use crate::date::{Date, Duration};
use crate::number::Number;
use crate::tokenizer::cursor::Range;

#[derive(Clone, Serialize)]
pub enum NumberFormat {
    Dec, Hex, Oct, Bin, Exp
}

#[derive(Clone)]
pub enum Variant {
    Numeric { number: Number },
    Date { date: Date},
    Duration { duration: Duration },
    List { values: Vec<Value> },
    FunctionDef,
    Comment, //echo comment
    Last, // used for dates.
    Error,
    Define,
    None,
}

impl Variant {
    pub fn name(&self) -> &'static str {
        match self {
            Variant::Numeric {..} => "Number",
            Variant::Date {..} => "Timepoint",
            Variant::Duration {..} => "Duration",
            Variant::List {..} => "List",
            Variant::FunctionDef => "FunctionDef",
            Variant::Comment  => "Comment",
            Variant::Last  => "Last",
            Variant::Error  => "Error",
            Variant::Define  => "Define",
            Variant::None  => "None",
        }
    }

    pub fn to_u32(&self) -> u32 { //implemeent a cast to numeric value?
        match self {
            Variant::Numeric {..} => 1,
            Variant::Date {..} => 2,
            Variant::Duration {..} => 3,
            Variant::List {..} => 4,
            Variant::FunctionDef => 5,
            Variant::Comment  => 6,
            Variant::Last  => 7,
            Variant::Error  => 8,
            Variant::Define  => 9,
            Variant::None  => 10,
        }
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
    pub fn error(range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::Error,
            has_errors: true,
        }
    }
    pub fn none(range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::None,
            has_errors: false,
        }
    }

    pub fn last_variant(range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::Last,
            has_errors: false,
        }
    }

    pub fn from_number(value: Number, range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::Numeric {number: value},
            has_errors: false,
        }
    }

    pub fn from_date(date: Date, range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::Date {date },
            has_errors: false,
        }
    }

    pub fn from_duration(duration: Duration, range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::Duration {duration},
            has_errors: false,
        }
    }

    pub fn as_number_mut(&mut self) -> Option<&mut Number> {
        if let Variant::Numeric { ref mut number, ..} = self.variant {
            Some(number)
        } else {
            None
        }
    }
    pub fn as_number(&self) -> Option<&Number> {
        if let Variant::Numeric { ref number, ..} = self.variant {
            Some(number)
        } else {
            None
        }
    }

    pub fn as_date(&mut self) -> Option<&mut Date> {
        if let Variant::Date { ref mut date, ..} = self.variant {
            Some(date)
        } else {
            None
        }
    }

    /// converts a Value to an f64 where NaN is replaced with 0.0
    pub fn sortable_value(&self) -> f64 {
        if let Variant::Numeric { ref number, ..} = self.variant {
            if number.to_double().is_nan() { 0.0 }
            else { number.to_double() }
        } else {
            todo!("no sortable value defined for this value type.")
        }
    }

}
