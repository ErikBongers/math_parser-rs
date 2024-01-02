use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::resolver::scope::Scope;
use crate::errors::{Error, ErrorId};
use crate::parser::CodeBlock;
use crate::parser::nodes::{FunctionDefExpr};
use crate::resolver::{add_error, Resolver};
use crate::resolver::globals::Globals;
use crate::resolver::unit::Unit;
use crate::resolver::value::{NumberFormat, Value};
use crate::resolver::value::Variant;
use crate::tokenizer::cursor::{Number, Range};

pub trait FunctionDef {
    fn is_correct_arg_count(&self, cnt: usize) -> bool;
    fn get_name(&self) -> &str;
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
    #[inline]
    fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl FunctionDef for CustomFunctionDef {
    #[inline]
    fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl CustomFunctionDef {
    #[inline]
    pub fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args
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

pub fn create_global_function_defs() -> HashMap<String, GlobalFunctionDef> {
    let defs: HashMap<String, GlobalFunctionDef> = HashMap::from( [
        ("abs".to_string(), GlobalFunctionDef { name: "abs".to_string(), min_args: 1, max_args: 1, execute: abs}),
        ("round".to_string(), GlobalFunctionDef { name: "round".to_string(), min_args: 1, max_args: 1, execute: round}),
        ("trunc".to_string(), GlobalFunctionDef { name: "trunc".to_string(), min_args: 1, max_args: 1, execute: trunc}),
        ("floor".to_string(), GlobalFunctionDef { name: "floor".to_string(), min_args: 1, max_args: 1, execute: floor}),
        ("ceil".to_string(), GlobalFunctionDef { name: "ceil".to_string(), min_args: 1, max_args: 1, execute: ceil}),
        ("sqrt".to_string(), GlobalFunctionDef { name: "sqrt".to_string(), min_args: 1, max_args: 1, execute: sqrt}),
        ("inc".to_string(), GlobalFunctionDef { name: "inc".to_string(), min_args: 1, max_args: 1, execute: inc}),
        ("dec".to_string(), GlobalFunctionDef { name: "dec".to_string(), min_args: 1, max_args: 1, execute: dec}),
        ("factorial".to_string(), GlobalFunctionDef { name: "factors".to_string(), min_args: 1, max_args: 1, execute: factorial}),
        ("sin".to_string(), GlobalFunctionDef { name: "sin".to_string(), min_args: 1, max_args: 1, execute: sin}),
        ("cos".to_string(), GlobalFunctionDef { name: "cos".to_string(), min_args: 1, max_args: 1, execute: cos}),
        ("tan".to_string(), GlobalFunctionDef { name: "tan".to_string(), min_args: 1, max_args: 1, execute: tan}),
        ("asin".to_string(), GlobalFunctionDef { name: "asin".to_string(), min_args: 1, max_args: 1, execute: asin}),
        ("acos".to_string(), GlobalFunctionDef { name: "acos".to_string(), min_args: 1, max_args: 1, execute: acos}),
        ("atan".to_string(), GlobalFunctionDef { name: "atan".to_string(), min_args: 1, max_args: 1, execute: atan}),
        ("sum".to_string(), GlobalFunctionDef { name: "sum".to_string(), min_args: 1, max_args: 999, execute: sum}),
        ("avg".to_string(), GlobalFunctionDef { name: "avg".to_string(), min_args: 1, max_args: 999, execute: avg}),
        ("max".to_string(), GlobalFunctionDef { name: "max".to_string(), min_args: 1, max_args: 999, execute: max}),
        ("min".to_string(), GlobalFunctionDef { name: "min".to_string(), min_args: 1, max_args: 999, execute: min}),
    ]);
    defs
}


fn abs(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.abs(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn round(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.round(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn trunc(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.trunc(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn floor(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.floor(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn ceil(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.ceil(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn sqrt(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.sqrt(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}
fn sum(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def.unwrap(), &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().reduce(|tot, num| tot+num).unwrap_or(0.0)
    })
}

fn max(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def.unwrap(), &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
    })
}

fn min(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def.unwrap(), &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
    })
}

fn avg(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def.unwrap(), &args, range, errors, globals, |num_vec| {
        let mut val = num_vec.into_iter().reduce(|tot, num| tot + num).unwrap_or(0.0);
        val / args.len() as f64
    })
}

fn with_num_vec(function_def: &dyn FunctionDef, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals, func: impl Fn(Vec<f64>) -> f64) -> Value {
    if let Some(num_vec) = to_num_vec(function_def, &args, range, errors, globals) {
        let mut val = Value::from_number(Number {significand: func(num_vec), exponent: 0, unit: Unit::none(), fmt: NumberFormat::Dec }, range);
        let Some(number) = match_arg_number(function_def, &args[0], range, errors) else { return Value::error(range) };
        let si_id = globals.unit_defs.get(&number.unit.id).unwrap().si_id;
        val.as_number().unwrap().unit.id = si_id.to_string();
        val
    } else { Value::error(range) }
}

fn to_num_vec(function_def: &dyn FunctionDef, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Option<Vec<f64>> {
    Some(to_num_iter(function_def.get_name(), args, range, errors, globals))
}

fn match_arg_number<'a>(function_def: &dyn FunctionDef, args: &'a Value, range: &Range, errors: &mut Vec<Error>) -> Option<&'a Number> {
    let Variant::Numeric { number, .. } = &args.variant else {
        add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &[function_def.get_name()], Value::error(range));
        return None;
    };
    Some(number)
}

//TODO: try to figure out how to return an iterator instead of a vec. Not easy.
//https://stackoverflow.com/questions/27535289/what-is-the-correct-way-to-return-an-iterator-or-any-other-trait
//the closure in map should probably be converted into a 'real' function.
fn to_num_iter(function_name: &str, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Vec<f64> {
    let num_iter: Vec<f64> = args.iter()
        .map(|value|
            {
                if let Variant::Numeric { number, .. } = &value.variant {
                    number.to_si(globals)
                } else {
                    add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &[function_name], Value::error(range));
                    0.0
                }
            }).collect();
    num_iter
}

fn inc(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else {
        return Value::error(range);
    };
    Value::from_number(Number {significand: number.significand+1.0, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

fn dec(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else {
        return Value::error(range);
    };
    Value::from_number(Number {significand: number.significand-1.0, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

fn sin(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { range: None, id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals); //TODO: we don't need to make another number. Just convert units for a f64.
    }
    Value::from_number(Number {significand: number.significand.sin(), exponent: number.exponent, unit: Unit::none(), fmt: NumberFormat::Dec }, range)
}

fn cos(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { range: None, id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals);     }
    Value::from_number(Number {significand: number.significand.cos(), exponent: number.exponent, unit: Unit::none(), fmt: NumberFormat::Dec }, range)
}

fn tan(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { range: None, id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals);
    }
    Value::from_number(Number {significand: number.significand.tan(), exponent: number.exponent, unit: Unit::none(), fmt: NumberFormat::Dec }, range)
}

fn asin(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.asin(), exponent: number.exponent, unit: Unit { id: "rad".to_string(), range: Some(range.clone())}, fmt: NumberFormat::Dec }, range)
}
fn acos(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.acos(), exponent: number.exponent, unit: Unit { id: "rad".to_string(), range: Some(range.clone())}, fmt: NumberFormat::Dec }, range)
}
fn atan(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else { return Value::error(range); };
    Value::from_number(Number {significand: number.significand.atan(), exponent: number.exponent, unit: Unit { id: "rad".to_string(), range: Some(range.clone())}, fmt: NumberFormat::Dec }, range)
}


fn factorial(global_function_def: Option<&GlobalFunctionDef>, _local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def.unwrap(), &args[0], range, errors) else {
        return Value::error(range);
    };
    let size = number.significand;
    if size < 0.0 {
        errors.push(Error { id: ErrorId::Expected, message: "factorial argument should be a non-negative integer".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range);
    }
    if size !=  (size as i64) as f64 {
        errors.push(Error { id: ErrorId::Expected, message: "factorial argument should be an integer value".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range);
    }
    let size = size as i64;
    let val = (1..=size).reduce(|v, i| v*i).unwrap_or(1);
    Value::from_number(Number {significand: val as f64, exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range)
}

pub fn execute_custom_function(_global_function_def: Option<&GlobalFunctionDef>, local_function_def: Option<&CustomFunctionDef>, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
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

