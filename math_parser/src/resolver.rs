mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;
pub mod unit;

use std::any::TypeId;
use crate::parser::CodeBlock;
use crate::parser::nodes::{BinExpr, ConstExpr, Node, PostfixExpr};
use crate::resolver::operator::OperatorType;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Error, Value, Variant};
use crate::resolver::value::Variant::Number;
use crate::tokenizer::cursor::Range;

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
            t if TypeId::of::<PostfixExpr>() == t => { self.resolve_postfix_expr(expr) },
            _ => { Value::from(Error{ msg:"It's a dunno...".to_string()}) },
        }
    }

    fn resolve_postfix_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
        let mut result = self.resolve_node(&expr.node);
        match &mut result.variant {
            Number { number, .. } => {
                let id = self.code_block.scope.globals.get_text(&expr.postfix_id.range);
                number.unit = Unit { id: id.to_string(), range: None }
            },
            _ => panic!("TODO in resolve_postfix_expr")
        };
        result
    }

    fn resolve_const_expr(expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
        let mut v = expr.value.clone();
        v.unit = expr.node_data.unit.clone();
        Value::from(v)
    }

    fn resolve_bin_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
        let expr1 = self.resolve_node(&expr.expr1);
        let expr2 = self.resolve_node(&expr.expr2);
        let mut errors: Vec<Error> = Vec::new();
        errors.extend(expr1.errors.iter().cloned());

        let operator_type = OperatorType::from(&expr.op.kind);
        let operator = self.code_block.scope.globals.get_operator(&expr1, operator_type, &expr2);
        let Some(operator) = operator else { panic!("TODO: gracefully report that there's no operator for this situation."); };

        let args = vec![expr1, expr2];
        let range = Range { source_index: 0, start: 0, end: 0};
        let result = operator(&self.code_block.scope.globals, &args, &range);
        result //TODO: add errors
    }

}

