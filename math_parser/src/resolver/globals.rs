use std::collections::HashMap;
use crate::functions::{create_global_function_defs, GlobalFunctionDef};
use crate::resolver::operator::{load_operators, operator_id_from, OperatorType};
use crate::resolver::unit::{create_unit_defs, UnitDef, UnitsView};
use crate::resolver::value::{Value, variant_to_value_type};
use crate::tokenizer::cursor::Range;

pub struct Globals {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    pub sources: Vec<String>,
    pub unit_defs: HashMap<String, UnitDef>,
    pub units_view: UnitsView,//TODO: remove this here? This is a mutable thing, so should be done in Scope.
    pub global_function_defs:  HashMap<String, GlobalFunctionDef>,
}

impl<'a> Globals {
    pub fn new () -> Self {
        let unit_defs = create_unit_defs();

        let mut units_view = UnitsView::new();
        units_view.add_all_classes(&unit_defs);

        let global_function_defs = create_global_function_defs();

        let mut globals = Globals { operators: HashMap::new(), sources: Vec::new(), unit_defs, units_view, global_function_defs };
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
        &self.sources[range.source_index as usize][range.start..range.end]
    }
}