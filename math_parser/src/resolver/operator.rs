use std::fmt::{Display, Formatter};
use crate::errors;
use crate::errors::Error;
use crate::globals::Globals;
use crate::resolver::unit::Unit;
use crate::resolver::value::{NumberFormat, OperandType, Value, Variant};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::token_type::TokenType;
use crate::number::Number;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum OperatorType {
    Plus = 1,
    Min = 2,
    Mult = 3,
    Div = 4,
    Power = 5,
    Remain = 6,
    Modulo = 7
}

impl Display for OperatorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorType::Plus => write!(f, "+"),
            OperatorType::Min => write!(f, "-"),
            OperatorType::Mult => write!(f, "*"),
            OperatorType::Div => write!(f, "/"),
            OperatorType::Power => write!(f, "^"),
            OperatorType::Remain => write!(f, "%"),
            OperatorType::Modulo => write!(f, "%%"),
        }
    }
}

impl From<&TokenType> for OperatorType {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Plus => OperatorType::Plus,
            TokenType::Min => OperatorType::Min,
            TokenType::Mult => OperatorType::Mult,
            TokenType::Div => OperatorType::Div,
            TokenType::Power => OperatorType::Power,
            TokenType::Percent => OperatorType::Remain,
            TokenType::Modulo=> OperatorType::Modulo,
            _ => unreachable!("This is not an operator!")
        }
    }
}

pub fn operator_id_from(type1: OperandType, op: OperatorType, type2: OperandType) -> u32 {
    (type1 as u32 *265*265) + (op as u32*265) + type2 as u32
}

pub fn op_num_plus_num(globals: &Globals, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Numeric {number: ref n2, ..} = args[1].variant else { unreachable!(); };
    Value::from_number( do_term(n1, true, n2, range, &globals, errors), range.clone())
}

pub fn op_num_min_num(globals: &Globals, args: &Vec<Value>, range: &Range, errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = args[1].variant else { unreachable!(); };
    Value::from_number( do_term(n1, false, n2, range, &globals, errors), range.clone())
}

pub fn op_num_mult_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(n1 * n2, range.clone())
}

pub fn op_num_div_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(n1 / n2, range.clone())
}

pub fn op_num_rem_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.to_double() % n2.to_double(), exponent: 0, unit : Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_mod_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: ((n1.to_double() % n2.to_double()) + n2.to_double()) % n2.to_double(), exponent: 0, unit : Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_pow_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.to_double().powf(n2.to_double()), exponent: 0, unit : Unit::none(), fmt: NumberFormat::Dec }, range.clone())
}

pub fn load_operators(globals: &mut Globals) {
    use OperandType as OT;
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Plus, OT::Number), op_num_plus_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Min, OT::Number), op_num_min_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Mult, OT::Number), op_num_mult_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Div, OT::Number), op_num_div_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Remain, OT::Number), op_num_rem_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Modulo, OT::Number), op_num_mod_num);
    globals.operators.insert(operator_id_from(OT::Number, OperatorType::Power, OT::Number), op_num_pow_num);
    globals.operators.insert(operator_id_from(OT::Date, OperatorType::Min, OT::Date), op_date_min_date);
    globals.operators.insert(operator_id_from(OT::Duration, OperatorType::Min, OT::Duration), op_dur_min_dur);
    globals.operators.insert(operator_id_from(OT::Duration, OperatorType::Min, OT::Duration), op_dur_plus_dur);
    globals.operators.insert(operator_id_from(OT::Duration, OperatorType::Mult, OT::Number), op_dur_mult_num);
    globals.operators.insert(operator_id_from(OT::Duration, OperatorType::Div, OT::Number), op_dur_div_num);
}

fn do_term(v1: &Number, adding: bool, v2: &Number, range: &Range, globals: &Globals, errors: &mut Vec<Error>) -> Number {
    //if both values have units: convert them to SI before operation.
    if !v1.unit.is_empty() && !v2.unit.is_empty() {
        let Some(u1) = &globals.unit_defs.get(&v1.unit.id) else {
            errors.push(errors::unit_not_def(&v1.unit.id, v1.unit.range.as_ref().unwrap_or(range).clone()));
            return Number::from(0.0);
        };
        let Some(u2) = &globals.unit_defs.get(&v2.unit.id) else {
            errors.push(errors::unit_not_def(&v2.unit.id, v2.unit.range.as_ref().unwrap_or(range).clone()));
            return Number::from(0.0);
        };
        if u1.property != u2.property {
            errors.push(errors::unit_prop_diff(range.clone()));
        }
        let d1 = v1.to_si(&globals);
        let d2 = v2.to_si(&globals);
        let mut result = match adding {
            true => &d1 + &d2,
            false => &d1 - &d2
        };
        result.significand = globals.unit_defs[&v1.unit.id].convert_from_si(result.significand); //ok for now, but would not work for custom units as this requires Scope.
        result.unit = v1.unit.clone();
        result
    } else {
        //if a unit is missing, just do operation.
        let result = match adding {
            true => v1 + v2,
            false => v1 - v2,
        };
        //if one unit is set, use it but give a warning
        if !v1.unit.is_empty() || ! v2.unit.is_empty() {
            errors.push(errors::w_assuming_unit(v2.unit.range.as_ref().unwrap_or(range).clone()));
        }
        result
    }
}

pub fn op_date_min_date(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Date {date: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Date {date: ref d2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.

    Value::from_duration(d1 - d2, range.clone())
}

pub fn op_dur_min_dur(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Duration {duration: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Duration {duration: ref d2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.

    Value::from_duration(*d1 - *d2, range.clone())
}

pub fn op_dur_plus_dur(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Duration {duration: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Duration {duration: ref d2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.

    Value::from_duration(*d1 + *d2, range.clone())
}

pub fn op_dur_mult_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Duration {duration: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.

    Value::from_duration(*d1 * n2, range.clone())
}

pub fn op_dur_div_num(_globals: &Globals, args: &Vec<Value>, range: &Range, _errors: &mut Vec<Error>) -> Value {
    let Variant::Duration {duration: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.

    Value::from_duration(*d1 / n2, range.clone())
}
