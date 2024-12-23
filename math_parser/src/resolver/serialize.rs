use std::cell::RefCell;
use std::rc::Rc;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::{date, errors};
use crate::globals::Globals;
use crate::number::Number;
use crate::number_format::NumberFormat;
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Value, Variant::*};
use crate::tokenizer::cursor::Range;

struct ScopedValue<'a> {
    scope: Rc<RefCell<Scope>>,
    globals: &'a Globals,
    value: &'a Value,
}

impl<'g, 'a> Resolver<'g, 'a> {
    fn build_scoped_values(&self) -> Vec<ScopedValue> {
        let context_results: Vec<ScopedValue> =
            self.results.iter()
            .map(|value|
                ScopedValue { scope: self.scope.clone(), globals: self.globals, value: &value})
            .collect();
        context_results
    }
}

impl<'g, 'a> Serialize for Resolver<'g, 'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_struct("result", 2)?;

        state.serialize_field("result", &self.build_scoped_values())?;
        let errors: Vec<ErrorContext> = self.errors.iter().map(|error| ErrorContext { error, globals: self.globals}).collect();
        state.serialize_field("errors", &errors)?;
        state.end()
    }
}

impl<'a> Serialize for ScopedValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {

        let mut state = if let Some(id) = &self.value.id {
            let mut state = serializer.serialize_struct("Value", 5)?;
            state.serialize_field("id", &self.globals.get_text(id))?;
            state
        } else {
            let state = serializer.serialize_struct("Value", 4)?;
            //don't output id -> shorter JSON
            state
        };

        state.serialize_field("type", &&self.value.variant.name())?;
        state.serialize_field("src", &self.value.stmt_range.source_index.as_int())?;
        let (line, _) = self.globals.get_line_and_column(&self.value.stmt_range);
        state.serialize_field("line", &line)?;

        match &self.value.variant {
            Numeric { number, .. } => {
                state.serialize_field("number", &NumberContext{ number: &number, scope: self.scope.clone()})
            },
            Date { date } => state.serialize_field("date", date),
            Duration { duration } => state.serialize_field("duration", duration),
            Comment  => state.serialize_field("comment", self.globals.get_text(&self.value.stmt_range)),
            FunctionDef => {
                let function_name =  self.globals.get_text(&self.value.stmt_range).to_string();
                state.serialize_field("function", &function_name)
            },
            List { values }=> {
                let scoped_values: Vec<ScopedValue> = values.iter().map(|v| ScopedValue { scope: self.scope.clone(), globals: &self.globals, value: &v }).collect();
                state.serialize_field("list", &scoped_values)
            },
            Last => {
                state.serialize_field("Last", "last")
            },
            None => {
                state.serialize_field("range", &&RangeContext{ range: &self.value.stmt_range, globals: self.globals })
            },
            _ => state.serialize_field("todo", "No serialization for this Value.Variant.")
        }?;
        state.end()
    }
}

impl Serialize for Unit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.id)
    }
}

#[inline]
fn reduce_precision(n: f64, prec: f64) -> f64 {
    (n*prec).round()/prec
}

struct NumberContext<'n> {
    number: &'n Number,
    scope: Rc<RefCell<Scope>>,
}

impl<'n> Serialize for NumberContext<'n> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("number", 5)?;

        state.serialize_field("sig", &self.number.significand)?;
        state.serialize_field("exp", &self.number.exponent)?;
        state.serialize_field("u", &self.number.unit)?;
        state.serialize_field("fmt", &self.number.fmt)?;
        let precision = self.scope.borrow().precision;
        let reduced_precision = reduce_precision(self.number.to_double(), precision);
        let fmtd = match &self.number.fmt {
            NumberFormat::Dec => format!("{}", reduced_precision),
            NumberFormat::Hex => format!("0x{:0X}", reduced_precision as u64),
            NumberFormat::Oct => format!("0o{:0o}", reduced_precision as u64),
            NumberFormat::Bin => format!("0b{:0b}", reduced_precision as u64),
            NumberFormat::Exp => {
                let norm = self.number.normalize_number();
                format!("{0}e{1}", reduce_precision(norm.significand, precision), norm.exponent)
            },
        };
        state.serialize_field("fmtd", &fmtd)?;
        state.end()
    }
}

struct ErrorContext<'a> {
    error: &'a errors::Error,
    globals: &'a Globals,
}

impl<'a> Serialize for ErrorContext<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut cnt_fields = 5;
        if self.error.stack_trace.is_none() { cnt_fields -= 1; }

        let mut state = serializer.serialize_struct("error", cnt_fields)?;

        state.serialize_field("id", &self.error.id)?;
        let error_type = &self.error.error_type;
        state.serialize_field("type", &error_type)?;
        state.serialize_field("msg", &self.error.message)?;
        state.serialize_field("range", &RangeContext{ range: &self.error.range, globals: self.globals })?;
        if let Some(stack_trace) = &self.error.stack_trace {
            let ctx_stack_trace = stack_trace
                .iter()
                .map(|error|
                    ErrorContext { error, globals: self.globals }
                );
            let adapter = IteratorAdapter::new(ctx_stack_trace);
            state.serialize_field("stackTrace", &adapter)?;
        }
        state.end()
    }
}

struct IteratorAdapter<I> {
    iterator: RefCell<I>,
}

impl<I> IteratorAdapter<I> {
    fn new(iterator: I) -> Self {
        Self { iterator: RefCell::new(iterator) }
    }
}

impl<I> Serialize for IteratorAdapter<I>
    where
        I: Iterator,
        I::Item: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        serializer.collect_seq(self.iterator.borrow_mut().by_ref())
    }
}

struct RangeContext<'a> {
    range: &'a Range,
    globals: &'a Globals,
}

impl<'a> Serialize for RangeContext<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let (start_line, start_pos, end_line, end_pos) = self.globals.get_lines_and_columns(&self.range);

        let mut state = serializer.serialize_struct("range", 5)?;

        state.serialize_field("sourceIndex", &self.range.source_index.as_int())?;
        state.serialize_field("startLine", &start_line)?;
        state.serialize_field("startPos", &start_pos)?;
        state.serialize_field("endLine", &end_line)?;
        state.serialize_field("endPos", &end_pos)?;

        state.end()
    }
}
/*
void ResultJsonSerializer::serialize(const Date& date, const Scope& scope)
    {
    sstr << "{";

    sstr << "\"formatted\" : ";

    sstr << std::setfill('0')
        << "\""
        << std::setw(4)
        << (date.year == Date::EmptyYear ? 0 : date.year)
        << "/"
        << std::setw(2)
        << monthToString(date.month)
        << "/"
        << static_cast<int>(date.day)
        << "\"";

    sstr << ",\"day\":\"" << (int)date.day << "\"";
    sstr << ",\"month\":\"" << (int)date.month<< "\"";
    sstr << ",\"year\":\"" << date.year<< "\"";

    sstr << "}";
    }
*/
impl Serialize for date::Timepoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_struct("Timepoint", 4)?;
        let norm_day = self.get_normalized_day();
        let str_day_formatted = if norm_day == 0 {
            "??".to_string()
        } else {
            norm_day.to_string()
        };
        let str_day = if self.day.is_none() {
            "--".to_string()
        } else {
            if self.day.is_last() {
                format!("last ({})", self.get_normalized_day())
            } else {
                self.get_normalized_day().to_string()
            }
        };
        let str_year = if let Some(year) = self.year { year.to_string()} else { "????".to_string()};
        let formatted = format!("{0}-{1:?}-{2}", &str_year, &self.month, &str_day_formatted);
        state.serialize_field("formatted", &formatted)?;
        state.serialize_field("day", &str_day)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field("year", &str_year)?;

        state.end()
    }
}
impl Serialize for date::Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut slices: Vec<String> = Vec::new();
        let mut dur = *self;
        dur.normalize();
        if self.years != 0 {
            slices.push(format!("{} years", dur.years));
            slices.push(format!("{} months", dur.months));
            slices.push(format!("{} days", dur.days));
        }
        let formatted = slices.join(",");

        let mut state = serializer.serialize_struct("Duration", 4)?;
        state.serialize_field("formatted", &formatted)?;
        state.serialize_field("days", &dur.days)?;
        state.serialize_field("months", &dur.months)?;
        state.serialize_field("years", &dur.years)?;

        state.end()
    }
}
