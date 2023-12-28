use std::collections::{HashMap, HashSet};
use crate::resolver::scope::Scope;
use crate::errors::{Error, ErrorId};
use crate::parser::CodeBlock;
use crate::parser::nodes::{FunctionDefExpr};
use crate::resolver::{add_error, Resolver};
use crate::resolver::globals::Globals;
use crate::resolver::unit::{Unit};
use crate::resolver::value::Value;
use crate::resolver::value::Variant;
use crate::tokenizer::cursor::{Number, Range};

pub struct GlobalFunctionDef {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    execute: fn(function_def: &GlobalFunctionDef, scope: &mut Scope, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value,
    pub function_def_expr: Option<FunctionDefExpr>,
}

pub struct CustomFunctionDef<'a> {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    execute: fn(function_def: &CustomFunctionDef, scope: &mut Scope, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value,
    pub code_block: Option<CodeBlock<'a>>, //only for custom functions!
    pub function_def_expr: Option<FunctionDefExpr>,
}

impl GlobalFunctionDef {
    pub fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args //TODO: add inline hint
    }

    pub fn call(&self, scope: &mut Scope, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(&self, scope, args, range, errors)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), "", Value::error())
        }
    }
}

impl<'a> CustomFunctionDef<'a> {
    pub fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args //TODO: add inline hint
    }

    pub fn call(&self, scope: &mut Scope, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(&self, scope, args, range, errors)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), "", Value::error())
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

pub fn create_global_function_defs() -> HashMap<&'static str, GlobalFunctionDef> {
    let defs: HashMap<&'static str, GlobalFunctionDef> = HashMap::from( [
        ("abs", GlobalFunctionDef { name: "abs".to_string(), min_args: 1, max_args: 1, execute: abs, function_def_expr: None})
    ]);
    defs
}


fn abs(function_def: &GlobalFunctionDef, scope: &mut Scope, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
    let Variant::Number { number, .. } = &args[0].variant else {
        add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &function_def.name, Value::error());
        return Value::error();
    };
    let mut sig = number.significand;
    if sig < 0.0 {
        sig = -sig;
    }
    Value::from(Number {significand: sig, exponent: number.exponent, unit: number.unit.clone() })
}

fn execute_custom_function<'a>(function_def: &CustomFunctionDef, scope: &'a mut Scope<'a>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
    let mut param_variables = HashMap::<String, Value>::new();

    //Note that number of args has already been checked in call()
    for (i, arg) in args.iter().enumerate() {
        param_variables.insert(function_def.function_def_expr.as_ref().unwrap().arg_names[i].clone(), arg.clone()); //TODO: clone or move?
    }
    scope.variables.extend(param_variables);
    let mut resolver = Resolver { scope, results: Vec::new(), errors: Vec::new()};
    let result = resolver.resolve(&function_def.code_block.as_ref().unwrap().statements);
    let Some(result) = result else {
        add_error(errors, ErrorId::FuncNoBody, range.clone(),&function_def.name, Value::error());
        return Value::error()
    };
    result
}

