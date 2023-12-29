pub mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;
pub mod unit;

use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;
use crate::errors::{Error, ErrorId};
use crate::functions::FunctionDef;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, ConstExpr, IdExpr, ListExpr, Node, PostfixExpr, Statement};
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::resolver::value::{Value, variant_to_value_type};
use crate::resolver::value::Variant::Number;
use crate::tokenizer::cursor::Range;

pub struct Resolver {
    pub scope: Rc<RefCell<Scope>>,
    pub results: Vec<Value>,
    pub errors: Vec<Error>,
    //date_format: DateFormat,
}

pub fn add_error(errors: &mut Vec<Error>, id: ErrorId, range: Range, arg1: &str, mut value: Value) -> Value {
    value.has_errors = true;
    errors.push(Error::build_1_arg(id, range, arg1));
    value
}


impl Resolver {

    pub fn resolve(&mut self, statements: &Vec<Box<Statement>>) -> Option<Value> {
        for stmt in statements {
            let result = self.resolve_node(&stmt.node);
            self.results.push(result);
        };
        let Some(result) = self.results.last() else {
            return None
        };
        Some(result.clone())
    }

    pub fn add_error(&mut self, id: ErrorId, range: Range, arg1: &str, value: Value) -> Value {
        add_error(&mut self.errors, id, range,  arg1, value)
   }

    pub fn resolve_node(&mut self, expr: &Box<dyn Node>) -> Value {
        match expr.as_any().type_id() {
            t if TypeId::of::<ConstExpr>() == t => { self.resolve_const_expr(expr) },
            t if TypeId::of::<BinExpr>() == t => { self.resolve_bin_expr(expr) },
            t if TypeId::of::<IdExpr>() == t => { self.resolve_id_expr(expr) },
            t if TypeId::of::<AssignExpr>() == t => { self.resolve_assign_expr(expr) },
            t if TypeId::of::<PostfixExpr>() == t => { self.resolve_postfix_expr(expr) },
            t if TypeId::of::<CallExpr>() == t => { self.resolve_call_expr(expr) },
            _ => { self.add_error(ErrorId::Expected, Range::none(), "It's a dunno...", Value::error()) },
        }
    }

    // fn get_trait_func(&mut self, name: &str) -> Option<impl &FunctionDef> {
    //     let global_function_def = self.scope.globals.global_function_defs.get(name);
    //     let local_function_def = self.scope.local_function_defs.get(name);
    //     if let Some(f) = global_function_def {
    //         Some(f)
    //     } else {
    //         if let Some(f) = local_function_def {
    //             Some(f)
    //         } else {
    //             None
    //         }
    //     }
    // }

    fn resolve_call_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let call_expr = expr.as_any().downcast_ref::<CallExpr>().unwrap();
        if call_expr.node_data.has_errors {
            return Value::error();
        };
        let global_function_def = self.scope.borrow().globals.global_function_defs.contains_key(call_expr.function_name.as_str());
        let local_function_def = self.scope.borrow().local_function_defs.contains_key(call_expr.function_name.as_str()); //TODO: replace with a recursove function over parent scopes.
        if !global_function_def && !local_function_def {
            return self.add_error(ErrorId::FuncNotDef, call_expr.function_name_range.clone(), &call_expr.function_name, Value::error());
        };

        let arguments = call_expr.arguments.as_any().downcast_ref::<ListExpr>().unwrap();

        //TODO: try trait objects. (trait references, actually)
        let mut arg_count_wrong = false;
        if global_function_def {
            if !self.scope.borrow().globals.global_function_defs.get(call_expr.function_name.as_str()).unwrap().is_correct_arg_count(arguments.nodes.len()) {
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
            return self.add_error(ErrorId::FuncArgWrong, call_expr.function_name_range.clone(), &call_expr.function_name, Value::error());
        };
        let mut arg_values: Vec<Value> = Vec::new();
        for arg in &arguments.nodes {
            let value = self.resolve_node(arg);
            if value.has_errors {
                return Value::error();
            }
            arg_values.push(value);
        };

        if global_function_def {
            return self.scope.borrow().globals.global_function_defs.get(call_expr.function_name.as_str()).unwrap()
                .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors);
        } else {
            if local_function_def {
                return self.scope.borrow().local_function_defs.get(call_expr.function_name.as_str()).unwrap()
                    .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors);
            } else {
                panic!("TODO");
            }
        }
        //TODO: apply units.
    }

    fn resolve_postfix_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
        let mut result = self.resolve_node(&expr.node);
        match &mut result.variant {
            Number { number, .. } => {
                let id = self.scope.borrow().globals.get_text(&expr.postfix_id.range).to_string();
                number.unit = Unit { id: id, range: None }
            },
            _ => panic!("TODO in resolve_postfix_expr")
        };
        result
    }

    fn resolve_assign_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
        let mut value = self.resolve_node(&expr.expr);
        let id_str = self.scope.borrow().globals.get_text(&expr.id.range).to_string();
        if !self.scope.borrow().variables.contains_key(&id_str) {
            //TODO: test if id  is unit or function or redef of constant.
            self.scope.borrow_mut().variables.insert(id_str, value.clone());
        }
        value.id = Some(expr.id.range.clone()); //add id here to avoid adding id to the self.scope.variables.
        value
    }

    fn resolve_id_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<IdExpr>().unwrap();
        let id = self.scope.borrow().globals.get_text(&expr.id.range).to_string();
        let var_exists = self.scope.borrow().variables.contains_key(&id);
        if var_exists  {
            self.scope.borrow().variables.get(&id).unwrap().clone()
        } else {
            self.add_error(ErrorId::VarNotDef, expr.id.range.clone(), &id, Value::error())
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
        let op_id = operator_id_from(variant_to_value_type(&expr1.variant), operator_type, variant_to_value_type(&expr2.variant));
        let exists_operator = self.scope.borrow().globals.exists_operator(op_id);

        let args = vec![expr1, expr2];
        let range = Range { source_index: 0, start: 0, end: 0};

        let result = (self.scope.borrow().globals.get_operator(op_id).unwrap())(&self.scope.borrow().globals, &args, &range);
        result //TODO: add errors
    }
}