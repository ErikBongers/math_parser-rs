mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;

use std::any::TypeId;
use crate::parser::CodeBlock;
use crate::parser::nodes::{BinExpr, ConstExpr, Node};
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::value::{Error, Value, Variant, variant_to_value_type};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::token_type::TokenType::Plus;

pub struct Resolver<'a> {
    pub code_block: &'a CodeBlock<'a>,
    pub results: Vec<Value>,
    //date_format: DateFormat,
    //output_stream ?
}

impl<'a> Resolver<'a> {
    pub fn resolve(&mut self) {
        let mut result = Value {
            id: None,
            range: None,
            errors: Vec::new(),
            variant: Variant::Error
        };
        for stmt in &self.code_block.statements {
            result = self.resolve_node(&stmt.node);
            self.results.push(result)
        };
    }

    fn resolve_node(&mut self, expr: &Box<dyn Node>) -> Value {
        match expr.as_any().type_id() {
            t if TypeId::of::<ConstExpr>() == t => { Resolver::resolve_const_expr(expr) },
            t if TypeId::of::<BinExpr>() == t => { self.resolve_bin_expr(expr) },
            _ => { Value::from(Error{ msg:"It's a dunno...".to_string()}) },
        }
    }

    fn resolve_const_expr(expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
        Value::from(expr.value.clone())
    }

    fn resolve_bin_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
        let expr1 = self.resolve_node(&expr.expr1);
        let expr2 = self.resolve_node(&expr.expr2);
        let mut errors: Vec<Error> = Vec::new();
        errors.extend(expr1.errors.iter().cloned());

        let operator_type = OperatorType::from(&expr.op.kind);
        let operator = self.code_block.scope.globals.get_operator(&expr1, operator_type, &expr2);
        let Some(operator) = operator else { panic!(""); };

        let args = vec![expr1, expr2];
        let range = Range { source_index: 0, start: 0, end: 0};
        let result = operator(&self.code_block.scope.globals, &args, &range);
        result //TODO: add errors
    }

}

