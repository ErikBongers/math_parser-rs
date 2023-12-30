use std::cell::RefCell;
use std::rc::Rc;
use serde::{Serialize, Serializer};
use serde::ser::{SerializeStruct, SerializeSeq};
use crate::errors;
use crate::errors::ERROR_MAP;
use crate::resolver::globals::Globals;
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Value, Variant::*, variant_to_value_type};
use crate::tokenizer::cursor::Range;

struct ScopedValue<'a> {
    scope: Rc<RefCell<Scope>>,
    value: &'a Value
}

impl<'a> Resolver<'a> {
    fn build_scoped_values(&self) -> Vec<ScopedValue> {
        let context_results: Vec<ScopedValue> =
            self.results.iter()
            .map(|value|
                ScopedValue { scope: self.scope.clone(), value: &value})
            .collect();
        context_results
    }
}

impl<'a> Serialize for Resolver<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_struct("result", 2)?;

        state.serialize_field("result", &self.build_scoped_values())?;
        let errors: Vec<ErrorContext> = self.errors.iter().map(|error| ErrorContext { error, globals: self.scope.borrow().globals.clone()}).collect();
        state.serialize_field("errors", &errors)?;
        state.end()
    }
}

impl<'a> Serialize for ScopedValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("Value", 3)?;

        if let Some(id) = &self.value.id {
            state.serialize_field("id", &self.scope.borrow().globals.sources[id.source_index as usize][id.start..id.end])?;
        } else {
            state.serialize_field("id", "_")?; //TODO: replace with None? This will be output as `null`
        }

        state.serialize_field("type", &variant_to_value_type(&self.value.variant))?;
        match &self.value.variant {
            Number { number, .. } => state.serialize_field("number", number),
            FunctionDef => {
                let mut function_name = "".to_string();
                if let Some(range) = &self.value.range {
                    function_name =  self.scope.borrow().globals.sources[range.source_index as usize][range.start..range.end].to_string();
                }
                state.serialize_field("function", &function_name)
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
    globals: Rc<Globals>,
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
        state.serialize_field("range", &self.error.range)?;
        state.serialize_field("stackTrace", &self.error.stack_trace)?;
        state.end()
    }
}
