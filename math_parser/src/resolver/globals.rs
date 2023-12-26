use std::collections::HashMap;
use crate::errors::{ErrorDef, ErrorId, load_error_defs};
use crate::resolver::operator::{load_operators, operator_id_from, OperatorType};
use crate::resolver::unit::{create_defs, UnitDef, UnitsView};
use crate::resolver::value::{Value, variant_to_value_type};
use crate::tokenizer::cursor::Range;

pub struct Globals<'a> {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    pub sources: Vec<&'a str>,
    pub unit_defs: HashMap<&'a str, UnitDef<'a>>,
    pub units_view: UnitsView<'a>,
    pub errors: HashMap<ErrorId, ErrorDef<'a>>,
}

impl<'a> Globals<'a> {
    pub fn new () -> Self {
        let unit_defs = create_defs();

        let mut units_view = UnitsView::new();
        units_view.add_all_classes(&unit_defs);

        let mut globals = Globals { operators: HashMap::new(), sources: Vec::new(), unit_defs, units_view, errors: load_error_defs() };
        load_operators(&mut globals);
        globals
    }

    pub fn get_operator(&self, value1: &Value, operator_type: OperatorType, value2: &Value) -> Option<&fn(&Globals, &Vec<Value>, &Range)-> Value> {
        let op_id = operator_id_from(variant_to_value_type(&value1.variant), operator_type, variant_to_value_type(&value2.variant));
        self.operators.get(&op_id)
    }

    pub fn get_text(&self, range: &Range) -> &str {
        &self.sources[range.source_index as usize][range.start..range.end]
    }
}