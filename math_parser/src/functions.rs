use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use chrono::{Datelike, Utc};
use crate::resolver::scope::Scope;
use crate::errors::{Error, ErrorId};
use crate::date::{Timepoint, Day};
use crate::date::{month_from_int};
use crate::parser::nodes::{CodeBlock, FunctionDefExpr};
use crate::resolver::{add_error, Resolver};
use crate::globals::Globals;
use crate::number::Number;
use crate::resolver::unit::Unit;
use crate::resolver::value::{NumberFormat, Value};
use crate::resolver::value::Variant;
use crate::tokenizer::cursor::Range;

pub trait FunctionDef {
    fn is_correct_arg_count(&self, cnt: usize) -> bool;
    fn get_name(&self) -> &str;
    fn call(&self, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value;
    fn get_min_args(&self) -> usize;
}

#[derive(PartialEq)]
pub enum FunctionType { Trig, Arithm, Date}

pub struct GlobalFunctionDef {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    pub func_type: FunctionType,
    execute: fn(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value,
}

pub struct CustomFunctionDef {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    pub execute: fn(local_function_def: &CustomFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value,
    pub code_block: CodeBlock,
    pub function_def_expr: FunctionDefExpr,
}

impl FunctionDef for GlobalFunctionDef {
    #[inline]
    fn is_correct_arg_count(&self, cnt: usize) -> bool {
        self.min_args <= cnt && cnt <= self.max_args
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn call(&self, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(&self, scope, args, range, errors, globals)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), &[""], Value::error(range.clone()))
        }
    }

    fn get_min_args(&self) -> usize {
        self.min_args
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

    fn call(&self, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
        if self.is_correct_arg_count(args.len()) {
            (self.execute)(&self, scope, args, range, errors, globals)
        } else {
            add_error(errors, ErrorId::FuncArgWrong, range.clone(), &[""], Value::error(range.clone()))
        }
    }

    fn get_min_args(&self) -> usize {
        self.min_args
    }
}

#[derive(Clone)]
pub struct FunctionView {
    pub ids: HashSet<String>,
}

impl FunctionView {

    pub fn new() -> Self {
        FunctionView {
            ids: HashSet::new()
        }
    }

    pub fn add_all(&mut self, function_defs: &HashMap<String, GlobalFunctionDef>) {
        self.ids.extend(function_defs.keys().map(|k| k.clone()));
    }

    pub fn add_type(&mut self, function_type: FunctionType, globals: &Globals) {
        self.ids.extend(
            globals.global_function_defs
                .values()
                .filter(|f|
                    f.func_type == function_type)
                .map(|f| f.name.clone()));
    }

    pub fn remove_type(&mut self, function_type: FunctionType, globals: &Globals) {
        self.ids.retain(|id|
            globals.global_function_defs
                .get(id)
                .map_or_else(|| true, |f|
                    f.func_type != function_type));
    }
}

pub fn create_global_function_defs() -> HashMap<String, GlobalFunctionDef> {
    let defs: HashMap<String, GlobalFunctionDef> = HashMap::from( [
        ("abs".to_string(), GlobalFunctionDef { name: "abs".to_string(), min_args: 1, max_args: 1, execute: abs, func_type: FunctionType::Arithm}),
        ("inc".to_string(), GlobalFunctionDef { name: "inc".to_string(), min_args: 1, max_args: 1, execute: inc, func_type: FunctionType::Arithm}),
        ("dec".to_string(), GlobalFunctionDef { name: "dec".to_string(), min_args: 1, max_args: 1, execute: dec, func_type: FunctionType::Arithm}),
        ("sqrt".to_string(), GlobalFunctionDef { name: "sqrt".to_string(), min_args: 1, max_args: 1, execute: sqrt, func_type: FunctionType::Arithm}),

        ("round".to_string(), GlobalFunctionDef { name: "round".to_string(), min_args: 1, max_args: 1, execute: round, func_type: FunctionType::Arithm}),
        ("trunc".to_string(), GlobalFunctionDef { name: "trunc".to_string(), min_args: 1, max_args: 1, execute: trunc, func_type: FunctionType::Arithm}),
        ("floor".to_string(), GlobalFunctionDef { name: "floor".to_string(), min_args: 1, max_args: 1, execute: floor, func_type: FunctionType::Arithm}),
        ("ceil".to_string(), GlobalFunctionDef { name: "ceil".to_string(), min_args: 1, max_args: 1, execute: ceil, func_type: FunctionType::Arithm}),

        ("factorial".to_string(), GlobalFunctionDef { name: "factors".to_string(), min_args: 1, max_args: 1, execute: factorial, func_type: FunctionType::Arithm}),

        ("sin".to_string(), GlobalFunctionDef { name: "sin".to_string(), min_args: 1, max_args: 1, execute: sin, func_type: FunctionType::Trig}),
        ("cos".to_string(), GlobalFunctionDef { name: "cos".to_string(), min_args: 1, max_args: 1, execute: cos, func_type: FunctionType::Trig}),
        ("tan".to_string(), GlobalFunctionDef { name: "tan".to_string(), min_args: 1, max_args: 1, execute: tan, func_type: FunctionType::Trig}),
        ("asin".to_string(), GlobalFunctionDef { name: "asin".to_string(), min_args: 1, max_args: 1, execute: asin, func_type: FunctionType::Trig}),
        ("acos".to_string(), GlobalFunctionDef { name: "acos".to_string(), min_args: 1, max_args: 1, execute: acos, func_type: FunctionType::Trig}),
        ("atan".to_string(), GlobalFunctionDef { name: "atan".to_string(), min_args: 1, max_args: 1, execute: atan, func_type: FunctionType::Trig}),

        ("sum".to_string(), GlobalFunctionDef { name: "sum".to_string(), min_args: 2, max_args: 999, execute: sum, func_type: FunctionType::Arithm}),
        ("avg".to_string(), GlobalFunctionDef { name: "avg".to_string(), min_args: 2, max_args: 999, execute: avg, func_type: FunctionType::Arithm}),
        ("max".to_string(), GlobalFunctionDef { name: "max".to_string(), min_args: 2, max_args: 999, execute: max, func_type: FunctionType::Arithm}),
        ("min".to_string(), GlobalFunctionDef { name: "min".to_string(), min_args: 2, max_args: 999, execute: min, func_type: FunctionType::Arithm}),

        ("reverse".to_string(), GlobalFunctionDef { name: "reverse".to_string(), min_args: 2, max_args: 999, execute: reverse, func_type: FunctionType::Arithm}),
        ("sort".to_string(), GlobalFunctionDef { name: "sort".to_string(), min_args: 2, max_args: 999, execute: sort, func_type: FunctionType::Arithm}),
        ("first".to_string(), GlobalFunctionDef { name: "first".to_string(), min_args: 2, max_args: 999, execute: first, func_type: FunctionType::Arithm}),
        ("last".to_string(), GlobalFunctionDef { name: "last".to_string(), min_args: 2, max_args: 999, execute: last, func_type: FunctionType::Arithm}),

        ("factors".to_string(), GlobalFunctionDef { name: "factors".to_string(), min_args: 1, max_args: 1, execute: factors, func_type: FunctionType::Arithm}),
        ("primes".to_string(), GlobalFunctionDef { name: "primes".to_string(), min_args: 1, max_args: 1, execute: primes, func_type: FunctionType::Arithm}),

        ("now".to_string(), GlobalFunctionDef { name: "now".to_string(), min_args: 0, max_args: 0, execute: now, func_type: FunctionType::Date}),
        ("date".to_string(), GlobalFunctionDef { name: "date".to_string(), min_args: 3, max_args: 3, execute: date_func, func_type: FunctionType::Date}),
    ]);
    defs
}


fn abs(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.significand.abs(), exponent: number.exponent, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn round(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().round(), exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn trunc(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().trunc(), exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn floor(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().floor(), exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn ceil(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().ceil(), exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn sqrt(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().sqrt(), exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}
fn sum(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let mut res = with_num_vec(global_function_def, &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().reduce(|tot, num| tot+num).unwrap_or(0.0)
    });
    res.as_number_mut().unwrap().convert_to_unit(&args[0].as_number().unwrap().unit, &scope.borrow().units_view, range, errors, globals);
    res
}

fn max(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def, &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
    })
}

fn min(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def, &args, range, errors, globals, |num_vec| {
        num_vec.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
    })
}

fn avg(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    with_num_vec(global_function_def, &args, range, errors, globals, |num_vec| {
        let val = num_vec.into_iter().reduce(|tot, num| tot + num).unwrap_or(0.0);
        val / args.len() as f64
    })
}

fn reverse(_global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let reversed: Vec<Value> = args.into_iter().rev().map(|value| {
       value.clone()
    }).collect();
    Value {
        id: None,
        stmt_range: range.clone(),
        variant: Variant::List { values: reversed },
        has_errors: false,
    }
}

fn sort(_global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let mut sorted: Vec<Value> = args.clone();
    sorted.sort_by(|a, b| a.sortable_value().partial_cmp(&b.sortable_value()).unwrap());
    Value {
        id: None,
        stmt_range: range.clone(),
        variant: Variant::List { values: sorted },
        has_errors: false,
    }
}

fn build_factors(val: f64) -> Vec<f64> { //generators not possible in rust.
    let int = val.trunc() as i32;
    let half = (val/2.0).trunc() as i32;
    let mut list = Vec::new();
    for i in 2..=half {
        if int%i == 0 {
            list.push(i as f64);
        }
    }
    list
}

fn floats_to_values(range: &Range, factors: &Vec<f64>) -> Vec<Value> {
    let list = factors.iter()
        .map(|i|
            Value {
                id: None,
                stmt_range: range.clone(),
                variant: Variant::Numeric {
                    number: Number {
                        significand: *i,
                        exponent: 0,
                        unit: Unit::none(),
                        fmt: NumberFormat::Dec,
                    },
                },
                has_errors: false,
            }
        )
        .collect();
    list
}

fn factors(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    let factors = build_factors(number.to_double());
    let list = floats_to_values(range, &factors);
    Value {
        id: None,
        stmt_range: range.clone(),
        variant: Variant::List { values: list },
        has_errors: false,
    }
}

fn primes(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    let factors = build_factors(number.to_double());
    let primez = factors.into_iter().filter(|&f| {
        PRIMES.binary_search(&(f as i32)).is_ok()
    }).collect();
    let list = floats_to_values(range, &primez);
    Value {
        id: None,
        stmt_range: range.clone(),
        variant: Variant::List { values: list },
        has_errors: false,
    }
}

const PRIMES: [i32; 168] = [
2,3,5,7,11,13,17,19,23,
29,31,37,41,43,47,53,59,61,67,
71,73,79,83,89,97,101,103,107,109,
113,127,131,137,139,149,151,157,163,167,
173,179,181,191,193,197,199,211,223,227,
229,233,239,241,251,257,263,269,271,277,
281,283,293,307,311,313,317,331,337,347,
349,353,359,367,373,379,383,389,397,401,
409,419,421,431,433,439,443,449,457,461,
463,467,479,487,491,499,503,509,521,523,
541,547,557,563,569,571,577,587,593,599,
601,607,613,617,619,631,641,643,647,653,
659,661,673,677,683,691,701,709,719,727,
733,739,743,751,757,761,769,773,787,797,
809,811,821,823,827,829,839,853,857,859,
863,877,881,883,887,907,911,919,929,937,
941,947,953,967,971,977,983,991,997
];

fn first(_global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, _range: &Range, _errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    args.first().unwrap().clone()
}

fn last(_global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, _range: &Range, _errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    args.last().unwrap().clone()
}

fn with_num_vec(function_def: &dyn FunctionDef, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals, func: impl Fn(Vec<f64>) -> f64) -> Value {
    let num_vec: Vec<f64> = to_num_iter(function_def.get_name(), &args, range, errors, globals).collect();
    let mut val = Value::from_number(Number {significand: func(num_vec), exponent: 0, unit: Unit::none(), fmt: NumberFormat::Dec }, range.clone());
    let Some(number) = match_arg_number(function_def, &args[0], range, errors) else { return Value::error(range.clone()) };
    let si_id = globals.unit_defs.get(&number.unit.id).unwrap().si_id;
    val.as_number_mut().unwrap().unit.id = si_id.to_string();
    val
}

fn match_arg_number<'a>(function_def: &dyn FunctionDef, args: &'a Value, range: &Range, errors: &mut Vec<Error>) -> Option<&'a Number> {
    let Variant::Numeric { number, .. } = &args.variant else {
        add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &[function_def.get_name()], Value::error(range.clone()));
        return None;
    };
    Some(number)
}

fn to_num_iter<'a>(function_name: &'a str, args: &'a Vec<Value>, range: &'a Range, errors: &'a mut Vec<Error>, globals: &'a Globals) -> impl Iterator<Item=f64> + 'a {
    args.iter()
        .map(|value| {
            if let Variant::Numeric { number, .. } = &value.variant {
                number.to_si(globals)
            } else {
                add_error(errors, ErrorId::FuncArgWrongType, range.clone(), &[function_name], Value::error(range.clone()));
                0.0
            }
        })
}

fn inc(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range.clone());
    };
    Value::from_number(Number {significand: number.to_double()+1.0, exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}

fn dec(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range.clone());
    };
    Value::from_number(Number {significand: number.to_double()-1.0, exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}

fn sin(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals);
    }
    Value::from_number(Number {significand: number.to_double().sin(), exponent: 0, unit: Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

fn cos(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals);     }
    Value::from_number(Number {significand: number.to_double().cos(), exponent: 0, unit: Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

fn tan(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    let mut number = number.clone(); //Needed? Test it.
    if number.unit.id == "deg" {
        number.convert_to_unit(&Unit { id: "rad".to_string() }, &scope.borrow().units_view, range, errors, globals);
    }
    Value::from_number(Number {significand: number.to_double().tan(), exponent: 0, unit: Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

fn asin(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().asin(), exponent: 0, unit: Unit { id: "rad".to_string()}, fmt: NumberFormat::Dec }, range.clone())
}
fn acos(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().acos(), exponent: 0, unit: Unit { id: "rad".to_string()}, fmt: NumberFormat::Dec }, range.clone())
}
fn atan(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else { return Value::error(range.clone()); };
    Value::from_number(Number {significand: number.to_double().atan(), exponent: 0, unit: Unit { id: "rad".to_string()}, fmt: NumberFormat::Dec }, range.clone())
}


fn factorial(global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let Some(number) = match_arg_number(global_function_def, &args[0], range, errors) else {
        return Value::error(range.clone());
    };
    let size = number.to_double();
    if size < 0.0 {
        errors.push(Error { id: ErrorId::ValueError, message: "Factorial argument should be a non-negative integer".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range.clone());
    }
    if size !=  (size as i64) as f64 {
        errors.push(Error { id: ErrorId::ValueError, message: "Factorial argument should be an integer value".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range.clone());
    }
    if size > 20.0 {
        errors.push(Error { id: ErrorId::ValueError, message: "Factorial argument should not be larger than 20 or it will produce a too large number.".to_string(), range: range.clone(), stack_trace: None,});
        return Value::error(range.clone());
    }
    let size = size as i64;
    let val = (1..=size).reduce(|v, i| v*i).unwrap_or(1);
    Value::from_number(Number {significand: val as f64, exponent: 0, unit: number.unit.clone(), fmt: NumberFormat::Dec }, range.clone())
}

pub fn execute_custom_function(local_function_def: &CustomFunctionDef, _scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
    let mut param_variables = HashMap::<String, Value>::new();

    //Note that number of args has already been checked in call()
    for (i, arg) in args.iter().enumerate() {
        param_variables.insert(local_function_def.function_def_expr.arg_names[i].clone(), arg.clone());
    }

    local_function_def.code_block.scope.borrow_mut().variables.extend(param_variables);
    let mut resolver = Resolver {globals, scope: local_function_def.code_block.scope.clone(), results: Vec::new(), errors, muted: true, current_statement_muted: false};
    let result = resolver.resolve_to_result(&local_function_def.code_block.statements);
    let Some(result) = result else {
        add_error(errors, ErrorId::FuncNoBody, range.clone(),&[&local_function_def.name], Value::error(range.clone()));
        return Value::error(range.clone())
    };
    result
}

pub fn explode_args<'a>(args: &'a Vec<Value>, exploded_args: &'a mut Vec<Value>) -> &'a Vec<Value> {
    if args.len() != 1 {
        return args;
    }

    if let Variant::List{values} = &args[0].variant {
        exploded_args.clone_from(values);
        exploded_args
    } else {
        args
    }
}

fn now(_global_function_def: &GlobalFunctionDef, _scope: &Rc<RefCell<Scope>>, _args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>, _globals: &Globals) -> Value {

    let current_date = Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();

   Value::from_date(Timepoint { month: month_from_int(month as i32), day: Day::Value(day as i8), year: Some(year), errors: vec![], }, range.clone())
}

fn date_func(global_function_def: &GlobalFunctionDef, scope: &Rc<RefCell<Scope>>, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>, _globals: &Globals) -> Value {
    let mut date = Timepoint::new();

    let  idx = scope.borrow().date_format.indices();
    if args.len() < 3 {
        errors.push(Error::build(ErrorId::FuncArgWrong, range.clone(), &[global_function_def.get_name()]));
        return Value::error(range.clone());
    }
    let day = &args[idx.day];
    let month = &args[idx.month];
    let year = &args[idx.year];

    if let Some(day_num) = &day.as_number() {
        date.day = Day::Value(day_num.to_double() as i8);
    } else {
        if let Variant::Last = &day.variant {
            date.day = Day::Last;
        }
    }
    if let Some(month_num) = &month.as_number() {
        date.month = month_from_int(month_num.to_double() as i32);
    }
    if let Some(year_num) = &year.as_number() {
        date.year = Some(year_num.to_double() as i32);
    }
    if !date.is_valid() {
        errors.push(Error::build(ErrorId::InvDate, range.clone(), &[]));
    }
    Value::from_date(date, range.clone())
}
