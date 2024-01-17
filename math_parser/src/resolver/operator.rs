use std::fmt::{Display, Formatter};
use std::ops::Sub;
use crate::parser::date::Date;
use crate::resolver::globals::Globals;
use crate::resolver::unit::Unit;
use crate::resolver::value::{NumberFormat, Value, Variant};
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::token_type::TokenType;

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

pub fn operator_id_from(type1: &Variant, op: OperatorType, type2: &Variant) -> u32 {
    (type1.to_u32() *265*265) + (op as u32*265) + type2.to_u32()
}

pub fn op_num_plus_num(globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Numeric {number: ref n2, ..} = args[1].variant else { unreachable!(); };
    Value::from_number( do_term(n1, true, n2, range, &globals), range.clone())
}

pub fn op_num_min_num(globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = args[1].variant else { unreachable!(); };
    Value::from_number( do_term(n1, false, n2, range, &globals), range.clone())
}

pub fn op_num_mult_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.significand * n2.significand, exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_div_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.significand / n2.significand, exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_rem_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.to_double() % n2.to_double(), exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_mod_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: ((n1.to_double() % n2.to_double()) + n2.to_double()) % n2.to_double(), exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range.clone())
}

pub fn op_num_pow_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Numeric {number: ref n1, ..} = &args[0].variant else { unreachable!(); };
    let Variant::Numeric {number: ref n2, ..} = &args[1].variant else { unreachable!(); };
    Value::from_number(Number { significand: n1.to_double().powf(n2.to_double()), exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range.clone())
}

pub fn load_operators(globals: &mut Globals) {
    let variant_num = Variant::Numeric { number: Number::from(0.0)}; //note that even with discriminant.hash(), you'd have to have a concrete variant to generate the hash.
    let variant_date = Variant::Date { date: Date::new()};
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Plus, &variant_num), op_num_plus_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Min, &variant_num), op_num_min_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Mult, &variant_num), op_num_mult_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Div, &variant_num), op_num_div_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Remain, &variant_num), op_num_rem_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Modulo, &variant_num), op_num_mod_num);
    globals.operators.insert(operator_id_from(&variant_num, OperatorType::Power, &variant_num), op_num_pow_num);
    globals.operators.insert(operator_id_from(&variant_date, OperatorType::Min, &variant_date), op_date_min_date);
}

fn do_term(v1: &Number, adding: bool, v2: &Number, _range: &Range, globals: &Globals) -> Number {
    //if both values have units: convert them to SI before operation.
    if !v1.unit.is_empty() && !v2.unit.is_empty() {
        //TODO: don't I have to check if the ids are valid?
        let u1 = &globals.unit_defs[&v1.unit.id];
        let u2 = &globals.unit_defs[&v2.unit.id];
        if u1.property != u2.property {
            panic!("TODO: implement errors.");
        }
        let d1 = v1.to_si(&globals);
        let d2 = v2.to_si(&globals);
        let result = match adding {
            true => d1 + d2,
            false => d1 - d2
        };
        let mut num = Number::from(result);
        if globals.unit_defs.contains_key(&*v1.unit.id) {
            num.significand = globals.unit_defs[&*v1.unit.id].convert_from_si(result);
            num.exponent = 0;
            num.unit = v1.unit.clone();
            return num;
        }
        num
        //if a unit is missing, just do operation.
    } else {
        let result = match adding {
            true => v1.to_double() + v2.to_double(), //TODO: see Number.operator+ for exponents
            false => v1.to_double() - v2.to_double()
        };
        Number::from(result)
    }
}

pub fn op_date_min_date(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Date {date: ref d1, ..} = &args[0].variant else { unreachable!(); }; //has been checked.
    let Variant::Date {date: ref d2, ..} = &args[1].variant else { unreachable!(); }; //has been checked.


    let dur = d1.sub(d2);
    Value::from_duration(dur, range.clone())
}
