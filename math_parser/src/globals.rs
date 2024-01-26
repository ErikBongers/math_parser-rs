use std::collections::HashMap;
use crate::functions::{create_global_function_defs, FunctionView, GlobalFunctionDef};
use crate::globals::sources::Source;
use crate::number::Number;
use crate::resolver::operator::load_operators;
use crate::resolver::unit::{create_unit_defs, Unit, UnitDef, UnitProperty, UnitsView, UnitTag};
use crate::resolver::value::{NumberFormat, Value};
use crate::tokenizer::cursor::Range;

pub mod sources;

///Opaque wrapper to avoid any int value being used.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceIndex(u8);

impl SourceIndex {
    pub fn as_int(&self) -> i32 {
        self.0 as i32
    }
}

pub struct Globals {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    sources: Vec<Source>, //keep private. Sources should only be added through set_source()
    pub unit_defs: HashMap<String, UnitDef>,
    pub global_function_defs:  HashMap<String, GlobalFunctionDef>,
    pub constants: HashMap<&'static str, Number>,
    pub units_view: UnitsView,
    pub function_view: FunctionView,
}

impl<'a> Globals {
    pub fn new () -> Self {
        let unit_defs = create_unit_defs();

        let global_function_defs = create_global_function_defs();
        let constants = HashMap::new();
        let mut globals = Globals { operators: HashMap::new(), sources: Vec::new(), unit_defs, global_function_defs, constants, units_view: UnitsView::new(), function_view: FunctionView::new() };
        globals.units_view.add_all_classes(&globals.unit_defs);
        globals.units_view.remove_tagged(UnitTag::ShortDateTime, &globals.unit_defs);
        load_operators(&mut globals);
        globals.fill_constants();
        globals.function_view.add_all(&globals.global_function_defs);
        globals
    }

    pub fn set_source(&mut self, name: String, text: String) -> SourceIndex {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.set_text(text);
            source.index
        } else  {
            let index = SourceIndex(self.sources.len() as u8);
            self.sources.push(Source::new(name, text, index));
            index
        }
    }

    pub fn get_source_by_name(&self, name: &str) -> Option<&Source> {
        self.sources.iter()
            .find(|source| source.name == name)
    }

    pub fn get_operator(&self, op_id: u32) -> Option<&fn(&Globals, &Vec<Value>, &Range)-> Value> {
        self.operators.get(&op_id)
    }

    pub fn exists_operator(&self, op_id: u32) -> bool {
        self.operators.contains_key(&op_id)
    }

    pub fn get_text(&self, range: &Range) -> &str {
        &self.sources[range.source_index.0 as usize].get_text()[range.start..range.end]
    }

    pub fn get_line_and_column(&self, range: &Range) -> (usize, usize) {
        self.sources[range.source_index.0 as usize].get_line_and_column(range.start)
    }

    pub fn get_lines_and_columns(&self, range: &Range) -> (usize, usize, usize, usize) {
        let (l1, c1) = self.sources[range.source_index.0 as usize].get_line_and_column(range.start);
        let (l2, c2) = self.sources[range.source_index.0 as usize].get_line_and_column(range.end);
        (l1, c1, l2, c2)
    }

    fn fill_constants(&mut self) {
        self.constants.insert("PI",  Number {
            significand: std::f64::consts::PI,
            exponent: 0,
            unit: Unit::none(),
            fmt: NumberFormat::Dec,
        });
    }

    pub fn is_unit(&self, unit: &Unit, property: UnitProperty) -> bool {
        if let Some(unit_def) =  self.unit_defs.get(&unit.id) {
            unit_def.property == property
        } else {
            false
        }
    }
}
