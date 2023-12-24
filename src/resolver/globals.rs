use std::collections::HashMap;
use crate::resolver::value::Value;
use crate::tokenizer::cursor::Range;

pub struct Globals {
    pub operators: HashMap<u32, fn(&Globals, &Vec<Value>, &Range)-> Value>
}