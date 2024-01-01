use std::collections::HashMap;
use crate::functions::{create_global_function_defs, GlobalFunctionDef};
use crate::resolver::operator::{load_operators, operator_id_from, OperatorType};
use crate::resolver::unit::{create_unit_defs, UnitDef, UnitsView};
use crate::resolver::value::{Value, variant_to_value_type};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::sources::Source;

pub struct Globals {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    pub sources: Vec<Source>,
    pub unit_defs: HashMap<String, UnitDef>,
    pub global_function_defs:  HashMap<String, GlobalFunctionDef>,
}

impl<'a> Globals {
    pub fn new () -> Self {
        let unit_defs = create_unit_defs();

        let global_function_defs = create_global_function_defs();

        let mut globals = Globals { operators: HashMap::new(), sources: Vec::new(), unit_defs, global_function_defs };
        load_operators(&mut globals);
        globals
    }

    pub fn get_operator(&self, op_id: u32) -> Option<&fn(&Globals, &Vec<Value>, &Range)-> Value> {
        self.operators.get(&op_id)
    }

    pub fn exists_operator(&self, op_id: u32) -> bool {
        self.operators.contains_key(&op_id)
    }

    pub fn get_text(&self, range: &Range) -> &str {
        &self.sources[range.source_index as usize].text[range.start..range.end]
    }
}