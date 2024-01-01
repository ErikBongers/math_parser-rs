use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::resolver::scope::Scope;
use crate::errors::{Error, ErrorId};
use crate::parser::CodeBlock;
use crate::parser::nodes::{FunctionDefExpr};
use crate::resolver::{add_error, Resolver};
use crate::resolver::globals::Globals;
use crate::resolver::value::{NumberFormat, Value};
use crate::resolver::value::Variant;
use crate::tokenizer::cursor::{Number, Range};

pub trait FunctionDef {
    fn is_correct_arg_count(&self, cnt: usize) -> bool;
}

pub struct GlobalFunctionDef {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    execute: fn(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value,
}

pub struct CustomFunctionDef {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    pub execute: fn(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value,
    pub code_block: CodeBlock,
    pub function_def_expr: FunctionDefExpr,
}

impl GlobalFunctionDef {
    pub fn call(&self, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(Some(&self), None, scope, args, range, errors, globals)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), &[""], Value::error(range))
        }
    }
}

impl FunctionDef for GlobalFunctionDef {
    fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args //TODO: add inline hint
    }
}

impl FunctionDef for CustomFunctionDef {
    fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args //TODO: add inline hint
    }
}

impl CustomFunctionDef {
    pub fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args //TODO: add inline hint
    }

    pub fn call(&self, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(None, Some(&self), scope, args, range, errors, globals)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), &[""], Value::error(range))
        }
    }
}


#[derive(Clone)]
pub struct FunctionView {
    pub ids: HashSet<String>,
}

impl FunctionView {
    pub fn from_globals(globals: &Globals) -> Self {
        let mut view = FunctionView {
           ids: HashSet::new()
        };
        for f in globals.global_function_defs.keys() {
            view.ids.insert(f.to_string());
        }
        view
    }
}

//TODO: add global functions.

pub fn create_global_function_defs() -> HashMap<String, GlobalFunctionDef> {
    let defs: HashMap<String, GlobalFunctionDef> = HashMap::from( [
        ("abs".to_string(), GlobalFunctionDef { name: "abs".to_string(), min_args: 1, max_args: 1, execute: abs}),
        ("inc".to_string(), GlobalFunctionDef { name: "inc".to_string(), min_args: 1, max_args: 1, execute: inc}),
        ("dec".to_string(), GlobalFunctionDef { name: "dec".to_string(), min_args: 1, max_args: 1, execute: dec}),
        ("factorial".to_string(), GlobalFunctionDef { name: "factors".to_string(), min_args: 1, max_args: 1, execute: factorial}),
    ]);
    defs
}


fn match_arg_number<'a>(global_function_def: Option<&GlobalFunctionDef>, args: &'a Value, range: &Range, errors: &mut Vec<Error>) -> Option<&'a Number> {
    let function_def = global_function_def.unwrap();
    let Variant::Numeric { number, .. } = &args.variant else {
        add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &[&function_def.name], Value::error(range));
        return None;
    };
    Some(number)
}

fn abs(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range);
    };
    let mut sig = number.significand;
    if sig < 0.0 {
        sig = -sig;
    }
    Value::from_number(Number {significand: sig, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

fn inc(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range);
    };
    Value::from_number(Number {significand: number.significand+1.0, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

fn dec(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range);
    };
    Value::from_number(Number {significand: number.significand-1.0, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

fn factorial(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range);
    };
    let size = number.significand;
    if( size < 0.0) {
        errors.push(Error { id: ErrorId::Expected, message: "factorial argument should be a non-negative integer".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range);
    }
    if( size !=  (size as i64) as f64) {
        errors.push(Error { id: ErrorId::Expected, message: "factorial argument should be an integer value".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range);
    }
    let size = size as i64;
    let val = (1..=size).reduce(|v, i| v*i).unwrap_or(1);
    Value::from_number(Number {significand: val as f64, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}


pub fn execute_custom_function(global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let mut function_def = local_function_def.unwrap();
    let mut param_variables = HashMap::<String, Value>::new();

    //Note that number of args has already been checked in call()
    for (i, arg) in args.iter().enumerate() {
        param_variables.insert(function_def.function_def_expr.arg_names[i].clone(), arg.clone()); //TODO: clone or move?
    }

    function_def.code_block.scope.borrow_mut().variables.extend(param_variables);
    let mut resolver = Resolver {globals, scope: function_def.code_block.scope.clone(), results: Vec::new(), errors};
    let result = resolver.resolve(&function_def.code_block.statements);
    let Some(result) = result else {
        add_error(errors, ErrorId::FuncNoBody, range.clone(),&[&function_def.name], Value::error(range));
        return Value::error(range)
    };
    result
}

