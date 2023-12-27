pub mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;
pub mod unit;

use std::any::TypeId;
use crate::errors::{Error, ErrorId};
use crate::parser::CodeBlock;
use crate::parser::nodes::{AssignExpr, BinExpr, ConstExpr, IdExpr, Node, PostfixExpr, Statement};
use crate::resolver::operator::OperatorType;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Value, Variant};
use crate::resolver::value::Variant::Number;
use crate::tokenizer::cursor::Range;

pub struct Resolver<'a> {
    pub scope: &'a mut Scope<'a>,
    pub results: Vec<Value>,
    pub errors: Vec<Error>,
    //date_format: DateFormat,
}

// struct Results {
//     pub results: Vec<Value>,
//     pub errors: Vec<Error>,
// }
//
// impl Into<Results> for Resolver {
//     fn into(self) -> Results {
//         Results {
//             results: self.results,
//             errors: self.errors,
//         }
//     }
// }

impl<'a> Resolver<'a> {

    pub fn resolve(&mut self, statements: &Vec<Statement>) {
        for stmt in statements {
            let result = self.resolve_node(&stmt.node);
            self.results.push(result);
        };
    }

    pub fn add_error(&mut self, id: ErrorId, range: Range, arg1: &str, mut value: Value) -> Value {
        value.has_errors = true;
        self.errors.push(Error::build_1_arg(id, range, arg1));
        value
    }

    pub fn resolve_node(&mut self, expr: &Box<dyn Node>) -> Value {
        match expr.as_any().type_id() {
            t if TypeId::of::<ConstExpr>() == t => { self.resolve_const_expr(expr) },
            t if TypeId::of::<BinExpr>() == t => { self.resolve_bin_expr(expr) },
            t if TypeId::of::<IdExpr>() == t => { self.resolve_id_expr(expr) },
            t if TypeId::of::<AssignExpr>() == t => { self.resolve_assign_expr(expr) },
            t if TypeId::of::<PostfixExpr>() == t => { self.resolve_postfix_expr(expr) },
            _ => { self.add_error(ErrorId::Expected, Range::none(), "It's a dunno...", Value::error()) },
        }
    }

    fn resolve_postfix_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
        let mut result = self.resolve_node(&expr.node);
        match &mut result.variant {
            Number { number, .. } => {
                let id = self.scope.globals.get_text(&expr.postfix_id.range);
                number.unit = Unit { id: id.to_string(), range: None }
            },
            _ => panic!("TODO in resolve_postfix_expr")
        };
        result
    }

    fn resolve_assign_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
        let mut value = self.resolve_node(&expr.expr);
        let id_str = self.scope.globals.get_text(&expr.id.range);
        if !self.scope.variables.contains_key(id_str) {
            //TODO: test if id  is unit or function or redef of constant.
            self.scope.variables.insert(id_str.to_string(), value.clone());
        }
        value.id = Some(expr.id.range.clone()); //add id here to avoid adding id to the self.scope.variables.
        value
    }

    fn resolve_id_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<IdExpr>().unwrap();
        let id = self.scope.globals.get_text(&expr.id.range);
        if let Some(val) = self.scope.variables.get(id) {
            val.clone()
        } else {
            self.add_error(ErrorId::VarNotDef, expr.id.range.clone(), id, Value::error())
        }
    }

    fn resolve_const_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
        let mut v = expr.value.clone();
        v.unit = expr.node_data.unit.clone();
        Value::from(v)
    }

    fn resolve_bin_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
        let expr1 = self.resolve_node(&expr.expr1);
        let expr2 = self.resolve_node(&expr.expr2);

        let operator_type = OperatorType::from(&expr.op.kind);
        let operator = self.scope.globals.get_operator(&expr1, operator_type, &expr2);
        let Some(operator) = operator else { panic!("TODO: gracefully report that there's no operator for this situation."); };

        let args = vec![expr1, expr2];
        let range = Range { source_index: 0, start: 0, end: 0};
        let result = operator(&self.scope.globals, &args, &range);
        result //TODO: add errors
    }
}