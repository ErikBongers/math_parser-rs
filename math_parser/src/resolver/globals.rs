use std::collections::HashMap;
use crate::resolver::operator::{load_operators, operator_id_from, OperatorType};
use crate::resolver::value::{Value, variant_to_value_type};
use crate::tokenizer::cursor::Range;
use super::operator;

pub struct Globals<'a> {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>,
    pub sources: Vec<&'a str>
}
impl<'a> Globals<'a> {
    pub fn new () -> Self {
        let mut globals = Globals { operators: HashMap::new(), sources: Vec::new() };
        load_operators(&mut globals);
        globals
    }

    pub fn get_operator(&self, value1: &Value, operator_type: OperatorType, value2: &Value) -> Option<&fn(&Globals, &Vec<Value>, &Range)-> Value> {
        let op_id = operator_id_from(variant_to_value_type(&value1.variant), operator_type, variant_to_value_type(&value2.variant));
        self.operators.get(&op_id)
    }
}