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
    pub code_block: &'a mut CodeBlock<'a>,
    pub results: Vec<Value>,
    //date_format: DateFormat,
}

pub fn resolve(statements: &Vec<Statement>, scope: &mut Scope) -> Vec<Value> {
    let mut results = Vec::new();
    let mut result = Value {
        id: None,
        range: None,
        errors: Vec::new(),
        variant: Variant::Error
    };
    for stmt in statements {
        result = resolve_node(scope, &stmt.node);
        results.push(result);
    };
    results
}

pub fn resolve_node(scope: &mut Scope, expr: &Box<dyn Node>) -> Value {
    match expr.as_any().type_id() {
        t if TypeId::of::<ConstExpr>() == t => { resolve_const_expr(expr) },
        t if TypeId::of::<BinExpr>() == t => { resolve_bin_expr(scope, expr) },
        t if TypeId::of::<IdExpr>() == t => { resolve_id_expr(scope, expr) },
        t if TypeId::of::<AssignExpr>() == t => { resolve_assign_expr(scope, expr) },
        t if TypeId::of::<PostfixExpr>() == t => { resolve_postfix_expr(scope, expr) },
        _ => { Value::from(Error::build_1_arg(ErrorId::Expected, Range::none(),  "It's a dunno...")) },
    }
}

fn resolve_postfix_expr(scope: &mut Scope, expr: &Box<dyn Node>) -> Value {
    let expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
    let mut result = resolve_node(scope, &expr.node);
    match &mut result.variant {
        Number { number, .. } => {
            let id = scope.globals.get_text(&expr.postfix_id.range);
            number.unit = Unit { id: id.to_string(), range: None }
        },
        _ => panic!("TODO in resolve_postfix_expr")
    };
    result
}

fn resolve_assign_expr(scope: &mut Scope, expr: &Box<dyn Node>) -> Value {
    let expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
    let mut value = resolve_node(scope, &expr.expr);
    let id_str = scope.globals.get_text(&expr.id.range);
    if !scope.variables.contains_key(id_str) {
        //TODO: test if id  is unit or function or redef of constant.
        scope.variables.insert(id_str.to_string(), value.clone());
    }
    value.id = Some(expr.id.range.clone()); //add id here to avoid adding id to the scope.variables.
    value
}

fn resolve_id_expr(scope: &mut Scope, expr: &Box<dyn Node>) -> Value {
    let expr = expr.as_any().downcast_ref::<IdExpr>().unwrap();
    let id = scope.globals.get_text(&expr.id.range);
    if let Some(val) = scope.variables.get(id) {
        val.clone()
    } else {
        Value::from(Error::build_1_arg(ErrorId::VarNotDef, expr.id.range.clone(), id))
    }
}

fn resolve_const_expr(expr: &Box<dyn Node>) -> Value {
    let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
    let mut v = expr.value.clone();
    v.unit = expr.node_data.unit.clone();
    Value::from(v)
}

fn resolve_bin_expr(scope: &mut Scope, expr: &Box<dyn Node>) -> Value {
    let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
    let expr1 = resolve_node(scope, &expr.expr1);
    let expr2 = resolve_node(scope, &expr.expr2);
    let mut errors: Vec<Error> = Vec::new();
    errors.extend(expr1.errors.iter().cloned());

    let operator_type = OperatorType::from(&expr.op.kind);
    let operator = scope.globals.get_operator(&expr1, operator_type, &expr2);
    let Some(operator) = operator else { panic!("TODO: gracefully report that there's no operator for this situation."); };

    let args = vec![expr1, expr2];
    let range = Range { source_index: 0, start: 0, end: 0};
    let result = operator(&scope.globals, &args, &range);
    result //TODO: add errors
}


