use crate::date::{Duration, Timepoint};
use crate::number::Number;
use crate::tokenizer::cursor::Range;

#[derive(Clone)]
pub enum Variant {
    Numeric { number: Number },
    Date { date: Timepoint },
    Duration { duration: Duration },
    List { values: Vec<Value> },
    FunctionDef,
    Comment, //echo comment
    Last, // used for dates.
    Error,
    Define,
    None,
}

#[repr(u8)]
pub enum OperandType { Number, Date, Duration, Invalid }

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

    pub fn to_operand_type(&self) -> OperandType {
        match self {
            Variant::Numeric {..} => OperandType::Number,
            Variant::Date {..} => OperandType::Date,
            Variant::Duration {..} => OperandType::Duration,
            _  => OperandType::Invalid,
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

    pub fn from_list(values: Vec<Value>, range: Range) -> Self {
        Value {
            id: None,
            stmt_range: range,
            variant: Variant::List { values },
            has_errors: false,
        }
    }

    pub fn from_date(date: Timepoint, range: Range) -> Self {
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

    pub fn as_sortable_number(&self) -> Option<f64> {
        if let Variant::Numeric { ref number, ..} = self.variant {
            Some(number.sortable_value())
        } else {
            None
        }
    }

    pub fn as_date(&mut self) -> Option<&mut Timepoint> {
        if let Variant::Date { ref mut date, ..} = self.variant {
            Some(date)
        } else {
            None
        }
    }

}
