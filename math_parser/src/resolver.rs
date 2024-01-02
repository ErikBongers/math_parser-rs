pub mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;
pub mod unit;

use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;
use macros::CastAny;
use crate::errors::{Error, ErrorId};
use crate::functions::FunctionDef;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, CommentExpr, ConstExpr, FunctionDefExpr, HasRange, IdExpr, ListExpr, Node, PostfixExpr, Statement, UnaryExpr, UnitExpr};
use crate::resolver::globals::Globals;
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::scope::Scope;
use crate::resolver::unit::{Unit, UnitsView};
use crate::resolver::value::{Value, Variant, variant_to_value_type};
use crate::resolver::value::Variant::Numeric;
use crate::tokenizer::cursor;
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::token_type::TokenType;

pub struct Resolver<'g, 'a> {
    pub globals: &'g Globals,
    pub scope: Rc<RefCell<Scope>>,
    pub results: Vec<Value>,
    pub errors: &'a mut Vec<Error>,
    //date_format: DateFormat,
}

pub fn add_error(errors: &mut Vec<Error>, id: ErrorId, range: Range, args: &[&str], mut value: Value) -> Value {
    value.has_errors = true;
    errors.push(Error::build(id, range, args));
    value
}


impl<'g, 'a> Resolver<'g, 'a> {

    pub fn resolve(&mut self, statements: &Vec<Box<Statement>>) -> Option<Value> {
        for stmt in statements {
            let result = self.resolve_statement(stmt);
            self.results.push(result);
        };
        let Some(result) = self.results.last() else {
            return None
        };
        Some(result.clone())
    }

    pub fn add_error(&mut self, id: ErrorId, range: Range, args: &[&str], mut value: Value) -> Value {
        value.has_errors = true;
        self.errors.push(Error::build(id, range, args));
        value
   }

    pub fn resolve_node(&mut self, expr: &Box<dyn Node>) -> Value {
        match expr.as_any().type_id() {
            t if TypeId::of::<ConstExpr>() == t => { self.resolve_const_expr(expr) },
            t if TypeId::of::<BinExpr>() == t => { self.resolve_bin_expr(expr) },
            t if TypeId::of::<IdExpr>() == t => { self.resolve_id_expr(expr) },
            t if TypeId::of::<AssignExpr>() == t => { self.resolve_assign_expr(expr) },
            t if TypeId::of::<UnaryExpr>() == t => { self.resolve_unary_expr(expr) },
            t if TypeId::of::<PostfixExpr>() == t => { self.resolve_postfix_expr(expr) },
            t if TypeId::of::<UnitExpr>() == t => { self.resolve_unit_expr(expr) },
            t if TypeId::of::<CallExpr>() == t => { self.resolve_call_expr(expr) },
            t if TypeId::of::<ListExpr>() == t => { self.resolve_list_expr(expr) },
            t if TypeId::of::<CommentExpr>() == t => { self.resolve_comment_expr(expr) },
            t if TypeId::of::<FunctionDefExpr>() == t => { self.resolve_func_def_expr(expr) },
            _ => { self.add_error(ErrorId::Expected, Range::none(), &["Unknown expression to reo"], Value::error(&expr.get_range())) },
        }
    }

    fn resolve_list_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let list_expr = expr.as_any().downcast_ref::<ListExpr>().unwrap();
        let mut number_list = Vec::<Value>::new();
        for item in &list_expr.nodes {
            let value = self.resolve_node(item);
            number_list.push(value);
        };
        Value {
            id: None,
            stmt_range: expr.get_range().clone(),
            variant: Variant::List {values: number_list},
            has_errors: false,
        }
    }

    fn resolve_statement(&mut self, expr: &Box<Statement>) -> Value {
        let stmt = expr.as_any().downcast_ref::<Statement>().unwrap();
        let mut result = self.resolve_node(&stmt.node);
        result.stmt_range = stmt.get_range();
        result
    }

    fn resolve_comment_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        Value {
            id: None,
            stmt_range: expr.get_range().clone(),
            variant: Variant::Comment,
            has_errors: false,
        }
    }

    fn resolve_func_def_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let func_expr = expr.as_any().downcast_ref::<FunctionDefExpr>().unwrap();
        Value { //TODO: add id and full range of function.
            id: Some(func_expr.id_range.clone()),
            has_errors: false,
            stmt_range: func_expr.get_range(),
            variant: Variant::FunctionDef
        }
    }

    fn resolve_call_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let call_expr = expr.as_any().downcast_ref::<CallExpr>().unwrap();
        if call_expr.node_data.has_errors {
            return Value::error(&call_expr.get_range());
        };
        let global_function_def = self.globals.global_function_defs.contains_key(call_expr.function_name.as_str());
        let local_function_def = self.scope.borrow().local_function_defs.contains_key(call_expr.function_name.as_str()); //TODO: replace with a recursove function over parent scopes.
        if !global_function_def && !local_function_def {
            return self.add_error(ErrorId::FuncNotDef, call_expr.function_name_range.clone(), &[&call_expr.function_name], Value::error(&call_expr.function_name_range));
        };

        let arguments = call_expr.arguments.as_any().downcast_ref::<ListExpr>().unwrap();

        //TODO: try trait objects. (trait references, actually)
        let mut arg_count_wrong = false;
        if global_function_def {
            if !self.globals.global_function_defs.get(call_expr.function_name.as_str()).unwrap().is_correct_arg_count(arguments.nodes.len()) {
                arg_count_wrong = true;
            }
        }
        //TODO: if both global and local function exists, then what? Use local?
        if local_function_def {
            if !self.scope.borrow().local_function_defs.get(call_expr.function_name.as_str()).unwrap().is_correct_arg_count(arguments.nodes.len()) {
                arg_count_wrong = true;
            }
        }

        if arg_count_wrong {
            return self.add_error(ErrorId::FuncArgWrong, call_expr.function_name_range.clone(), &[&call_expr.function_name], Value::error(&call_expr.function_name_range));
        };
        let mut arg_values: Vec<Value> = Vec::new();
        for arg in &arguments.nodes {
            let value = self.resolve_node(arg);
            if value.has_errors {
                return Value::error(&value.stmt_range);
            }
            arg_values.push(value);
        };

        let mut result = if global_function_def {
            self.globals.global_function_defs.get(call_expr.function_name.as_str()).unwrap()
                .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors, self.globals)
        } else {
            if local_function_def {
                self.scope.borrow().local_function_defs.get(call_expr.function_name.as_str()).unwrap()
                    .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors, self.globals)
            } else {
                panic!("TODO");
            }
        };
        Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        result
    }

    fn resolve_unit_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let unit_expr = expr.as_any().downcast_ref::<UnitExpr>().unwrap();
        let mut result = self.resolve_node(&unit_expr.node);
        if let Numeric { ref mut number, .. } = &mut result.variant {
            Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        }
        result
    }

    fn resolve_postfix_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let pfix_expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
        let mut result = self.resolve_node(&pfix_expr.node);
        match &mut result.variant {
            Numeric { ref mut number, .. } => {
                let id = self.globals.get_text(&pfix_expr.postfix_id.range).to_string();
                number.convert_to_unit(&Unit { range: Some(pfix_expr.postfix_id.range.clone()), id }, &self.scope.borrow().units_view,&pfix_expr.postfix_id.range, self.errors, self.globals);
            },
            _ => return self.add_error(ErrorId::UnknownExpr, pfix_expr.postfix_id.range.clone(), &["Postfix expression not valid here."], result)
        };
        Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        result
    }

    //in case of (x.km)m, both postfixId (km) and unit (m) are filled.
    fn apply_unit(value: &mut Value, node: &Box<dyn Node>, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) {
        if let Some(number) = value.as_number() {
            if !node.get_node_data().unit.is_empty() {
                number.convert_to_unit(&node.get_node_data().unit, units_view, range, errors, globals);
            }
        }
        //else: ignore.
    }

    fn resolve_assign_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
        let mut value = self.resolve_node(&expr.expr);
        let id_str = self.globals.get_text(&expr.id.range).to_string();
        if !self.scope.borrow().variables.contains_key(&id_str) {
            //TODO: test if id  is unit or function or redef of constant.
            self.scope.borrow_mut().variables.insert(id_str, value.clone());
        }
        value.id = Some(expr.id.range.clone()); //add id here to avoid adding id to the self.scope.variables.
        value
    }

    fn resolve_id_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<IdExpr>().unwrap();
        let id = self.globals.get_text(&expr.id.range).to_string();
        let var_exists = self.scope.borrow().variables.contains_key(&id);
        if var_exists  {
            self.scope.borrow().variables.get(&id).unwrap().clone()
        } else {
            self.add_error(ErrorId::VarNotDef, expr.id.range.clone(), &[&id], Value::error(&expr.get_range()))
        }
    }

    fn resolve_unary_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<UnaryExpr>().unwrap();
        let mut result = self.resolve_node(&expr.expr);
        if(expr.op.kind == TokenType::Min) {
            if let Numeric {ref mut number,..} = result.variant {
                number.significand = -number.significand
            }
        }
        result
    }

    fn resolve_const_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
        let mut n = expr.value.clone();
        n.unit = expr.node_data.unit.clone();
        Value::from_number(n, &expr.get_range())
    }

    fn resolve_bin_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();

        let error_cnt_before = self.errors.len();
        let expr1 = self.resolve_node(&expr.expr1);
        let expr2 = self.resolve_node(&expr.expr2);
        if error_cnt_before != self.errors.len() {
            for error in &self.errors[error_cnt_before..] {
                if error.id != ErrorId::None { //TODO: should be check if the error is a 'real' error and not a warning.
                    return Value::error(&expr.get_range());
                }
            }
        }

        let operator_type = OperatorType::from(&expr.op.kind);
        let op_id = operator_id_from(variant_to_value_type(&expr1.variant), operator_type, variant_to_value_type(&expr2.variant));
        if !self.globals.exists_operator(op_id) {
            let op_str = operator_type.to_string();
            let val_type1 = variant_to_value_type(&expr1.variant).to_string();
            let val_type2 = variant_to_value_type(&expr2.variant).to_string();
            return self.add_error(ErrorId::NoOp, expr.get_range().clone(), &[&op_str, &val_type1, &val_type2], Value::error(&expr.get_range()));
        }

        let args = vec![expr1, expr2];
        let range = Range { source_index: 0, start: 0, end: 0};

        let result = (self.globals.get_operator(op_id).unwrap())(&self.globals, &args, &range);
        result //TODO: add errors
    }
}