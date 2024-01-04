use serde::{Serialize, Serializer};
use serde::ser::{SerializeStruct, SerializeSeq};
use crate::errors;
use crate::errors::ERROR_MAP;
use crate::parser::date;
use crate::parser::date::date::EMPTY_YEAR;
use crate::resolver::globals::Globals;
use crate::resolver::Resolver;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Value, Variant::*, variant_to_value_type};
use crate::tokenizer::cursor::Range;

struct ScopedValue<'a> {
    // scope: Rc<RefCell<Scope>>,
    globals: &'a Globals,
    value: &'a Value
}

impl<'g, 'a> Resolver<'g, 'a> {
    fn build_scoped_values(&self) -> Vec<ScopedValue> {
        let context_results: Vec<ScopedValue> =
            self.results.iter()
            .map(|value|
                ScopedValue { globals: self.globals, value: &value})
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
        let mut state = serializer.serialize_struct("Value", 5)?;

        if let Some(id) = &self.value.id {
            state.serialize_field("id", &self.globals.sources[id.source_index as usize].text[id.start..id.end])?;
        } else {
            state.serialize_field("id", "_")?; //TODO: replace with None? This will be output as `null` or better, DON'T output id -> shorter JSON
        }

        state.serialize_field("type", &variant_to_value_type(&self.value.variant))?;
        state.serialize_field("src", &self.value.stmt_range.source_index)?;
        let source = &self.globals.sources[self.value.stmt_range.source_index as usize];
        let (line, _) = source.get_line_and_column(self.value.stmt_range.start);
        state.serialize_field("line", &line)?;

        match &self.value.variant {
            Numeric { number, .. } => state.serialize_field("number", number),
            Date { date } => state.serialize_field("date", date),
            Comment  => state.serialize_field("comment", &source.text[self.value.stmt_range.start..self.value.stmt_range.end]),
            FunctionDef => {
                let mut function_name = "".to_string();
                function_name =  source.text[self.value.stmt_range.start..self.value.stmt_range.end].to_string();
                state.serialize_field("function", &function_name)
            },
            List { values }=> {
                let scoped_values: Vec<ScopedValue> = values.iter().map(|v| ScopedValue { globals: &self.globals, value: &v }).collect();
                state.serialize_field("list", &scoped_values)
            }
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

struct ErrorContext<'a> {
    error: &'a errors::Error,
    globals: &'a Globals,
}

impl<'a> Serialize for ErrorContext<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_struct("error", 5)?;

        state.serialize_field("id", &self.error.id)?;
        let error_type = &ERROR_MAP.get(&self.error.id).unwrap().error_type;
        state.serialize_field("type", &error_type)?;
        state.serialize_field("msg", &self.error.message)?;
        state.serialize_field("range", &RangeContext{ range: &self.error.range, globals: self.globals })?;
        state.serialize_field("stackTrace", &self.error.stack_trace)?;
        state.end()
    }
}

struct RangeContext<'a> { //TODO: lot of overhead. Perhaps create the inner struct directly in ErrorContext?
    range: &'a Range,
    globals: &'a Globals,
}

impl<'a> Serialize for RangeContext<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let source = &self.globals.sources[self.range.source_index as usize];
        let (start_line, start_pos) = source.get_line_and_column(self.range.start);
        let (end_line, end_pos) = source.get_line_and_column(self.range.end);

        let mut state = serializer.serialize_struct("range", 5)?;

        state.serialize_field("sourceIndex", &self.range.source_index)?;
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
impl Serialize for date::Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_struct("date", 4)?;
        let str_day = if self.day == 0 { "??".to_string()} else { self.day.to_string()};
        let str_year = if self.year == EMPTY_YEAR { "????".to_string()} else { self.year.to_string()};
        let formatted = format!("{0}-{1:?}-{2}", &str_year, &self.month, &self.day);
        state.serialize_field("formatted", &formatted)?;
        state.serialize_field("day", &self.day)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field("year", &str_year)?;

        state.end()
    }
}
