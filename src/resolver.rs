mod value;
mod operator;
mod globals;

use std::any::TypeId;
use crate::parser::CodeBlock;
use crate::parser::nodes::{BinExpr, ConstExpr, Node};
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::value::{Error, Value, variant_to_value_type};
use crate::tokenizer::token_type::TokenType::Plus;

struct Resolver<'a> {
    code_block: &'a CodeBlock,
    //date_format: DateFormat,
    //output_stream ?
}

impl<'a> Resolver<'a> {
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
        let operator_id = operator_id_from(variant_to_value_type(&expr1.variant), operator_type, variant_to_value_type(&expr2.variant));
        //TEST
        expr1
    }

}

