use std::collections::HashMap;
use crate::functions::{create_global_function_defs, FunctionView, GlobalFunctionDef};
use crate::resolver::operator::load_operators;
use crate::resolver::unit::{create_unit_defs, Unit, UnitDef, UnitsView, UnitTag};
use crate::resolver::value::{NumberFormat, Value, Variant};
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::sources::Source;

pub struct Globals {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    pub sources: Vec<Source>, //TODO: try to make private. Serializer may fail.
    pub unit_defs: HashMap<String, UnitDef>,
    pub global_function_defs:  HashMap<String, GlobalFunctionDef>,
    pub constants: HashMap<&'static str, Value>,
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

    pub fn set_source(&mut self, name: String, text: String) -> i32 {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.set_text(text);
            source.index as i32
        } else  {
            self.sources.push(Source::new(name, text));
            let index = self.sources.len() - 1;
            if let Some(last) = self.sources.last_mut() {
                last.index = index;
            }
            index as i32
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
        &self.sources[range.source_index as usize].get_text()[range.start..range.end]
    }
    
    fn fill_constants(&mut self) {
        self.constants.insert("PI", Value {
            id: None,
            stmt_range: Range::none(),
            variant: Variant::Numeric {
                number: Number {
                    significand: std::f64::consts::PI,
                    exponent: 0,
                    unit: Unit::none(),
                    fmt: NumberFormat::Dec,
                },
            },
            has_errors: false,
        });
    }
}