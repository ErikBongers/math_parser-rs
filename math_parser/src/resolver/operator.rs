use crate::resolver::globals::Globals;
use crate::resolver::unit::{Unit, UnitsView};
use crate::resolver::value::{NumberFormat, Value, ValueType, Variant};
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::token_type::TokenType;

#[repr(u8)]
pub enum OperatorType {
    Plus = 1,
    Min = 2,
    Mult = 3,
    Div = 4,
    Power = 5,
    Remain = 6,
    Modulo = 7
}

impl From<&TokenType> for OperatorType {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Plus => OperatorType::Plus,
            TokenType::Min => OperatorType::Min,
            TokenType::Mult => OperatorType::Mult,
            TokenType::Div => OperatorType::Div,
            TokenType::Power => OperatorType::Power,
            //TODO TokenType::Remain => OperatorType::Remain,
            TokenType::Modulo=> OperatorType::Modulo,
            _ => unreachable!("This is not an operator!")
        }
    }
}

pub fn operator_id_from(type1: ValueType, op: OperatorType, type2: ValueType) -> u32 {
    (type1 as u32*265*265) + (op as u32*265) + type2 as u32
}

pub fn op_num_plus_num(globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Number{number: ref n1, ..} = args[0].variant else { unreachable!("has been checked."); };
    let Variant::Number{number: ref n2, ..} = args[1].variant else { unreachable!("has been checked."); };
    Value::from_number( do_term(&globals.units_view, n1, true, n2, range, &globals), &range)
}

pub fn op_num_min_num(globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    let Variant::Number{number: ref n1, ..} = args[0].variant else { unreachable!("has been checked."); };
    let Variant::Number{number: ref n2, ..} = args[1].variant else { unreachable!("has been checked."); };
    Value::from_number( do_term(&globals.units_view, n1, false, n2, range, &globals), &range)
}

pub fn op_num_mult_num(_globals: &Globals, args: &Vec<Value>, range: &Range) -> Value {
    //TODO: this is quick and dirty
    //TODO number of args already checked?
    let val1 = &args[0];
    let val2 = &args[1];
    let Variant::Number{number: ref n1, ..} = val1.variant else { unreachable!("has been checked."); };
    let Variant::Number{number: ref n2, ..} = val2.variant else { unreachable!("has been checked."); };
    let mut _result = val1;
    Value::from_number(Number { significand: n1.significand * n2.significand, exponent: 0, unit : Unit { range: None, id: "".to_string() }, fmt: NumberFormat::Dec }, range)
}

pub fn load_operators(globals: &mut Globals) {
    globals.operators.insert(operator_id_from(ValueType::Number, OperatorType::Plus, ValueType::Number), op_num_plus_num);
    globals.operators.insert(operator_id_from(ValueType::Number, OperatorType::Min, ValueType::Number), op_num_min_num);
    globals.operators.insert(operator_id_from(ValueType::Number, OperatorType::Mult, ValueType::Number), op_num_mult_num);
}

fn do_term(units_view: &UnitsView, v1: &Number, adding: bool, v2: &Number, range: &Range, globals: &Globals) -> Number {
    //if both values have units: convert them to SI before operation.
    if !v1.unit.is_empty() && !v2.unit.is_empty() {
        let u1 = units_view.get_def(&v1.unit.id, &globals.unit_defs);
        let u2 = units_view.get_def(&v2.unit.id, &globals.unit_defs);
        if let (Some(u1), Some(u2)) = (u1, u2) {
            if u1.property != u2.property {
                panic!("TODO: implement errors.");
            }
        }
        let d1 = v1.to_si(&units_view, &globals);
        let d2 = v2.to_si(&units_view, &globals);
        let result = match adding {
            true => d1 + d2,
            false => d1 - d2
        };
        let mut num = Number::from(result);
        if units_view.units.contains(&*v1.unit.id) {
            let from_si = globals.unit_defs[&*v1.unit.id].from_si;
            num.significand = from_si(&globals.unit_defs[&*v1.unit.id], result);
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
